import * as vscode from 'vscode';
import * as path from 'path';
import axios from 'axios';

export class DashboardPanel {
    public static currentPanel: DashboardPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this._panel = panel;

        // Set the webview's initial html content
        this._panel.webview.html = this._getHtmlForWebview();

        // Listen for when the panel is disposed
        // This happens when the user closes the panel or when the panel is closed programmatically
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        // Handle messages from the webview
        this._panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'refresh':
                        this._updateDashboard();
                        return;
                }
            },
            null,
            this._disposables
        );

        // Initial update
        this._updateDashboard();
    }

    public static createOrShow(extensionUri: vscode.Uri) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        // If we already have a panel, show it
        if (DashboardPanel.currentPanel) {
            DashboardPanel.currentPanel._panel.reveal(column);
            return;
        }

        // Otherwise, create a new panel
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

    public dispose() {
        DashboardPanel.currentPanel = undefined;

        // Clean up our resources
        this._panel.dispose();

        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    public async updateProgress(progress: number) {
        if (this._panel.visible) {
            await this._panel.webview.postMessage({ 
                command: 'updateProgress',
                progress: progress
            });
        }
    }

    private async _updateDashboard() {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const [metricsResponse, historyResponse] = await Promise.all([
                axios.get(`${serverUrl}/api/metrics/project`, {
                    headers: { 'X-API-Key': apiKey }
                }),
                axios.get(`${serverUrl}/api/history/summary`, {
                    headers: { 'X-API-Key': apiKey }
                })
            ]);

            await this._panel.webview.postMessage({
                command: 'updateDashboard',
                metrics: metricsResponse.data,
                history: historyResponse.data
            });

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to update dashboard: ${error.message}`);
        }
    }

    private _getHtmlForWebview() {
        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
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
                    .metric {
                        font-size: 24px;
                        font-weight: bold;
                        color: var(--vscode-textLink-foreground);
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
                        progressBar.style.width = `${progress * 100}%`;
                    }

                    function updateDashboard(metrics, history) {
                        // Update repository metrics
                        document.getElementById('repoMetrics').innerHTML = `
                            <div class="metric">${metrics.totalFiles}</div>
                            <div>Total Files</div>
                            <div class="metric">${metrics.totalCommits}</div>
                            <div>Total Commits</div>
                            <div class="metric">${metrics.totalAuthors}</div>
                            <div>Total Authors</div>
                        `;

                        // Update charts
                        updateChurnChart(history.churn);
                        updateImpactChart(history.impact);
                    }

                    function updateChurnChart(data) {
                        if (churnChart) {
                            churnChart.destroy();
                        }

                        const ctx = document.getElementById('churnChart').getContext('2d');
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

                    function updateImpactChart(data) {
                        if (impactChart) {
                            impactChart.destroy();
                        }

                        const ctx = document.getElementById('impactChart').getContext('2d');
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
                </script>
            </body>
            </html>
        `;
    }
} 