import * as assert from 'assert';
import * as sinon from 'sinon';
import { TreeItemCollapsibleState, WorkspaceConfiguration, workspace } from '../mock/vscode';
import { MetricsProvider, MetricItem } from '../../../providers/metricsProvider';

interface MockWorkspaceConfig extends WorkspaceConfiguration {
    get: sinon.SinonStub;
    update: sinon.SinonStub;
    has: sinon.SinonStub;
    inspect: sinon.SinonStub;
}

suite('MetricsProvider Test Suite', () => {
    let provider: MetricsProvider;
    let mockWorkspace: sinon.SinonStubbedInstance<typeof workspace>;

    setup(() => {
        mockWorkspace = sinon.stub(workspace);
        provider = new MetricsProvider();
    });

    teardown(() => {
        sinon.restore();
    });

    test('MetricItem correctly initializes with icon paths', () => {
        const item = new MetricItem(
            'Test Metric',
            TreeItemCollapsibleState.Collapsed,
            'description'
        );
        assert.strictEqual(item.label, 'Test Metric');
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
        const item = new MetricItem(
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
        assert.strictEqual(items[0].label, 'Code Coverage');
        assert.strictEqual(items[1].label, 'Code Complexity');
    });

    test('getChildren returns detail items for a metric element', async () => {
        const mockConfig = {
            get: sinon.stub().returns('http://localhost:3000'),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const parent = new MetricItem(
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
        assert.strictEqual(items[0].label, 'Error fetching metrics data');
    });

    test('handles special characters in metric names', async () => {
        const mockConfig = {
            get: sinon.stub().returns('http://localhost:3000'),
            update: sinon.stub().resolves(),
            has: sinon.stub().returns(true),
            inspect: sinon.stub()
        } as MockWorkspaceConfig;
        mockWorkspace.getConfiguration.returns(mockConfig);

        const item = new MetricItem(
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
            assert.strictEqual(items[0].label, 'Metrics unavailable');
            assert.ok(items[0].description?.includes('timeout'));
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
            assert.ok(items[0].description?.includes('metrics'));
        });

        test('handles invalid metric data', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://invalid-metrics-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Invalid metrics data');
            assert.ok(items[0].description?.includes('format'));
        });

        test('handles partial metric data', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://partial-data-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.ok(items.length >= 1);
            assert.ok(items.some(item => item.description?.includes('partial')));
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
            assert.strictEqual(items[0].label, 'Authentication required');
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
            assert.ok(items[0].description?.includes('limit'));
        });

        test('handles concurrent metric requests', async () => {
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

        test('handles server disconnection during metrics fetch', async () => {
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
            assert.ok(items[0].description?.includes('metrics'));
        });

        test('handles invalid metric calculations', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://invalid-calc-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'Calculation error');
            assert.ok(items[0].description?.includes('invalid'));
        });

        test('handles empty metrics data', async () => {
            const mockConfig = {
                get: sinon.stub().returns('http://empty-metrics-url'),
                update: sinon.stub().resolves(),
                has: sinon.stub().returns(true),
                inspect: sinon.stub()
            } as MockWorkspaceConfig;
            mockWorkspace.getConfiguration.returns(mockConfig);

            const items = await provider.getChildren();
            assert.strictEqual(items.length, 1);
            assert.strictEqual(items[0].label, 'No metrics available');
            assert.ok(items[0].description?.includes('empty'));
        });
    });
}); 