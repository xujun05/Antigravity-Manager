# Web Version - Upstream Code Modifications

This document tracks ALL modifications made to the original `src-tauri/` codebase to support the Web Version (headless mode).

**Purpose:** When pulling upstream changes, re-apply these modifications to maintain Web Version compatibility.

---

## Quick Reference

After `git pull` or `git merge` from upstream, check and re-apply these changes:

1. `src-tauri/Cargo.toml` - Feature flags and optional dependencies
2. `src-tauri/build.rs` - Conditional tauri_build
3. `src-tauri/src/lib.rs` - Conditional imports and pub mod
4. `src-tauri/src/error.rs` - Conditional Tauri error variant
5. `src-tauri/src/modules/mod.rs` - Conditional module exports
6. `src-tauri/src/proxy/monitor.rs` - Conditional tauri import

---

## Detailed Changes

### 1. `src-tauri/Cargo.toml`

**Purpose:** Make `tauri` and related dependencies optional, only required for desktop builds.

```toml
# BEFORE (upstream)
[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-opener = "2"
tauri-plugin-dialog = "2.4.2"
tauri-plugin-fs = "2.4.4"
tauri-plugin-single-instance = { version = "2.3.6", features = ["deep-link"] }
tauri-plugin-autostart = "2.5.1"

# AFTER (web-compatible)
[build-dependencies]
tauri-build = { version = "2", features = [], optional = true }

[features]
default = ["desktop"]
desktop = ["tauri", "tauri-build", "tauri/tray-icon", "tauri/image-png", "tauri-plugin-opener", "tauri-plugin-dialog", "tauri-plugin-fs", "tauri-plugin-single-instance", "tauri-plugin-autostart"]

[dependencies]
tauri = { version = "2", default-features = false, optional = true }
tauri-plugin-opener = { version = "2", optional = true }
tauri-plugin-dialog = { version = "2.4.2", optional = true }
tauri-plugin-fs = { version = "2.4.4", optional = true }
tauri-plugin-single-instance = { version = "2.3.6", features = ["deep-link"], optional = true }
tauri-plugin-autostart = { version = "2.5.1", optional = true }
```

---

### 2. `src-tauri/build.rs`

**Purpose:** Only call `tauri_build::build()` when desktop feature is enabled.

```rust
// BEFORE (upstream)
fn main() {
    tauri_build::build()
}

// AFTER (web-compatible)
fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::build()
}
```

---

### 3. `src-tauri/src/lib.rs`

**Purpose:**
- Change `mod` to `pub mod` for modules needed by web-bridge
- Add `#[cfg(feature = "desktop")]` guards for tauri-specific code

```rust
// BEFORE (upstream)
mod models;
mod modules;
mod commands;
mod utils;
mod proxy;
pub mod error;

use tauri::Manager;
use modules::logger;
use tracing::{info, error};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ... tauri app code
}

// AFTER (web-compatible)
pub mod models;
pub mod modules;
#[cfg(feature = "desktop")]
pub mod commands;
pub mod utils;
pub mod proxy;
pub mod error;

#[cfg(feature = "desktop")]
use tauri::Manager;
#[cfg(feature = "desktop")]
use modules::logger;
#[cfg(feature = "desktop")]
use tracing::{info, error};

#[cfg(feature = "desktop")]
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ... tauri app code
}

#[cfg(not(feature = "desktop"))]
pub fn run() {
    println!("Headless library mode active. GUI not available.");
}
```

---

### 4. `src-tauri/src/error.rs`

**Purpose:** Make the `Tauri` error variant conditional.

```rust
// BEFORE (upstream)
#[derive(Error, Debug)]
pub enum AppError {
    // ... other variants
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    // ... other variants
}

// AFTER (web-compatible)
#[derive(Error, Debug)]
pub enum AppError {
    // ... other variants
    #[cfg(feature = "desktop")]
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    // ... other variants
}
```

---

### 5. `src-tauri/src/modules/mod.rs`

**Purpose:** Make `oauth_server` module conditional (it depends on tauri::AppHandle).

```rust
// BEFORE (upstream)
pub mod oauth;
pub mod oauth_server;
// ...
#[cfg(feature = "desktop")]
pub mod tray;

// AFTER (web-compatible)
pub mod oauth;
#[cfg(feature = "desktop")]
pub mod oauth_server;
// ...
#[cfg(feature = "desktop")]
pub mod tray;
```

---

### 6. `src-tauri/src/proxy/monitor.rs`

**Purpose:** Make tauri imports and AppHandle conditional.

```rust
// BEFORE (upstream)
use tauri::Emitter;
// ...
pub struct ProxyMonitor {
    // ...
    pub app_handle: Option<tauri::AppHandle>,
}

// AFTER (web-compatible)
#[cfg(feature = "desktop")]
use tauri::Emitter;
// ...
pub struct ProxyMonitor {
    // ...
    #[cfg(feature = "desktop")]
    pub app_handle: Option<tauri::AppHandle>,
    #[cfg(not(feature = "desktop"))]
    pub app_handle: Option<()>,
}

impl ProxyMonitor {
    #[cfg(feature = "desktop")]
    pub fn new(max_logs: usize, app_handle: Option<tauri::AppHandle>) -> Self {
        // ... desktop implementation
    }

    #[cfg(not(feature = "desktop"))]
    pub fn new(max_logs: usize, _app_handle: Option<()>) -> Self {
        // ... headless implementation
    }
}
```

---

## Merge Workflow

When merging upstream changes:

```bash
# 1. Fetch and merge
git fetch origin main
git merge origin/main

# 2. If conflicts in src-tauri/, resolve by:
#    - Keep upstream logic changes
#    - Re-apply the cfg(feature) guards listed above

# 3. Verify web-bridge compiles
cd web-bridge
cargo check

# 4. If compilation fails, check error messages and apply missing guards
```

---

## Files That Should NOT Be Modified

These upstream files should remain unchanged:
- `src-tauri/src/proxy/` (except `monitor.rs` imports)
- `src-tauri/src/models/`
- `src-tauri/src/modules/` (except `mod.rs`)
- Business logic in any file

---

## Last Updated

- **Date:** 2026-01-08
- **Upstream Version:** v3.3.16 (commit 16ad298)
- **Web Branch:** web-decoupling-v1-2079836201000725839
