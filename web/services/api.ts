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

/**
 * Helper to fetch JSON and handle errors gracefully.
 * Specifically checks if the response is HTML (which happens when API is missing and SPA fallback kicks in).
 */
async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
    const res = await fetch(url, options);

    // Check for HTML response (SPA fallback)
    const contentType = res.headers.get('content-type');
    if (contentType && contentType.includes('text/html')) {
        const text = await res.text();
        // If it looks like the index.html fallback
        if (text.trim().toLowerCase().startsWith('<!doctype html>')) {
             throw new Error(`API endpoint not found (returned HTML fallback): ${url}`);
        }
        // Otherwise try to parse it? Unlikely to be valid JSON if HTML.
        throw new Error(`Expected JSON but got HTML from: ${url}`);
    }

    if (!res.ok) {
        const text = await res.text();
        throw new Error(`API Error ${res.status}: ${text || res.statusText}`);
    }

    try {
        return await res.json();
    } catch (e) {
        throw new Error(`Failed to parse JSON response from ${url}: ${e}`);
    }
}

export const apiClient = {
  // Accounts
  listAccounts: async (): Promise<Account[]> => {
    return await fetchJson<Account[]>(`${API_BASE}/accounts`);
  },

  addAccount: async (refreshToken: string): Promise<Account> => {
    return await fetchJson<Account>(`${API_BASE}/accounts`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
  },

  deleteAccount: async (id: string): Promise<void> => {
    // DELETE might return empty body, handle accordingly
    const res = await fetch(`${API_BASE}/accounts/${id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete account');
    // Don't parse JSON for DELETE usually
  },

  switchAccount: async (id: string): Promise<void> => {
    const res = await fetch(`${API_BASE}/accounts/${id}/switch`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to switch account');
  },

  getCurrentAccount: async (): Promise<Account | null> => {
     try {
         return await fetchJson<Account | null>(`${API_BASE}/accounts/current`);
     } catch (e) {
         // If current account is null/empty, backend might return 200 with null, or 404.
         // fetchJson handles 200/null correctly (returns null).
         // If backend returns 404, fetchJson throws.
         // We might want to return null on 404?
         // Original code: if (!res.ok) return null;
         // Let's replicate that behavior safely.
         // Actually, fetchJson throws on !res.ok.
         // We should catch and return null if it's a 404 or "not found".
         console.warn("Failed to fetch current account:", e);
         return null;
     }
  },

  refreshAllQuotas: async (): Promise<any> => {
    return await fetchJson<any>(`${API_BASE}/accounts/quota/refresh_all`, { method: 'POST' });
  },

  fetchAccountQuota: async (id: string): Promise<any> => {
    return await fetchJson<any>(`${API_BASE}/accounts/${id}/quota`);
  },

  toggleProxyStatus: async (id: string, enable: boolean, reason?: string): Promise<void> => {
      await fetchJson<void>(`${API_BASE}/accounts/${id}/toggle_proxy`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ enable, reason })
      });
  },

  // Config
  loadConfig: async (): Promise<any> => {
    return await fetchJson<any>(`${API_BASE}/config`);
  },

  saveConfig: async (config: any): Promise<void> => {
    await fetchJson<void>(`${API_BASE}/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
  },

  // OAuth
  getOAuthUrl: async (): Promise<string> => {
     return await fetchJson<string>(`${API_BASE}/oauth/url`);
  },

  // Proxy (Stubs for now as per backend)
  getProxyStatus: async (): Promise<any> => {
      try {
          return await fetchJson<any>(`${API_BASE}/proxy/status`);
      } catch (e) {
          console.warn("Proxy status fetch failed (maybe not implemented):", e);
          return { running: false, port: 0, active_accounts: 0 };
      }
  },

  getProxyLogs: async (): Promise<any[]> => {
      try {
          return await fetchJson<any[]>(`${API_BASE}/proxy/logs`);
      } catch (e) {
          return [];
      }
  },

  getProxyStats: async (): Promise<any> => {
      try {
          return await fetchJson<any>(`${API_BASE}/proxy/stats`);
      } catch (e) {
          return {};
      }
  },

  startProxy: async (): Promise<any> => {
      return await fetchJson<any>(`${API_BASE}/proxy/start`, { method: 'POST' });
  },

  stopProxy: async (): Promise<any> => {
      return await fetchJson<any>(`${API_BASE}/proxy/stop`, { method: 'POST' });
  }
};
