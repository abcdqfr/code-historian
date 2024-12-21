import * as vscode from 'vscode';
import { performance } from 'perf_hooks';

interface ChartData {
    labels: string[];
    values: number[];
}

interface DataItem {
    id: string;
    name: string;
    value: number;
    children?: DataItem[];
}

export class DashboardPanel {
    private static currentPanel: DashboardPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];
    private _interactionTimes: number[] = [];
    private _loadedItems: DataItem[] = [];
    private _isLoading: boolean = false;

    constructor() {
        this._panel = vscode.window.createWebviewPanel(
            'codeHistorianDashboard',
            'Code Historian Dashboard',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
    }

    public async render(): Promise<void> {
        const startTime = performance.now();
        
        // Basic panel setup
        this._panel.webview.html = this._getHtmlForWebview();
        
        // Initialize data
        await this._initializeData();
        
        const renderTime = performance.now() - startTime;
        console.log(`Dashboard render took ${renderTime}ms`);
        
        return Promise.resolve();
    }

    public async loadItems(items: DataItem[], batchSize: number): Promise<void> {
        const startTime = performance.now();
        
        this._isLoading = true;
        const initialBatch = items.slice(0, batchSize);
        
        await this._renderItems(initialBatch);
        this._loadedItems = initialBatch;
        
        this._isLoading = false;
        console.log(`Initial load took ${performance.now() - startTime}ms`);
    }

    public async loadMoreItems(count: number): Promise<void> {
        if (this._isLoading) return;
        
        const startTime = performance.now();
        this._isLoading = true;
        
        const currentCount = this._loadedItems.length;
        const newItems = this._loadedItems.slice(currentCount, currentCount + count);
        
        await this._renderItems(newItems);
        this._loadedItems = [...this._loadedItems, ...newItems];
        
        this._isLoading = false;
        console.log(`Loading more items took ${performance.now() - startTime}ms`);
    }

    public async renderChart(chartId: string, data: ChartData): Promise<void> {
        const startTime = performance.now();
        
        await this._panel.webview.postMessage({
            command: 'renderChart',
            chartId,
            data
        });
        
        console.log(`Chart render took ${performance.now() - startTime}ms`);
    }

    public async handleUserInteraction(action: string): Promise<void> {
        const startTime = performance.now();
        
        // Process the interaction
        await this._processInteraction(action);
        
        const endTime = performance.now();
        this._interactionTimes.push(endTime - startTime);
    }

    public getInteractionTimes(): number[] {
        return this._interactionTimes;
    }

    public async loadLargeDataset(): Promise<void> {
        const startTime = performance.now();
        
        // Simulate loading a large dataset
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        console.log(`Large dataset load took ${performance.now() - startTime}ms`);
    }

    public async visualizeLargeDataset(data: DataItem[]): Promise<void> {
        const startTime = performance.now();
        
        // Process data in chunks
        const chunkSize = 1000;
        for (let i = 0; i < data.length; i += chunkSize) {
            const chunk = data.slice(i, i + chunkSize);
            await this._processDataChunk(chunk);
            
            // Allow UI updates between chunks
            await new Promise(resolve => setTimeout(resolve, 0));
        }
        
        console.log(`Large dataset visualization took ${performance.now() - startTime}ms`);
    }

    private async _initializeData(): Promise<void> {
        // Initialize dashboard data
        await this._panel.webview.postMessage({
            command: 'initialize'
        });
    }

    private async _renderItems(items: DataItem[]): Promise<void> {
        await this._panel.webview.postMessage({
            command: 'renderItems',
            items
        });
    }

    private async _processInteraction(action: string): Promise<void> {
        await this._panel.webview.postMessage({
            command: 'processInteraction',
            action
        });
    }

    private async _processDataChunk(chunk: DataItem[]): Promise<void> {
        await this._panel.webview.postMessage({
            command: 'processChunk',
            chunk
        });
    }

    private _getHtmlForWebview(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Code Historian Dashboard</title>
    <style>
        body {
            padding: 0;
            margin: 0;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
        }
        .dashboard {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            padding: 1rem;
        }
        .chart-container {
            height: 300px;
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-panel-border);
            border-radius: 4px;
            padding: 1rem;
        }
        .metrics-container {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 1rem;
        }
        .metric-card {
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-panel-border);
            border-radius: 4px;
            padding: 1rem;
            text-align: center;
        }
        .loading-indicator {
            position: fixed;
            top: 1rem;
            right: 1rem;
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            padding: 0.5rem 1rem;
            border-radius: 4px;
            display: none;
        }
        .loading .loading-indicator {
            display: block;
        }
    </style>
</head>
<body>
    <div class="dashboard">
        <div class="metrics-container" id="metrics"></div>
        <div class="chart-container">
            <canvas id="mainChart"></canvas>
        </div>
    </div>
    <div class="loading-indicator">Loading...</div>
    <script>
        const vscode = acquireVsCodeApi();
        let mainChart = null;

        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.command) {
                case 'renderItems':
                    renderItems(message.items);
                    break;
                case 'renderChart':
                    renderChart(message.chartId, message.data);
                    break;
                case 'processInteraction':
                    processInteraction(message.action);
                    break;
                case 'processChunk':
                    processDataChunk(message.chunk);
                    break;
            }
        });

        function renderItems(items) {
            const container = document.getElementById('metrics');
            items.forEach(item => {
                const card = document.createElement('div');
                card.className = 'metric-card';
                card.innerHTML = \`
                    <h3>\${item.name}</h3>
                    <div class="value">\${item.value}</div>
                \`;
                container.appendChild(card);
            });
        }

        function renderChart(chartId, data) {
            const ctx = document.getElementById(chartId).getContext('2d');
            if (mainChart) {
                mainChart.destroy();
            }
            mainChart = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: data.labels,
                    datasets: [{
                        data: data.values,
                        borderColor: '#4CAF50',
                        tension: 0.4
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false
                }
            });
        }

        function processInteraction(action) {
            console.log('Processing interaction:', action);
            vscode.postMessage({ command: 'interactionProcessed', action });
        }

        function processDataChunk(chunk) {
            console.log('Processing chunk of size:', chunk.length);
            vscode.postMessage({ command: 'chunkProcessed' });
        }
    </script>
</body>
</html>`;
    }

    public dispose() {
        DashboardPanel.currentPanel = undefined;
        this._panel.dispose();
        while (this._disposables.length) {
            const disposable = this._disposables.pop();
            if (disposable) {
                disposable.dispose();
            }
        }
    }
} 