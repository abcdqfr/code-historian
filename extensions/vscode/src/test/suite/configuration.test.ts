import * as assert from 'assert';
import * as sinon from 'sinon';
import { workspace, WorkspaceConfiguration } from '../mock/vscode';
import { ConfigurationManager } from '../../configuration/configManager';

interface MockWorkspaceConfig extends WorkspaceConfiguration {
    get: sinon.SinonStub;
    update: sinon.SinonStub;
    has: sinon.SinonStub;
    inspect: sinon.SinonStub;
}

suite('Configuration Persistence Test Suite', () => {
    let configManager: ConfigurationManager;
    let mockWorkspace: sinon.SinonStubbedInstance<typeof workspace>;
    let mockConfig: MockWorkspaceConfig;

    setup(() => {
        mockConfig = {
            get: sinon.stub(),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        };
        mockWorkspace = sinon.stub(workspace);
        mockWorkspace.getConfiguration.returns(mockConfig);
        configManager = new ConfigurationManager();
    });

    teardown(() => {
        sinon.restore();
    });

    test('saves and retrieves server URL', async () => {
        const testUrl = 'http://test-server:3000';
        mockConfig.get.withArgs('serverUrl').returns(testUrl);
        
        const url = await configManager.getServerUrl();
        assert.strictEqual(url, testUrl);
        
        await configManager.setServerUrl('http://new-server:3000');
        assert.ok(mockConfig.update.calledOnce);
    });

    test('saves and retrieves API key securely', async () => {
        const testKey = 'test-api-key';
        mockConfig.get.withArgs('apiKey').returns(testKey);
        
        const key = await configManager.getApiKey();
        assert.strictEqual(key, testKey);
        
        await configManager.setApiKey('new-api-key');
        assert.ok(mockConfig.update.calledOnce);
        assert.ok(mockConfig.update.firstCall.args[1].startsWith('secure:'));
    });

    test('persists notification settings', async () => {
        mockConfig.get.withArgs('notifications.enabled').returns(true);
        mockConfig.get.withArgs('notifications.showInStatusBar').returns(true);
        
        const settings = await configManager.getNotificationSettings();
        assert.strictEqual(settings.enabled, true);
        assert.strictEqual(settings.showInStatusBar, true);
        
        await configManager.updateNotificationSettings({ enabled: false, showInStatusBar: false });
        assert.ok(mockConfig.update.calledTwice);
    });

    test('handles missing configuration gracefully', async () => {
        mockConfig.has.returns(false);
        mockConfig.get.returns(undefined);
        
        const settings = await configManager.getDefaultSettings();
        assert.ok(settings.serverUrl.includes('localhost'));
        assert.strictEqual(settings.notifications.enabled, true);
    });

    test('migrates legacy settings', async () => {
        mockConfig.get.withArgs('legacyServerUrl').returns('http://old-server:3000');
        mockConfig.has.withArgs('serverUrl').returns(false);
        
        const url = await configManager.getServerUrl();
        assert.strictEqual(url, 'http://old-server:3000');
        assert.ok(mockConfig.update.calledTwice); // Once for new setting, once for removing old
    });

    test('validates settings before saving', async () => {
        await assert.rejects(
            configManager.setServerUrl('invalid-url'),
            /Invalid server URL/
        );
        
        await assert.rejects(
            configManager.setApiKey(''),
            /API key cannot be empty/
        );
    });

    test('handles concurrent configuration updates', async () => {
        const promises = [
            configManager.setServerUrl('http://server1:3000'),
            configManager.setServerUrl('http://server2:3000'),
            configManager.setServerUrl('http://server3:3000')
        ];
        
        await Promise.all(promises);
        assert.strictEqual(mockConfig.update.callCount, 3);
    });

    test('persists analysis settings', async () => {
        mockConfig.get.withArgs('analysis.maxDepth').returns(100);
        mockConfig.get.withArgs('analysis.excludePaths').returns(['node_modules']);
        
        const settings = await configManager.getAnalysisSettings();
        assert.strictEqual(settings.maxDepth, 100);
        assert.deepStrictEqual(settings.excludePaths, ['node_modules']);
        
        await configManager.updateAnalysisSettings({
            maxDepth: 200,
            excludePaths: ['node_modules', 'dist']
        });
        assert.ok(mockConfig.update.calledTwice);
    });

    test('handles configuration reset', async () => {
        await configManager.resetToDefaults();
        
        const defaultSettings = configManager.getDefaultSettings();
        assert.ok(mockConfig.update.called);
        assert.ok(mockConfig.update.getCalls().length >= 3); // At least server URL, API key, and notifications
        
        const serverUrl = await configManager.getServerUrl();
        assert.strictEqual(serverUrl, defaultSettings.serverUrl);
    });

    test('persists UI state', async () => {
        const testState = {
            treeViewCollapsed: true,
            selectedAnalysis: 'test-analysis',
            lastRefresh: Date.now()
        };
        
        await configManager.saveUIState(testState);
        assert.ok(mockConfig.update.calledOnce);
        
        mockConfig.get.withArgs('ui.state').returns(testState);
        const savedState = await configManager.getUIState();
        assert.deepStrictEqual(savedState, testState);
    });
}); 