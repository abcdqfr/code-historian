import * as assert from 'assert';
import * as sinon from 'sinon';
import { workspace, WorkspaceConfiguration } from '../mock/vscode';
import { ConfigurationManager } from '../../configuration/configManager';
import { HistorianError } from '../../error/historianError';

interface MockWorkspaceConfig extends WorkspaceConfiguration {
    get: sinon.SinonStub;
    update: sinon.SinonStub;
    has: sinon.SinonStub;
    inspect: sinon.SinonStub;
}

suite('State Restoration Test Suite', () => {
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

    test('restores UI state after crash', async () => {
        const originalState = {
            treeViewCollapsed: true,
            selectedAnalysis: 'test-analysis',
            lastRefresh: Date.now()
        };
        
        // Save the original state
        await configManager.saveUIState(originalState);
        assert.ok(mockConfig.update.calledOnce);
        
        // Simulate a crash by clearing the in-memory state
        await configManager.resetToDefaults();
        
        // Mock the stored state
        mockConfig.get.withArgs('ui.state').returns(originalState);
        
        // Restore the state
        const restoredState = await configManager.getUIState();
        assert.deepStrictEqual(restoredState, originalState);
    });

    test('restores settings after network failure', async () => {
        const testSettings = {
            serverUrl: 'http://test-server:3000',
            notifications: {
                enabled: true,
                showInStatusBar: true
            },
            analysis: {
                maxDepth: 100,
                excludePaths: ['node_modules']
            }
        };
        
        // Save the settings
        await configManager.setServerUrl(testSettings.serverUrl);
        await configManager.updateNotificationSettings(testSettings.notifications);
        await configManager.updateAnalysisSettings(testSettings.analysis);
        
        // Simulate network failure and recovery
        mockConfig.get.withArgs('serverUrl').returns(testSettings.serverUrl);
        mockConfig.get.withArgs('notifications').returns(testSettings.notifications);
        mockConfig.get.withArgs('analysis').returns(testSettings.analysis);
        
        // Verify settings are restored
        const serverUrl = await configManager.getServerUrl();
        assert.strictEqual(serverUrl, testSettings.serverUrl);
        
        const notifications = await configManager.getNotificationSettings();
        assert.deepStrictEqual(notifications, testSettings.notifications);
        
        const analysis = await configManager.getAnalysisSettings();
        assert.deepStrictEqual(analysis, testSettings.analysis);
    });

    test('restores partial state with defaults', async () => {
        const partialState = {
            treeViewCollapsed: true
        };
        
        // Save partial state
        mockConfig.get.withArgs('ui.state').returns(partialState);
        
        // Get state with defaults for missing values
        const restoredState = await configManager.getUIState();
        assert.strictEqual(restoredState.treeViewCollapsed, true);
        assert.strictEqual(restoredState.selectedAnalysis, null);
        assert.ok(restoredState.lastRefresh !== undefined);
    });

    test('handles corrupted state gracefully', async () => {
        const corruptedState = 'invalid-json-data';
        mockConfig.get.withArgs('ui.state').returns(corruptedState);
        
        // Should return default state when corrupted
        const restoredState = await configManager.getUIState();
        assert.strictEqual(restoredState.treeViewCollapsed, false);
        assert.strictEqual(restoredState.selectedAnalysis, null);
        assert.ok(restoredState.lastRefresh !== undefined);
    });

    test('restores state with version migration', async () => {
        const legacyState = {
            version: '1.0.0',
            treeCollapsed: true, // Old format
            analysis: 'test' // Old format
        };
        
        mockConfig.get.withArgs('ui.state').returns(legacyState);
        
        // Should migrate to new format
        const restoredState = await configManager.getUIState();
        assert.strictEqual(restoredState.treeViewCollapsed, true);
        assert.strictEqual(restoredState.selectedAnalysis, 'test');
    });

    test('preserves state during concurrent operations', async () => {
        const state1 = {
            treeViewCollapsed: true,
            selectedAnalysis: 'analysis1',
            lastRefresh: Date.now()
        };
        
        const state2 = {
            treeViewCollapsed: false,
            selectedAnalysis: 'analysis2',
            lastRefresh: Date.now()
        };
        
        // Simulate concurrent saves
        const save1 = configManager.saveUIState(state1);
        const save2 = configManager.saveUIState(state2);
        
        await Promise.all([save1, save2]);
        
        // Last save should win
        mockConfig.get.withArgs('ui.state').returns(state2);
        const finalState = await configManager.getUIState();
        assert.deepStrictEqual(finalState, state2);
    });

    test('recovers from invalid settings', async () => {
        const invalidSettings = {
            serverUrl: 'invalid-url',
            notifications: null,
            analysis: undefined
        };
        
        mockConfig.get.withArgs('serverUrl').returns(invalidSettings.serverUrl);
        mockConfig.get.withArgs('notifications').returns(invalidSettings.notifications);
        mockConfig.get.withArgs('analysis').returns(invalidSettings.analysis);
        
        // Should recover with default values
        const serverUrl = await configManager.getServerUrl();
        assert.strictEqual(serverUrl, 'http://localhost:3000');
        
        const notifications = await configManager.getNotificationSettings();
        assert.deepStrictEqual(notifications, {
            enabled: true,
            showInStatusBar: true
        });
        
        const analysis = await configManager.getAnalysisSettings();
        assert.deepStrictEqual(analysis, {
            maxDepth: 100,
            excludePaths: ['node_modules']
        });
    });
}); 