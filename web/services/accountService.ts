// services/accountService.ts
// Refactored to use apiClient instead of Tauri invoke

import { apiClient, Account } from './api';

export const listAccounts = async (): Promise<Account[]> => {
    return await apiClient.listAccounts();
};

export const getCurrentAccount = async (): Promise<Account | null> => {
    return await apiClient.getCurrentAccount();
};

export const addAccount = async (email: string, refreshToken: string): Promise<Account> => {
    // Note: email param is ignored by backend logic anyway, but kept for signature compatibility if needed
    return await apiClient.addAccount(refreshToken);
};

export const deleteAccount = async (accountId: string): Promise<void> => {
    return await apiClient.deleteAccount(accountId);
};

export const deleteAccounts = async (accountIds: string[]): Promise<void> => {
    // We didn't implement bulk delete in the bridge yet. Loop or implement?
    // Let's implement loop here for simplicity or skip if not critical.
    // Ideally backend should support it.
    for (const id of accountIds) {
        await apiClient.deleteAccount(id);
    }
};

export const switchAccount = async (accountId: string): Promise<void> => {
    return await apiClient.switchAccount(accountId);
};

export const fetchAccountQuota = async (accountId: string): Promise<any> => {
    return await apiClient.fetchAccountQuota(accountId);
};

export const refreshAllQuotas = async (): Promise<any> => {
    return await apiClient.refreshAllQuotas();
};

export const startOAuthLogin = async (): Promise<any> => {
    // Web Flow:
    // 1. Get URL
    const url = await apiClient.getOAuthUrl();
    // 2. We can't "await" the result like in Tauri because we need to redirect the user or show a link.
    // However, the original code expects a promise that resolves when login is complete.
    // This is hard in a web context without opening a popup or redirecting.

    // For now, let's just return a rejected promise or throw to indicate UI should handle it differently?
    // Or we assume this function is called when user clicks "Add Account".
    // We should probably change the UI flow.

    // But since we want "minimal frontend changes", we can try to simulate it:
    // Open a new window?
    window.open(url, '_blank');

    // Then we need to poll or wait for user to click "I finished".
    // The bridge backend route for oauth/wait isn't implemented yet.
    throw new Error("Please check the opened window to login. (Polling not implemented yet)");
};

export const completeOAuthLogin = async (): Promise<any> => {
     // Stub
     throw new Error("Not implemented in Web Mode");
};

export const cancelOAuthLogin = async (): Promise<void> => {
     // Stub
};

export const importV1Accounts = async (): Promise<any[]> => {
    // Not implemented
    return [];
};

export const importFromDb = async (): Promise<any> => {
     // Not feasible in browser unless we upload the file
     throw new Error("Import from local DB not supported in Web Mode");
};

export const importCustomDb = async (path: string): Promise<any> => {
     throw new Error("Import from local DB not supported in Web Mode");
};

export const syncAccountFromDb = async (): Promise<any> => {
     return null;
};

export const toggleProxyStatus = async (accountId: string, enable: boolean, reason?: string): Promise<void> => {
    return await apiClient.toggleProxyStatus(accountId, enable, reason);
};

export const reorderAccounts = async (accountIds: string[]): Promise<void> => {
    // Not implemented in bridge
    // await apiClient.reorderAccounts(accountIds);
};
