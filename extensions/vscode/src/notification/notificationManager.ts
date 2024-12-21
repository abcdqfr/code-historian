import { window, StatusBarItem, StatusBarAlignment } from 'vscode';
import { ConfigurationManager } from '../configuration/configManager';

export enum NotificationType {
    Error = 'error',
    Warning = 'warning',
    Info = 'info',
    Success = 'success'
}

export interface NotificationOptions {
    type: NotificationType;
    message: string;
    detail?: string;
    buttons?: string[];
    persistent?: boolean;
    statusBarMessage?: string;
}

export class NotificationManager {
    private statusBarItem: StatusBarItem;
    private configManager: ConfigurationManager;
    private activeNotifications: Map<string, any>;

    constructor(configManager: ConfigurationManager) {
        this.configManager = configManager;
        this.statusBarItem = window.createStatusBarItem(StatusBarAlignment.Right);
        this.activeNotifications = new Map();
    }

    public async showNotification(options: NotificationOptions): Promise<void> {
        const settings = await this.configManager.getNotificationSettings();
        if (!settings.enabled) {
            return;
        }

        const notificationId = `${Date.now()}-${Math.random()}`;
        let notification;

        switch (options.type) {
            case NotificationType.Error:
                notification = options.buttons
                    ? window.showErrorMessage(options.message, ...options.buttons)
                    : window.showErrorMessage(options.message);
                break;
            case NotificationType.Warning:
                notification = options.buttons
                    ? window.showWarningMessage(options.message, ...options.buttons)
                    : window.showWarningMessage(options.message);
                break;
            case NotificationType.Info:
                notification = options.buttons
                    ? window.showInformationMessage(options.message, ...options.buttons)
                    : window.showInformationMessage(options.message);
                break;
            case NotificationType.Success:
                notification = options.buttons
                    ? window.showInformationMessage(`✓ ${options.message}`, ...options.buttons)
                    : window.showInformationMessage(`✓ ${options.message}`);
                break;
        }

        if (options.persistent) {
            this.activeNotifications.set(notificationId, notification);
        }

        if (settings.showInStatusBar && options.statusBarMessage) {
            this.updateStatusBar(options.statusBarMessage, options.type);
        }
    }

    public async showErrorRecoveryNotification(error: Error, recoverySteps?: string[]): Promise<void> {
        const message = `Error Recovery: ${error.message}`;
        const detail = recoverySteps
            ? `Recovery steps:\n${recoverySteps.map((step, index) => `${index + 1}. ${step}`).join('\n')}`
            : undefined;

        await this.showNotification({
            type: NotificationType.Error,
            message,
            detail,
            buttons: ['Retry', 'Ignore'],
            persistent: true,
            statusBarMessage: 'Error Recovery in Progress'
        });
    }

    public async showStateRestorationNotification(success: boolean, details?: string): Promise<void> {
        if (success) {
            await this.showNotification({
                type: NotificationType.Success,
                message: 'State restored successfully',
                detail: details,
                statusBarMessage: 'State Restored'
            });
        } else {
            await this.showNotification({
                type: NotificationType.Warning,
                message: 'State restoration incomplete',
                detail: details,
                buttons: ['Retry', 'Reset to Defaults'],
                persistent: true,
                statusBarMessage: 'State Restoration Failed'
            });
        }
    }

    public async showNetworkRecoveryNotification(retryCount: number): Promise<void> {
        await this.showNotification({
            type: NotificationType.Warning,
            message: `Network recovery attempt ${retryCount}`,
            buttons: ['Retry Now', 'Cancel'],
            persistent: true,
            statusBarMessage: 'Network Recovery'
        });
    }

    public async showDataCorruptionNotification(dataType: string, fallbackAction: string): Promise<void> {
        await this.showNotification({
            type: NotificationType.Error,
            message: `Data corruption detected in ${dataType}`,
            detail: `Fallback action: ${fallbackAction}`,
            buttons: ['Apply Fallback', 'Try Recovery', 'Reset Data'],
            persistent: true,
            statusBarMessage: 'Data Corruption Detected'
        });
    }

    public async clearNotification(id: string): Promise<void> {
        const notification = this.activeNotifications.get(id);
        if (notification) {
            this.activeNotifications.delete(id);
            // If the notification has a hide method, call it
            if (notification.hide) {
                notification.hide();
            }
        }
    }

    public async clearAllNotifications(): Promise<void> {
        for (const id of this.activeNotifications.keys()) {
            await this.clearNotification(id);
        }
    }

    private updateStatusBar(message: string, type: NotificationType): void {
        let icon = '';
        switch (type) {
            case NotificationType.Error:
                icon = '$(error)';
                break;
            case NotificationType.Warning:
                icon = '$(warning)';
                break;
            case NotificationType.Info:
                icon = '$(info)';
                break;
            case NotificationType.Success:
                icon = '$(check)';
                break;
        }

        this.statusBarItem.text = `${icon} ${message}`;
        this.statusBarItem.show();

        // Hide status bar item after 5 seconds unless it's an error
        if (type !== NotificationType.Error) {
            setTimeout(() => {
                this.statusBarItem.hide();
            }, 5000);
        }
    }

    public dispose(): void {
        this.statusBarItem.dispose();
        this.clearAllNotifications();
    }
} 