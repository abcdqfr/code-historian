import * as vscode from 'vscode';
import * as path from 'path';
import axios, { AxiosError } from 'axios';

interface DashboardMetrics {
    totalFiles: number;
    totalCommits: number;
    totalAuthors: number;
}

interface HistoryData {
    churn: {
        labels: string[];
        values: number[];
    };
    impact: {
        labels: string[];
        values: number[];
    };
}

interface WebviewMessage {
    command: string;
    progress?: number;
    metrics?: DashboardMetrics;
    history?: HistoryData;
}

interface ErrorResponse {
    message: string;
    details?: string;
}

export class DashboardPanel {
    public static currentPanel: DashboardPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this._panel = panel;
        this._panel.webview.html = this._getHtmlForWebview(extensionUri);
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        this._panel.webview.onDidReceiveMessage(
            (message: WebviewMessage) => {
                switch (message.command) {
                    case 'refresh':
                        void this._updateDashboard();
                        return;
                }
            },
            null,
            this._disposables
        );

        void this._updateDashboard();
    }

    public static createOrShow(extensionUri: vscode.Uri): void {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        if (DashboardPanel.currentPanel) {
            DashboardPanel.currentPanel._panel.reveal(column);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'codeHistorian.dashboard',
            'Code Historian Dashboard',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [
                    vscode.Uri.joinPath(extensionUri, 'media')
                ]
            }
        );

        DashboardPanel.currentPanel = new DashboardPanel(panel, extensionUri);
    }

    public dispose(): void {
        DashboardPanel.currentPanel = undefined;
        this._panel.dispose();

        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    public async updateProgress(progress: number): Promise<void> {
        if (this._panel.visible) {
            await this._panel.webview.postMessage({ 
                command: 'updateProgress',
                progress: progress
            } as WebviewMessage);
        }
    }

    private async _updateDashboard(): Promise<void> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const [metricsResponse, historyResponse] = await Promise.all([
                axios.get<DashboardMetrics>(`${serverUrl}/api/metrics/project`, {
                    headers: { 'X-API-Key': apiKey }
                }),
                axios.get<HistoryData>(`${serverUrl}/api/history/summary`, {
                    headers: { 'X-API-Key': apiKey }
                })
            ]);

            await this._panel.webview.postMessage({
                command: 'updateDashboard',
                metrics: metricsResponse.data,
                history: historyResponse.data
            } as WebviewMessage);

        } catch (error) {
            let errorMessage: string;
            
            if (error instanceof AxiosError && error.response?.data) {
                const errorData = error.response.data as ErrorResponse;
                errorMessage = errorData.message || errorData.details || error.message;
            } else if (error instanceof Error) {
                errorMessage = error.message;
            } else {
                errorMessage = 'An unknown error occurred';
            }
            
            void vscode.window.showErrorMessage(`Failed to update dashboard: ${errorMessage}`);
        }
    }

    private _getHtmlForWebview(extensionUri: vscode.Uri): string {
        // Get the local path to script and css files
        const scriptUri = vscode.Uri.joinPath(extensionUri, 'media', 'main.js');
        const styleUri = vscode.Uri.joinPath(extensionUri, 'media', 'style.css');

        // And get the special URI to use with the webview
        const scriptWebviewUri = this._panel.webview.asWebviewUri(scriptUri);
        const styleWebviewUri = this._panel.webview.asWebviewUri(styleUri);

        return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline' ${styleWebviewUri}; script-src 'unsafe-inline' ${scriptWebviewUri} https://cdn.jsdelivr.net;">
                <title>Code Historian Dashboard</title>
                <style>
                    body {
                        font-family: var(--vscode-font-family);
                        color: var(--vscode-foreground);
                        background-color: var(--vscode-editor-background);
                        padding: 20px;
                    }
                    .dashboard-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                        gap: 20px;
                        margin-bottom: 20px;
                    }
                    .card {
                        background-color: var(--vscode-editor-background);
                        border: 1px solid var(--vscode-panel-border);
                        border-radius: 4px;
                        padding: 15px;
                    }
                    .card h2 {
                        margin-top: 0;
                        color: var(--vscode-foreground);
                    }
                    .metric-item {
                        margin-bottom: 15px;
                    }
                    .metric {
                        font-size: 24px;
                        font-weight: bold;
                        color: var(--vscode-textLink-foreground);
                    }
                    .metric-label {
                        color: var(--vscode-descriptionForeground);
                        font-size: 14px;
                    }
                    .chart-container {
                        height: 300px;
                        margin: 20px 0;
                    }
                    .progress-bar {
                        width: 100%;
                        height: 20px;
                        background-color: var(--vscode-progressBar-background);
                        border-radius: 10px;
                        overflow: hidden;
                    }
                    .progress-bar-fill {
                        height: 100%;
                        background-color: var(--vscode-progressBar-foreground);
                        transition: width 0.3s ease-in-out;
                    }
                </style>
                <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
            </head>
            <body>
                <div class="dashboard-grid">
                    <div class="card">
                        <h2>Analysis Progress</h2>
                        <div class="progress-bar">
                            <div class="progress-bar-fill" style="width: 0%"></div>
                        </div>
                    </div>
                    <div class="card">
                        <h2>Repository Overview</h2>
                        <div id="repoMetrics"></div>
                    </div>
                    <div class="card">
                        <h2>Code Churn</h2>
                        <div class="chart-container">
                            <canvas id="churnChart"></canvas>
                        </div>
                    </div>
                    <div class="card">
                        <h2>Impact Analysis</h2>
                        <div class="chart-container">
                            <canvas id="impactChart"></canvas>
                        </div>
                    </div>
                </div>
                <script>
                    const vscode = acquireVsCodeApi();
                    let churnChart, impactChart;

                    window.addEventListener('message', event => {
                        const message = event.data;
                        switch (message.command) {
                            case 'updateProgress':
                                updateProgress(message.progress);
                                break;
                            case 'updateDashboard':
                                updateDashboard(message.metrics, message.history);
                                break;
                        }
                    });

                    function updateProgress(progress) {
                        const progressBar = document.querySelector('.progress-bar-fill');
                        if (progressBar instanceof HTMLElement) {
                            progressBar.style.width = \`\${Math.round(progress * 100)}%\`;
                        }
                    }

                    function updateDashboard(metrics, history) {
                        const repoMetrics = document.getElementById('repoMetrics');
                        if (repoMetrics instanceof HTMLElement) {
                            repoMetrics.innerHTML = \`
                                <div class="metric-item">
                                    <div class="metric">\${metrics.totalFiles}</div>
                                    <div class="metric-label">Total Files</div>
                                </div>
                                <div class="metric-item">
                                    <div class="metric">\${metrics.totalCommits}</div>
                                    <div class="metric-label">Total Commits</div>
                                </div>
                                <div class="metric-item">
                                    <div class="metric">\${metrics.totalAuthors}</div>
                                    <div class="metric-label">Total Authors</div>
                                </div>
                            \`;
                        }

                        updateChurnChart(history.churn);
                        updateImpactChart(history.impact);
                    }

                    function updateChurnChart(data) {
                        if (churnChart) {
                            churnChart.destroy();
                        }

                        const canvas = document.getElementById('churnChart');
                        if (canvas instanceof HTMLCanvasElement) {
                            const ctx = canvas.getContext('2d');
                            if (ctx) {
                                churnChart = new Chart(ctx, {
                                    type: 'line',
                                    data: {
                                        labels: data.labels,
                                        datasets: [{
                                            label: 'Code Churn',
                                            data: data.values,
                                            borderColor: getComputedStyle(document.body).getPropertyValue('--vscode-textLink-foreground'),
                                            tension: 0.4
                                        }]
                                    },
                                    options: {
                                        responsive: true,
                                        maintainAspectRatio: false
                                    }
                                });
                            }
                        }
                    }

                    function updateImpactChart(data) {
                        if (impactChart) {
                            impactChart.destroy();
                        }

                        const canvas = document.getElementById('impactChart');
                        if (canvas instanceof HTMLCanvasElement) {
                            const ctx = canvas.getContext('2d');
                            if (ctx) {
                                impactChart = new Chart(ctx, {
                                    type: 'bar',
                                    data: {
                                        labels: data.labels,
                                        datasets: [{
                                            label: 'Impact Score',
                                            data: data.values,
                                            backgroundColor: getComputedStyle(document.body).getPropertyValue('--vscode-textLink-foreground')
                                        }]
                                    },
                                    options: {
                                        responsive: true,
                                        maintainAspectRatio: false
                                    }
                                });
                            }
                        }
                    }
                </script>
            </body>
            </html>`;
    }
} 