import { workspace, WorkspaceConfiguration } from 'vscode';
import { URL } from 'url';

interface NotificationSettings {
    enabled: boolean;
    showInStatusBar: boolean;
}

interface AnalysisSettings {
    maxDepth: number;
    excludePaths: string[];
}

interface UIState {
    treeViewCollapsed: boolean;
    selectedAnalysis: string | null;
    lastRefresh: number;
}

export class ConfigurationManager {
    private readonly config: WorkspaceConfiguration;
    private readonly SECURE_PREFIX = 'secure:';

    constructor() {
        this.config = workspace.getConfiguration('codeHistorian');
    }

    public async getServerUrl(): Promise<string> {
        const serverUrl = this.config.get<string>('serverUrl');
        if (serverUrl) {
            return serverUrl;
        }

        // Check for legacy setting
        const legacyUrl = this.config.get<string>('legacyServerUrl');
        if (legacyUrl) {
            await this.migrateServerUrl(legacyUrl);
            return legacyUrl;
        }

        return this.getDefaultSettings().serverUrl;
    }

    public async setServerUrl(url: string): Promise<void> {
        if (!this.isValidUrl(url)) {
            throw new Error('Invalid server URL');
        }
        await this.config.update('serverUrl', url, true);
    }

    public async getApiKey(): Promise<string> {
        const apiKey = this.config.get<string>('apiKey');
        return apiKey ? this.decryptApiKey(apiKey) : '';
    }

    public async setApiKey(key: string): Promise<void> {
        if (!key) {
            throw new Error('API key cannot be empty');
        }
        await this.config.update('apiKey', this.encryptApiKey(key), true);
    }

    public async getNotificationSettings(): Promise<NotificationSettings> {
        return {
            enabled: this.config.get<boolean>('notifications.enabled', true),
            showInStatusBar: this.config.get<boolean>('notifications.showInStatusBar', true)
        };
    }

    public async updateNotificationSettings(settings: NotificationSettings): Promise<void> {
        await this.config.update('notifications.enabled', settings.enabled, true);
        await this.config.update('notifications.showInStatusBar', settings.showInStatusBar, true);
    }

    public async getAnalysisSettings(): Promise<AnalysisSettings> {
        return {
            maxDepth: this.config.get<number>('analysis.maxDepth', 100),
            excludePaths: this.config.get<string[]>('analysis.excludePaths', ['node_modules'])
        };
    }

    public async updateAnalysisSettings(settings: AnalysisSettings): Promise<void> {
        await this.config.update('analysis.maxDepth', settings.maxDepth, true);
        await this.config.update('analysis.excludePaths', settings.excludePaths, true);
    }

    public async getUIState(): Promise<UIState> {
        const defaultState: UIState = {
            treeViewCollapsed: false,
            selectedAnalysis: null,
            lastRefresh: 0
        };
        return this.config.get<UIState>('ui.state', defaultState);
    }

    public async saveUIState(state: UIState): Promise<void> {
        await this.config.update('ui.state', state, true);
    }

    public async resetToDefaults(): Promise<void> {
        const defaults = this.getDefaultSettings();
        await this.setServerUrl(defaults.serverUrl);
        await this.config.update('apiKey', '', true);
        await this.updateNotificationSettings(defaults.notifications);
        await this.updateAnalysisSettings(defaults.analysis);
        await this.saveUIState(defaults.ui);
    }

    public getDefaultSettings() {
        return {
            serverUrl: 'http://localhost:3000',
            notifications: {
                enabled: true,
                showInStatusBar: true
            },
            analysis: {
                maxDepth: 100,
                excludePaths: ['node_modules']
            },
            ui: {
                treeViewCollapsed: false,
                selectedAnalysis: null,
                lastRefresh: 0
            }
        };
    }

    private async migrateServerUrl(legacyUrl: string): Promise<void> {
        await this.setServerUrl(legacyUrl);
        await this.config.update('legacyServerUrl', undefined, true);
    }

    private isValidUrl(urlString: string): boolean {
        try {
            new URL(urlString);
            return true;
        } catch {
            return false;
        }
    }

    private encryptApiKey(key: string): string {
        // In a real implementation, this would use proper encryption
        // For now, we just add a prefix to simulate secure storage
        return `${this.SECURE_PREFIX}${key}`;
    }

    private decryptApiKey(encryptedKey: string): string {
        // In a real implementation, this would use proper decryption
        // For now, we just remove the prefix
        return encryptedKey.startsWith(this.SECURE_PREFIX)
            ? encryptedKey.slice(this.SECURE_PREFIX.length)
            : encryptedKey;
    }
} 