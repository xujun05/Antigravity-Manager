# Backend Bridge Isolation Marker

This directory contains the **Web Bridge Server**.

*   **Status**: Standalone Rust Binary.
*   **Role**:
    1.  Acts as a REST API server for the `web/` frontend.
    2.  Imports `antigravity_tools_lib` (from `../src-tauri`) to reuse core business logic.
    3.  Manages the lifecycle of the AI Proxy service (`AxumServer`).
*   **Isolation**: This code is separate from the Tauri Main process. It runs independently.

## Dependencies
It depends on `../src-tauri` as a local crate. Ensure `src-tauri/src/lib.rs` has `pub mod` for modules we need to access.
