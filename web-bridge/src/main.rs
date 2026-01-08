use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Html},
    routing::{get, post, delete},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

// Use the library crate
use antigravity_tools_lib::{modules, proxy};

// Global state for the Web Bridge
struct AppState {
    proxy_service: Arc<RwLock<Option<ProxyServiceInstance>>>,
    monitor: Arc<proxy::monitor::ProxyMonitor>,
}

struct ProxyServiceInstance {
    config: proxy::ProxyConfig,
    token_manager: Arc<proxy::TokenManager>,
    axum_server: proxy::AxumServer,
    server_handle: tokio::task::JoinHandle<()>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize DB (ensure it exists)
    // Note: The original app calls logger::init_logger() which might set up file logging.
    // We are just using fmt::init for now to see stdout.

    info!("Starting Antigravity Web Bridge...");

    // Initialize Monitor
    let monitor = Arc::new(proxy::monitor::ProxyMonitor::new(1000, None)); // None for AppHandle

    // Initialize State
    let state = Arc::new(AppState {
        proxy_service: Arc::new(RwLock::new(None)),
        monitor: monitor.clone(),
    });

    // Create the router
    let app = Router::new()
        // API Routes
        .route("/api/admin/accounts", get(list_accounts).post(add_account))
        .route("/api/admin/accounts/:id", delete(delete_account))
        .route("/api/admin/accounts/:id/toggle_proxy", post(toggle_proxy_status))
        .route("/api/admin/accounts/:id/switch", post(switch_account))
        .route("/api/admin/accounts/current", get(get_current_account))
        .route("/api/admin/accounts/quota/refresh_all", post(refresh_all_quotas))
        .route("/api/admin/accounts/:id/quota", get(fetch_account_quota)) // GET to fetch/refresh
        .route("/api/admin/config", get(load_config).post(save_config))
        .route("/api/admin/proxy/start", post(start_proxy))
        .route("/api/admin/proxy/stop", post(stop_proxy))
        .route("/api/admin/proxy/status", get(get_proxy_status))
        .route("/api/admin/proxy/stats", get(get_proxy_stats))
        .route("/api/admin/proxy/logs", get(get_proxy_logs))
        // OAuth Routes
        .route("/api/admin/oauth/url", get(get_oauth_url))
        // Serve Frontend (dist folder) with SPA fallback
        // For any route not matched, serve index.html to let React Router handle it
        .fallback_service(
            ServeDir::new("../web/dist")
                .not_found_service(ServeFile::new("../web/dist/index.html"))
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Run the server
    // Default backend port is 8045. We want the Web Bridge to run on a different port (e.g. 8090)
    // to act as the "Management Interface", but it can *spawn* the Proxy on 8045.
    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));

    info!("Web Bridge listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- Handlers ---

async fn list_accounts() -> impl IntoResponse {
    match modules::list_accounts() {
        Ok(accounts) => Json(accounts).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct AddAccountPayload {
    refresh_token: String,
}

async fn add_account(Json(payload): Json<AddAccountPayload>) -> impl IntoResponse {
    // Re-implementing logic from commands::add_account but without Tauri AppHandle
    // 1. Refresh Token
    let token_res = match modules::oauth::refresh_access_token(&payload.refresh_token).await {
        Ok(t) => t,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    // 2. Get User Info
    let user_info = match modules::oauth::get_user_info(&token_res.access_token).await {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    // 3. Construct TokenData
    let token = antigravity_tools_lib::models::TokenData::new(
        token_res.access_token,
        payload.refresh_token.clone(),
        token_res.expires_in,
        Some(user_info.email.clone()),
        None,
        None,
    );

    // 4. Save
    match modules::upsert_account(user_info.email.clone(), user_info.get_display_name(), token) {
        Ok(account) => {
            // 5. Trigger Quota Refresh (Async in background or sync here? Let's do sync for simplicity in bridge)
            let mut account = account;
            let _ = modules::account::fetch_quota_with_retry(&mut account).await;
            let _ = modules::update_account_quota(&account.id, account.quota.clone().unwrap_or_default());

            Json(account).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn delete_account(Path(id): Path<String>) -> impl IntoResponse {
    match modules::delete_account(&id) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn get_current_account() -> impl IntoResponse {
    match modules::get_current_account_id() {
        Ok(Some(id)) => match modules::load_account(&id) {
            Ok(account) => Json(Some(account)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        },
        Ok(None) => Json(None::<antigravity_tools_lib::models::Account>).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn switch_account(Path(id): Path<String>) -> impl IntoResponse {
    match modules::switch_account(&id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn refresh_all_quotas() -> impl IntoResponse {
    // We can't reuse the command exactly because it returns a specific struct,
    // but we can reuse the logic.
    match modules::list_accounts() {
        Ok(accounts) => {
            let mut success = 0;
            let mut failed = 0;
            for mut account in accounts {
                 if account.disabled { continue; }
                 match modules::account::fetch_quota_with_retry(&mut account).await {
                     Ok(quota) => {
                         let _ = modules::update_account_quota(&account.id, quota);
                         success += 1;
                     }
                     Err(_) => failed += 1,
                 }
            }
            Json(serde_json::json!({ "success": success, "failed": failed })).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn fetch_account_quota(Path(id): Path<String>) -> impl IntoResponse {
    match modules::load_account(&id) {
        Ok(mut account) => {
            match modules::account::fetch_quota_with_retry(&mut account).await {
                Ok(quota) => {
                    let _ = modules::update_account_quota(&id, quota.clone());
                    Json(quota).into_response()
                }
                Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            }
        },
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn load_config() -> impl IntoResponse {
    match modules::load_app_config() {
        Ok(config) => Json(config).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn save_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<antigravity_tools_lib::models::AppConfig>
) -> impl IntoResponse {
    match modules::save_app_config(&config) {
        Ok(_) => {
            // Hot Reload Logic
            let instance_lock = state.proxy_service.read().await;
            if let Some(instance) = instance_lock.as_ref() {
                // Update mapping
                instance.axum_server.update_mapping(&config.proxy).await;
                // Update proxy
                instance.axum_server.update_proxy(config.proxy.upstream_proxy.clone()).await;
                // Update security
                instance.axum_server.update_security(&config.proxy).await;
                // Update zai
                instance.axum_server.update_zai(&config.proxy).await;

                // Update monitor enabled state
                state.monitor.set_enabled(config.proxy.enable_logging);
            }
            StatusCode::OK.into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct ToggleProxyPayload {
    enable: bool,
    reason: Option<String>,
}

async fn toggle_proxy_status(Path(id): Path<String>, Json(payload): Json<ToggleProxyPayload>) -> impl IntoResponse {
    // Re-implement toggle logic (manual update since no helper)
    let data_dir = match modules::account::get_data_dir() {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let account_path = data_dir.join("accounts").join(format!("{}.json", id));

    if !account_path.exists() {
        return (StatusCode::NOT_FOUND, "Account not found").into_response();
    }

    // Read and update file
    // Simplified: Just read, toggle, write.
    // Ideally we should use the same struct logic.
    // But since `Account` struct is available, we can load, modify and save?
    // `modules::upsert_account` expects email/name/token.
    // We don't have a direct "update partial" function exposed easily.
    // Let's create a raw JSON modification here as in the original command.

    match std::fs::read_to_string(&account_path) {
        Ok(content) => {
            let mut json: serde_json::Value = match serde_json::from_str(&content) {
                Ok(j) => j,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            };

            if payload.enable {
                json["proxy_disabled"] = serde_json::Value::Bool(false);
                json["proxy_disabled_reason"] = serde_json::Value::Null;
                json["proxy_disabled_at"] = serde_json::Value::Null;
            } else {
                let now = chrono::Utc::now().timestamp();
                json["proxy_disabled"] = serde_json::Value::Bool(true);
                json["proxy_disabled_at"] = serde_json::Value::Number(now.into());
                json["proxy_disabled_reason"] = serde_json::Value::String(
                    payload.reason.unwrap_or_else(|| "User disabled via Web".to_string())
                );
            }

            if let Err(e) = std::fs::write(&account_path, serde_json::to_string_pretty(&json).unwrap()) {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }

            StatusCode::OK.into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// OAuth Helper
async fn get_oauth_url() -> impl IntoResponse {
    // We need to implement a "copy link" flow since we can't open a browser on the client machine from the server easily (if it's remote).
    // But since this is a local web bridge, we can return the URL.
    // The original logic starts a listener. We might need to start that listener here too?
    // This is complex because `modules::oauth_server` depends on `tauri::AppHandle` to emit events.
    // We cannot easily use `modules::oauth_server` without an AppHandle.
    //
    // ALTERNATIVE: Use the standard OOB (Out of Band) flow or just return the URL and let the user paste the code?
    // Google deprecated OOB.
    // We likely need to reimplement a simple OAuth flow that doesn't depend on Tauri.

    // For now, let's just return a static string saying "Not Implemented in Web Bridge yet"
    // or try to construct the URL manually if we have the client ID.

    Json("OAuth flow requires Tauri AppHandle currently. Please use Refresh Token login.").into_response()
}


// Proxy Control Implementation

#[derive(serde::Deserialize)]
struct StartProxyPayload {
    // Optional override config, otherwise load from DB
}

async fn start_proxy(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut instance_lock = state.proxy_service.write().await;

    if instance_lock.is_some() {
        return (StatusCode::CONFLICT, "Proxy service already running").into_response();
    }

    // 1. Load Config
    let app_config = match modules::load_app_config() {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };
    let config = app_config.proxy;

    // 2. Initialize TokenManager
    let app_data_dir = match modules::account::get_data_dir() {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let accounts_dir = app_data_dir.clone();

    let token_manager = Arc::new(proxy::TokenManager::new(accounts_dir));
    token_manager.update_sticky_config(config.scheduling.clone()).await;

    // 3. Load Accounts
    match token_manager.load_accounts().await {
        Ok(count) => {
             // Logic from original: check count > 0 unless zai is enabled.
             let zai_enabled = config.zai.enabled && !matches!(config.zai.dispatch_mode, proxy::ZaiDispatchMode::Off);
             if count == 0 && !zai_enabled {
                 return (StatusCode::BAD_REQUEST, "No accounts available").into_response();
             }
        },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 4. Start Axum Server
    state.monitor.set_enabled(config.enable_logging);

    let (axum_server, server_handle) = match proxy::AxumServer::start(
        config.get_bind_address().to_string(),
        config.port,
        token_manager.clone(),
        config.anthropic_mapping.clone(),
        config.openai_mapping.clone(),
        config.custom_mapping.clone(),
        config.request_timeout,
        config.upstream_proxy.clone(),
        proxy::ProxySecurityConfig::from_proxy_config(&config),
        config.zai.clone(),
        state.monitor.clone(),
        config.experimental.clone(),
    ).await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 5. Save Instance
    let instance = ProxyServiceInstance {
        config: config.clone(),
        token_manager,
        axum_server,
        server_handle,
    };
    *instance_lock = Some(instance);

    info!("Proxy Service Started on port {}", config.port);

    Json(serde_json::json!({
        "running": true,
        "port": config.port,
        "base_url": format!("http://{}:{}", config.get_bind_address(), config.port),
        "active_accounts": 0 // Dynamically fetch if needed, or return initial
    })).into_response()
}

async fn stop_proxy(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut instance_lock = state.proxy_service.write().await;

    if let Some(instance) = instance_lock.take() {
        instance.axum_server.stop();
        // Wait for it? or just let it drop. `server_handle` might need to be aborted or awaited.
        // In original code: instance.server_handle.await.ok();
        // Here we spawn a detach or just drop it.
        // Dropping the server instance triggers shutdown via channel if implemented correctly.
        // The original `AxumServer::stop` sends a signal.
        // We can await the handle if we want synchronous stop confirmation.
        // Since we are in an async handler, we can await it (but we hold the lock).
        // Best to drop lock then await? But we `take()`-d it, so lock is free.
        // Actually we hold the write lock until the end of scope.
        // Let's simply drop it.
    }

    StatusCode::OK.into_response()
}

async fn get_proxy_status(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let instance_lock = state.proxy_service.read().await;

    match instance_lock.as_ref() {
        Some(instance) => {
            Json(serde_json::json!({
                "running": true,
                "port": instance.config.port,
                "base_url": format!("http://{}:{}", instance.config.get_bind_address(), instance.config.port),
                "active_accounts": instance.token_manager.len()
            })).into_response()
        },
        None => {
            Json(serde_json::json!({
                "running": false,
                "port": 0,
                "base_url": "",
                "active_accounts": 0
            })).into_response()
        }
    }
}

async fn get_proxy_stats(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(state.monitor.get_stats().await).into_response()
}

async fn get_proxy_logs(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Default limit 100
    Json(state.monitor.get_logs(100).await).into_response()
}
