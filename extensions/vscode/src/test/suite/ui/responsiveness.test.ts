import * as assert from 'assert';
import * as sinon from 'sinon';
import { workspace, TreeItemCollapsibleState } from '../../mock/vscode';
import { AnalysisProvider } from '../../../providers/analysisProvider';
import { MetricsProvider } from '../../../providers/metricsProvider';
import { DashboardPanel } from '../../../panels/dashboardPanel';
import { ConfigurationManager } from '../../../configuration/configManager';
import { performance } from 'perf_hooks';

suite('UI Responsiveness Test Suite', () => {
    let analysisProvider: AnalysisProvider;
    let metricsProvider: MetricsProvider;
    let configManager: ConfigurationManager;
    let clock: sinon.SinonFakeTimers;

    setup(() => {
        analysisProvider = new AnalysisProvider();
        metricsProvider = new MetricsProvider();
        configManager = new ConfigurationManager();
        clock = sinon.useFakeTimers();
    });

    teardown(() => {
        clock.restore();
        sinon.restore();
    });

    test('tree view updates within performance budget', async () => {
        const startTime = performance.now();
        await analysisProvider.getChildren();
        const endTime = performance.now();
        
        const updateTime = endTime - startTime;
        assert.ok(updateTime < 100, `Tree view update took ${updateTime}ms, should be under 100ms`);
    });

    test('handles rapid tree view expansion/collapse', async () => {
        const operations = [];
        for (let i = 0; i < 10; i++) {
            operations.push(analysisProvider.getChildren());
        }
        
        const startTime = performance.now();
        await Promise.all(operations);
        const endTime = performance.now();
        
        const avgTime = (endTime - startTime) / operations.length;
        assert.ok(avgTime < 50, `Average operation time ${avgTime}ms exceeds 50ms budget`);
    });

    test('metrics update performance', async () => {
        const updateTimes: number[] = [];
        
        for (let i = 0; i < 5; i++) {
            const start = performance.now();
            await metricsProvider.getChildren();
            updateTimes.push(performance.now() - start);
        }
        
        const avgUpdateTime = updateTimes.reduce((a, b) => a + b) / updateTimes.length;
        assert.ok(avgUpdateTime < 150, `Average metrics update time ${avgUpdateTime}ms exceeds 150ms budget`);
    });

    test('dashboard rendering performance', async () => {
        const panel = new DashboardPanel();
        
        const startTime = performance.now();
        await panel.render();
        const renderTime = performance.now() - startTime;
        
        assert.ok(renderTime < 500, `Initial dashboard render took ${renderTime}ms, should be under 500ms`);
    });

    test('handles rapid configuration changes', async () => {
        const changes = [];
        for (let i = 0; i < 10; i++) {
            changes.push(
                configManager.updateAnalysisSettings({
                    maxDepth: 100 + i,
                    excludePaths: [`path${i}`]
                })
            );
        }
        
        const startTime = performance.now();
        await Promise.all(changes);
        const totalTime = performance.now() - startTime;
        
        assert.ok(totalTime < 200, `Configuration updates took ${totalTime}ms, should be under 200ms`);
    });

    test('lazy loading performance', async () => {
        const panel = new DashboardPanel();
        const items = Array(1000).fill(null).map((_, i) => ({
            id: `item${i}`,
            name: `Item ${i}`,
            value: i
        }));
        
        const startTime = performance.now();
        await panel.loadItems(items, 50); // Load 50 items initially
        const initialLoadTime = performance.now() - startTime;
        
        assert.ok(initialLoadTime < 100, `Initial lazy load took ${initialLoadTime}ms, should be under 100ms`);
        
        const moreStartTime = performance.now();
        await panel.loadMoreItems(50); // Load 50 more items
        const moreLoadTime = performance.now() - moreStartTime;
        
        assert.ok(moreLoadTime < 100, `Loading more items took ${moreLoadTime}ms, should be under 100ms`);
    });

    test('chart rendering performance', async () => {
        const panel = new DashboardPanel();
        const data = {
            labels: Array(100).fill(null).map((_, i) => `Label ${i}`),
            values: Array(100).fill(null).map(() => Math.random() * 100)
        };
        
        const startTime = performance.now();
        await panel.renderChart('testChart', data);
        const renderTime = performance.now() - startTime;
        
        assert.ok(renderTime < 300, `Chart rendering took ${renderTime}ms, should be under 300ms`);
    });

    test('handles concurrent UI operations', async () => {
        const operations = [
            analysisProvider.getChildren(),
            metricsProvider.getChildren(),
            configManager.getAnalysisSettings(),
            new DashboardPanel().render()
        ];
        
        const startTime = performance.now();
        await Promise.all(operations);
        const totalTime = performance.now() - startTime;
        
        assert.ok(totalTime < 1000, `Concurrent operations took ${totalTime}ms, should be under 1000ms`);
    });

    test('responsive to user interactions during load', async () => {
        const panel = new DashboardPanel();
        const loadPromise = panel.loadLargeDataset();
        
        // Simulate user interactions during load
        const interactions = [];
        for (let i = 0; i < 5; i++) {
            interactions.push(panel.handleUserInteraction(`action${i}`));
            clock.tick(100); // Simulate time passing
        }
        
        await Promise.all([loadPromise, ...interactions]);
        
        // Verify all interactions were handled
        const interactionTimes = panel.getInteractionTimes();
        interactionTimes.forEach(time => {
            assert.ok(time < 50, `Interaction took ${time}ms, should be under 50ms`);
        });
    });

    test('maintains performance with large datasets', async () => {
        const largeDataset = Array(10000).fill(null).map((_, i) => ({
            id: `item${i}`,
            name: `Item ${i}`,
            value: Math.random() * 1000,
            children: Array(5).fill(null).map((_, j) => ({
                id: `item${i}-${j}`,
                name: `Subitem ${j}`,
                value: Math.random() * 100
            }))
        }));
        
        const startTime = performance.now();
        
        // Test tree view performance
        await analysisProvider.handleLargeDataset(largeDataset);
        
        // Test metrics calculation
        await metricsProvider.processDataset(largeDataset);
        
        // Test visualization rendering
        const panel = new DashboardPanel();
        await panel.visualizeLargeDataset(largeDataset);
        
        const totalTime = performance.now() - startTime;
        
        assert.ok(totalTime < 3000, `Large dataset processing took ${totalTime}ms, should be under 3000ms`);
    });
}); 