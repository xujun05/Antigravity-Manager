// utils/request.ts
// Stub for request to allow existing code to import it if missed during refactor
// But ideally should not be used.

// Mock invoke to log error if still used
export async function request<T>(cmd: string, args?: any): Promise<T> {
    console.error(`Attempted to call Tauri invoke: ${cmd}`, args);
    throw new Error(`Tauri invoke not supported in Web Mode: ${cmd}`);
}
