import * as assert from 'assert';
import * as sinon from 'sinon';
import { window, StatusBarItem, StatusBarAlignment } from 'vscode';
import { NotificationManager, NotificationType } from '../../notification/notificationManager';
import { ConfigurationManager } from '../../configuration/configManager';

suite('Notification Manager Test Suite', () => {
    let notificationManager: NotificationManager;
    let configManager: ConfigurationManager;
    let mockWindow: sinon.SinonMock;
    let mockStatusBarItem: sinon.SinonStubbedInstance<StatusBarItem>;

    setup(() => {
        // Mock window
        mockWindow = sinon.mock(window);
        
        // Mock status bar item
        mockStatusBarItem = {
            text: '',
            show: sinon.stub(),
            hide: sinon.stub(),
            dispose: sinon.stub(),
            tooltip: '',
            command: undefined,
            color: undefined,
            backgroundColor: undefined,
            alignment: StatusBarAlignment.Right,
            priority: 0,
            name: 'test'
        };
        
        sinon.stub(window, 'createStatusBarItem').returns(mockStatusBarItem);
        
        // Mock config manager
        configManager = new ConfigurationManager();
        sinon.stub(configManager, 'getNotificationSettings').resolves({
            enabled: true,
            showInStatusBar: true
        });
        
        notificationManager = new NotificationManager(configManager);
    });

    teardown(() => {
        sinon.restore();
    });

    test('shows error recovery notification', async () => {
        const error = new Error('Test error');
        const recoverySteps = ['Step 1', 'Step 2'];
        
        const showErrorMessage = mockWindow.expects('showErrorMessage')
            .once()
            .withArgs(
                'Error Recovery: Test error',
                'Retry',
                'Ignore'
            );
        
        await notificationManager.showErrorRecoveryNotification(error, recoverySteps);
        
        assert.ok(showErrorMessage.verify());
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('Error Recovery'));
    });

    test('shows state restoration success notification', async () => {
        const showInfoMessage = mockWindow.expects('showInformationMessage')
            .once()
            .withArgs('✓ State restored successfully');
        
        await notificationManager.showStateRestorationNotification(true, 'All data restored');
        
        assert.ok(showInfoMessage.verify());
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('State Restored'));
    });

    test('shows state restoration failure notification', async () => {
        const showWarningMessage = mockWindow.expects('showWarningMessage')
            .once()
            .withArgs(
                'State restoration incomplete',
                'Retry',
                'Reset to Defaults'
            );
        
        await notificationManager.showStateRestorationNotification(false, 'Partial data loss');
        
        assert.ok(showWarningMessage.verify());
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('State Restoration Failed'));
    });

    test('shows network recovery notification', async () => {
        const showWarningMessage = mockWindow.expects('showWarningMessage')
            .once()
            .withArgs(
                'Network recovery attempt 3',
                'Retry Now',
                'Cancel'
            );
        
        await notificationManager.showNetworkRecoveryNotification(3);
        
        assert.ok(showWarningMessage.verify());
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('Network Recovery'));
    });

    test('shows data corruption notification', async () => {
        const showErrorMessage = mockWindow.expects('showErrorMessage')
            .once()
            .withArgs(
                'Data corruption detected in settings',
                'Apply Fallback',
                'Try Recovery',
                'Reset Data'
            );
        
        await notificationManager.showDataCorruptionNotification('settings', 'Reset to defaults');
        
        assert.ok(showErrorMessage.verify());
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('Data Corruption'));
    });

    test('respects notification settings when disabled', async () => {
        (configManager.getNotificationSettings as sinon.SinonStub).resolves({
            enabled: false,
            showInStatusBar: true
        });
        
        const showErrorMessage = mockWindow.expects('showErrorMessage').never();
        
        await notificationManager.showErrorRecoveryNotification(new Error('Test error'));
        
        assert.ok(showErrorMessage.verify());
        assert.ok(!mockStatusBarItem.show.called);
    });

    test('clears notifications', async () => {
        // Show a notification first
        const notification = {
            hide: sinon.stub()
        };
        
        mockWindow.expects('showErrorMessage')
            .once()
            .returns(Promise.resolve(notification));
        
        await notificationManager.showErrorRecoveryNotification(new Error('Test error'));
        
        // Clear all notifications
        await notificationManager.clearAllNotifications();
        
        assert.ok(notification.hide.called);
    });

    test('handles status bar updates', async () => {
        await notificationManager.showNotification({
            type: NotificationType.Success,
            message: 'Test message',
            statusBarMessage: 'Status bar test'
        });
        
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('Status bar test'));
        
        // Wait for auto-hide timeout
        await new Promise(resolve => setTimeout(resolve, 5500));
        assert.ok(mockStatusBarItem.hide.called);
    });

    test('keeps error messages in status bar', async () => {
        await notificationManager.showNotification({
            type: NotificationType.Error,
            message: 'Test error',
            statusBarMessage: 'Error test'
        });
        
        assert.ok(mockStatusBarItem.show.called);
        assert.ok(mockStatusBarItem.text.includes('Error test'));
        
        // Wait for auto-hide timeout
        await new Promise(resolve => setTimeout(resolve, 5500));
        assert.ok(!mockStatusBarItem.hide.called);
    });

    test('disposes resources properly', () => {
        notificationManager.dispose();
        assert.ok(mockStatusBarItem.dispose.called);
    });
}); 