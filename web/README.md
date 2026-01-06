# Frontend Code Isolation Marker

This directory contains the **Decoupled Web Frontend**.

*   **Status**: Standalone React App (migrated from `../src`).
*   **Communication**: Uses `services/api.ts` to talk to the `web-bridge` backend via REST API.
*   **Isolation**: This code is independent of the original `src/` directory. Updates to the original desktop frontend do **not** affect this folder automatically. You must manually port changes if desired.

## Key Changes from Original
1.  Removed `@tauri-apps/*` dependencies.
2.  Replaced `invoke` calls with `apiClient` calls.
3.  Disabled native features like `shell.open`, `dialog.save` (replaced with web equivalents).
