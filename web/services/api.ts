// api.ts
// Abstract layer for API calls to decouple from Tauri

const API_BASE = '/api/admin';

export interface Account {
  id: string;
  email: string;
  disabled: boolean;
  quota?: any;
  token?: any;
  // Add other fields as necessary from your Account model
}

export const apiClient = {
  // Accounts
  listAccounts: async (): Promise<Account[]> => {
    const res = await fetch(`${API_BASE}/accounts`);
    if (!res.ok) throw new Error('Failed to list accounts');
    return res.json();
  },

  addAccount: async (refreshToken: string): Promise<Account> => {
    const res = await fetch(`${API_BASE}/accounts`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
    if (!res.ok) {
       const err = await res.text();
       throw new Error(`Failed to add account: ${err}`);
    }
    return res.json();
  },

  deleteAccount: async (id: string): Promise<void> => {
    const res = await fetch(`${API_BASE}/accounts/${id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete account');
  },

  switchAccount: async (id: string): Promise<void> => {
    const res = await fetch(`${API_BASE}/accounts/${id}/switch`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to switch account');
  },

  getCurrentAccount: async (): Promise<Account | null> => {
     const res = await fetch(`${API_BASE}/accounts/current`);
     if (!res.ok) return null;
     return res.json();
  },

  refreshAllQuotas: async (): Promise<any> => {
    const res = await fetch(`${API_BASE}/accounts/quota/refresh_all`, { method: 'POST' });
    return res.json();
  },

  fetchAccountQuota: async (id: string): Promise<any> => {
    const res = await fetch(`${API_BASE}/accounts/${id}/quota`);
    if (!res.ok) throw new Error('Failed to fetch quota');
    return res.json();
  },

  toggleProxyStatus: async (id: string, enable: boolean, reason?: string): Promise<void> => {
      const res = await fetch(`${API_BASE}/accounts/${id}/toggle_proxy`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ enable, reason })
      });
      if (!res.ok) throw new Error('Failed to toggle proxy status');
  },

  // Config
  loadConfig: async (): Promise<any> => {
    const res = await fetch(`${API_BASE}/config`);
    if (!res.ok) throw new Error('Failed to load config');
    return res.json();
  },

  saveConfig: async (config: any): Promise<void> => {
    const res = await fetch(`${API_BASE}/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    if (!res.ok) throw new Error('Failed to save config');
  },

  // OAuth
  getOAuthUrl: async (): Promise<string> => {
     const res = await fetch(`${API_BASE}/oauth/url`);
     if (!res.ok) throw new Error('Failed to get OAuth URL');
     return res.json();
  },

  // Proxy (Stubs for now as per backend)
  getProxyStatus: async (): Promise<any> => {
      // Mock response for now as backend returns 501
      return { running: false, port: 0, active_accounts: 0 };
  },

  getProxyLogs: async (): Promise<any[]> => {
      return [];
  },

  getProxyStats: async (): Promise<any> => {
      return {};
  }
};
