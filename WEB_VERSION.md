# Antigravity Web Version Documentation

This document describes the decoupled Web Version of Antigravity Tools, implemented to run in a standard browser environment while reusing the core logic of the original Desktop (Tauri) application.

## 1. API Mapping & Architecture

We have introduced a "Bridge" architecture to decouple the frontend from the Tauri runtime.

### Architecture Overview
*   **Original Core (`src-tauri/`):** Contains the business logic (`modules`), data models (`models`), and proxy engine (`proxy`). Used as a library.
*   **Web Bridge (`web-bridge/`):** A standalone Rust web server (`axum`) that imports the Original Core library. It exposes a REST API to perform management tasks (Add Account, Config, etc.) that were previously Tauri Commands. It also spawns the actual AI Proxy service.
*   **Web Frontend (`web/`):** A standard React application (migrated from the original source) that communicates with the Web Bridge via HTTP REST API instead of Tauri IPC.

### API Mapping Table

| Feature | Original Tauri Command | New Bridge REST API | Frontend Client (`apiClient`) |
| :--- | :--- | :--- | :--- |
| **Accounts** | `list_accounts` | `GET /api/admin/accounts` | `listAccounts()` |
| | `add_account` | `POST /api/admin/accounts` | `addAccount(token)` |
| | `delete_account` | `DELETE /api/admin/accounts/:id` | `deleteAccount(id)` |
| | `switch_account` | `POST /api/admin/accounts/:id/switch` | `switchAccount(id)` |
| | `get_current_account` | `GET /api/admin/accounts/current` | `getCurrentAccount()` |
| | `fetch_account_quota` | `GET /api/admin/accounts/:id/quota` | `fetchAccountQuota(id)` |
| | `toggle_proxy_status` | `POST /api/admin/accounts/:id/toggle_proxy` | `toggleProxyStatus(id, ...)` |
| **Config** | `load_config` | `GET /api/admin/config` | `loadConfig()` |
| | `save_config` | `POST /api/admin/config` | `saveConfig(cfg)` |
| **OAuth** | `prepare_oauth_url` | `GET /api/admin/oauth/url` | `getOAuthUrl()` |
| **Proxy** | `start_proxy_service` | `POST /api/admin/proxy/start` | `startProxy()` |
| | `stop_proxy_service` | `POST /api/admin/proxy/stop` | `stopProxy()` |
| | `get_proxy_status` | `GET /api/admin/proxy/status` | `getProxyStatus()` |
| | `get_proxy_stats` | `GET /api/admin/proxy/stats` | `getProxyStats()` |
| | `get_proxy_logs` | `GET /api/admin/proxy/logs` | `getProxyLogs()` |

---

## 2. Implemented Functionalities

The Web Version currently supports the following core capabilities:

1.  **Dashboard Monitoring**: View global quota usage, active account status, and system health (reused existing Dashboard UI).
2.  **Account Management**:
    *   List all accounts.
    *   Add new accounts via **Refresh Token** (OAuth flow is simplified to link generation).
    *   Switch active account.
    *   Manually refresh quotas.
    *   Toggle proxy availability for specific accounts.
    *   Delete accounts.
3.  **Configuration**:
    *   Modify system settings (Theme, Language).
    *   Configure Proxy settings (Port, Timeout).
    *   **Hot Reload**: Changing settings in the Web UI immediately updates the running Proxy service in the backend.
4.  **AI Proxy Service**:
    *   Start/Stop the AI Proxy (running on port 8045 by default).
    *   Monitor proxy status.
    *   Full compatibility with OpenAI/Anthropic/Gemini clients.

*Limitations:* Native file dialogs (Import/Export) are replaced by browser-based downloads or disabled where file system access is restricted.

---

## 3. Deployment Guide

### Prerequisites
*   **Rust**: Latest stable version.
*   **Node.js**: v18 or higher (for building frontend).
*   **Linux Dependencies** (if on Linux): `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `libatspi-dev` (required by the underlying Tauri libs even if we don't use the GUI).

### Build Steps

1.  **Build the Frontend**:
    ```bash
    cd web
    npm install
    npm run build
    # Output will be in web/dist/
    ```

2.  **Build the Backend Bridge**:
    ```bash
    cd web-bridge
    cargo build --release
    # Binary will be in web-bridge/target/release/web-bridge
    ```

### Running

1.  Ensure the `web/dist` folder exists (the backend serves static files from there relative to its execution path, or you can configure it). The current code expects `../web/dist` relative to CWD.
2.  Run the bridge:
    ```bash
    cd web-bridge
    ./target/release/web-bridge
    ```
3.  Access the Web UI at: **http://localhost:8090**
4.  The AI Proxy service (when started via UI) will listen on **http://localhost:8045** (or configured port).

---

## 4. Maintenance & Update Guide (For Future Developers/AI)

**Principle: Strict Isolation & Minimal Intrusion**

We have isolated our code into `web/` (Frontend) and `web-bridge/` (Backend). The original `src-tauri/` code is treated as an **upstream library**.

### How to Update (Pull Upstream Changes)

If you pull the latest code from the original `Antigravity-Manager` repository, you might overwrite our minimal modifications. **You must re-apply them.**

**The ONLY modification we make to the original source is in `src-tauri/src/lib.rs`:**

We change the visibility of internal modules to `pub` so our `web-bridge` can use them.

**Steps to Restore Compatibility after `git pull`:**

1.  Open `src-tauri/src/lib.rs`.
2.  Ensure the module declarations look like this (add `pub`):
    ```rust
    pub mod models;   // Change 'mod' to 'pub mod'
    pub mod modules;  // Change 'mod' to 'pub mod'
    pub mod commands; // Change 'mod' to 'pub mod'
    pub mod utils;    // Change 'mod' to 'pub mod'
    pub mod proxy;    // Change 'mod' to 'pub mod'
    pub mod error;
    ```
3.  That's it. Do not modify business logic inside `src-tauri/`. If you need to change logic, implement a wrapper in `web-bridge/src/main.rs`.

### Working on the Code

*   **Frontend Changes**: Edit files in `web/`. Do NOT edit `src/`. The `web/` folder is a decoupled fork.
*   **Backend Changes**: Edit files in `web-bridge/`. Do NOT edit `src-tauri/` unless absolutely necessary to expose new internal functions.
