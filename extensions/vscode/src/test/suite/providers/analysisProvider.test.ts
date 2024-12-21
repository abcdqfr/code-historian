import * as assert from 'assert';
import * as sinon from 'sinon';
import { TreeItemCollapsibleState, WorkspaceConfiguration, workspace } from '../mock/vscode';
import { AnalysisProvider, AnalysisItem } from '../../../providers/analysisProvider';

interface MockWorkspaceConfig extends WorkspaceConfiguration {
    get: sinon.SinonStub;
    update: sinon.SinonStub;
    has: sinon.SinonStub;
    inspect: sinon.SinonStub;
}

suite('AnalysisProvider Test Suite', () => {
    let provider: AnalysisProvider;
    let mockWorkspace: sinon.SinonStubbedInstance<typeof workspace>;

    setup(() => {
        mockWorkspace = sinon.stub(workspace);
        provider = new AnalysisProvider();
    });

    teardown(() => {
        sinon.restore();
    });

    test('AnalysisItem correctly initializes with icon paths', () => {
        const item = new AnalysisItem(
            'Test Analysis',
            TreeItemCollapsibleState.Collapsed,
            'description'
        );
        assert.strictEqual(item.label, 'Test Analysis');
        assert.strictEqual(item.description, 'description');
        assert.strictEqual(item.collapsibleState, TreeItemCollapsibleState.Collapsed);
    });

    test('refresh fires tree data change event', () => {
        const spy = sinon.spy();
        provider.onDidChangeTreeData(spy);
        provider.refresh();
        assert.ok(spy.calledOnce);
    });

    test('getTreeItem returns the same item', () => {
        const item = new AnalysisItem(
            'Test',
            TreeItemCollapsibleState.Collapsed,
            'desc'
        );
        assert.strictEqual(provider.getTreeItem(item), item);
    });

    test('getChildren returns root items when no element is provided', async () => {
        const mockConfig = {
            get: sinon.stub().returns('http://localhost:3000'),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const items = await provider.getChildren();
        assert.strictEqual(items.length, 2);
        assert.strictEqual(items[0].label, 'Code Quality');
        assert.strictEqual(items[1].label, 'Technical Debt');
    });

    test('getChildren returns detail items for an analysis element', async () => {
        const mockConfig = {
            get: sinon.stub().returns('http://localhost:3000'),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const parent = new AnalysisItem(
            'Test Parent',
            TreeItemCollapsibleState.Collapsed,
            'desc'
        );
        const items = await provider.getChildren(parent);
        assert.strictEqual(items.length, 2);
    });

    test('handles missing configuration gracefully', async () => {
        const mockConfig = {
            get: sinon.stub().returns(undefined),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(false),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const items = await provider.getChildren();
        assert.strictEqual(items.length, 1);
        assert.strictEqual(items[0].label, 'Server URL not configured');
    });

    test('handles API errors gracefully', async () => {
        const mockConfig = {
            get: sinon.stub().returns('invalid-url'),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const items = await provider.getChildren();
        assert.strictEqual(items.length, 1);
        assert.strictEqual(items[0].label, 'Error fetching analysis data');
    });

    test('verifies command arguments for items with details', () => {
        const item = new AnalysisItem(
            'Test',
            TreeItemCollapsibleState.Collapsed,
            'desc'
        );
        assert.deepStrictEqual(item.command, {
            command: 'codeHistorian.showAnalysisDetails',
            title: 'Show Analysis Details',
            arguments: [item]
        });
    });

    test('handles special characters in analysis names', async () => {
        const mockConfig = {
            get: sinon.stub().returns('http://localhost:3000'),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const item = new AnalysisItem(
            'Test & Special < > " \' Characters',
            TreeItemCollapsibleState.Collapsed,
            'desc'
        );
        assert.ok(item.label.includes('&'));
        assert.ok(item.label.includes('<'));
        assert.ok(item.label.includes('>'));
    });

    suite('Error Propagation Tests', () => {
        test('handles network timeout gracefully', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://timeout-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Connection timeout');
            assert.ok(items[0].description?.includes('retry'));
        });

        test('handles server error responses', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://error-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Server error');
            assert.ok(items[0].description?.includes('status'));
        });

        test('handles invalid response data', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://invalid-data-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Invalid data received');
            assert.ok(items[0].description?.includes('format'));
        });

        test('handles authentication errors', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://auth-error-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Authentication failed');
            assert.ok(items[0].description?.includes('credentials'));
        });

        test('handles rate limiting', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://rate-limited-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Rate limit exceeded');
            assert.ok(items[0].description?.includes('try again'));
        });

        test('handles concurrent request errors', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://localhost:3000'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            // Make multiple concurrent requests
            const promises = Array(5).fill(null).map(() => provider.getChildren());
            const results = await Promise.all(promises);

            // All requests should complete without throwing
            results.forEach(items => {
                assert.ok(Array.isArray(items));
                assert.ok(items.length > 0);
            });
        });

        test('handles server disconnection during operation', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://disconnect-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Connection lost');
            assert.ok(items[0].description?.includes('reconnect'));
        });

        test('handles malformed URLs gracefully', async () => {
            const mockConfig = {
                get: sinon.stub().returns('not-a-valid-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Invalid server URL');
            assert.ok(items[0].description?.includes('configuration'));
        });

        test('handles empty response data', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://empty-response-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'No data available');
            assert.ok(items[0].description?.includes('empty'));
        });
    });
}); 