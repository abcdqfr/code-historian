import * as vscode from 'vscode';
import * as path from 'path';

export class DashboardPanel {
    public static currentPanel: DashboardPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];

    private constructor(extensionUri: vscode.Uri) {
        this._panel = vscode.window.createWebviewPanel(
            'code-historian-dashboard',
            'Code Historian Dashboard',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [
                    vscode.Uri.joinPath(extensionUri, 'media'),
                    vscode.Uri.joinPath(extensionUri, 'resources')
                ]
            }
        );

        this._panel.webview.html = this._getHtmlContent(this._panel.webview, extensionUri);

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        this._panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'refresh':
                        vscode.commands.executeCommand('code-historian.analyze');
                        break;
                    case 'error':
                        vscode.window.showErrorMessage(message.text);
                        break;
                }
            },
            null,
            this._disposables
        );

        DashboardPanel.currentPanel = this;
    }

    public static createOrShow(extensionUri: vscode.Uri) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        if (DashboardPanel.currentPanel) {
            DashboardPanel.currentPanel._panel.reveal(column);
            return;
        }

        DashboardPanel.currentPanel = new DashboardPanel(extensionUri);
    }

    public update(data: any) {
        this._panel.webview.postMessage({ type: 'update', data });
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

    private _getHtmlContent(webview: vscode.Webview, extensionUri: vscode.Uri): string {
        const scriptUri = webview.asWebviewUri(
            vscode.Uri.joinPath(extensionUri, 'media', 'dashboard.js')
        );
        const styleUri = webview.asWebviewUri(
            vscode.Uri.joinPath(extensionUri, 'media', 'dashboard.css')
        );
        const chartJsUri = webview.asWebviewUri(
            vscode.Uri.joinPath(extensionUri, 'node_modules', 'chart.js', 'dist', 'chart.min.js')
        );
        const momentUri = webview.asWebviewUri(
            vscode.Uri.joinPath(extensionUri, 'node_modules', 'moment', 'min', 'moment.min.js')
        );

        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Code Historian Dashboard</title>
            <link href="${styleUri}" rel="stylesheet">
            <script src="${chartJsUri}"></script>
            <script src="${momentUri}"></script>
        </head>
        <body>
            <div class="container">
                <header>
                    <h1>Code Evolution Dashboard</h1>
                    <button id="refreshBtn" class="refresh-button">
                        <span class="codicon codicon-refresh"></span>
                        Refresh
                    </button>
                </header>

                <div class="metrics-grid">
                    <div class="metric-card">
                        <div class="metric-value" id="activeAnalyses">0</div>
                        <div class="metric-label">Active Analyses</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="teamMembers">0</div>
                        <div class="metric-label">Team Members</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="totalProjects">0</div>
                        <div class="metric-label">Total Projects</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="avgImpactScore">0.0</div>
                        <div class="metric-label">Average Impact Score</div>
                    </div>
                </div>

                <div class="chart-grid">
                    <div class="chart-container">
                        <h2>Analysis Progress</h2>
                        <canvas id="progressChart"></canvas>
                    </div>
                    <div class="chart-container">
                        <h2>Team Activity</h2>
                        <canvas id="activityChart"></canvas>
                    </div>
                    <div class="chart-container">
                        <h2>Project Comparison</h2>
                        <canvas id="comparisonChart"></canvas>
                    </div>
                </div>

                <div class="activity-feed">
                    <h2>Recent Activity</h2>
                    <div id="activityList" class="activity-list">
                        <!-- Activity items will be inserted here -->
                    </div>
                </div>
            </div>

            <script src="${scriptUri}"></script>
            <script>
                const vscode = acquireVsCodeApi();
                document.getElementById('refreshBtn').addEventListener('click', () => {
                    vscode.postMessage({ command: 'refresh' });
                });
            </script>
        </body>
        </html>`;
    }
} 