// services/configService.ts
// Refactored to use apiClient

import { apiClient } from './api';

export interface AppConfig {
    // Define the shape based on usage or import if possible.
    // For now using any to match original loose typing or if types were imported.
    [key: string]: any;
}

export const loadConfig = async (): Promise<AppConfig> => {
    return await apiClient.loadConfig();
};

export const saveConfig = async (config: AppConfig): Promise<void> => {
    return await apiClient.saveConfig(config);
};
