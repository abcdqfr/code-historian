import * as vscode from 'vscode';
import { AnalysisProvider } from './providers/analysisProvider';
import { MetricsProvider } from './providers/metricsProvider';
import { DashboardPanel } from './panels/dashboardPanel';
import { WebSocketClient } from './utils/websocket';
import axios from 'axios';

let dashboardPanel: DashboardPanel | undefined;
let webSocketClient: WebSocketClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
    console.log('Code Historian extension is now active');

    // Initialize providers
    const analysisProvider = new AnalysisProvider();
    const metricsProvider = new MetricsProvider();

    // Register tree view providers
    vscode.window.registerTreeDataProvider('codeHistorianExplorer', analysisProvider);
    vscode.window.registerTreeDataProvider('codeHistorianMetrics', metricsProvider);

    // Register commands
    let startAnalysis = vscode.commands.registerCommand('codeHistorian.startAnalysis', async () => {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const workspaceFolders = vscode.workspace.workspaceFolders;
            if (!workspaceFolders) {
                throw new Error('No workspace folder is open');
            }

            const response = await axios.post(`${serverUrl}/api/analysis/start`, {
                projectPath: workspaceFolders[0].uri.fsPath
            }, {
                headers: {
                    'X-API-Key': apiKey
                }
            });

            // Connect WebSocket for real-time updates
            webSocketClient = new WebSocketClient(
                `${serverUrl.replace('http', 'ws')}/ws/analysis/${response.data.id}`,
                (progress: number) => {
                    vscode.window.setStatusBarMessage(`Analysis Progress: ${Math.round(progress * 100)}%`);
                    if (dashboardPanel) {
                        dashboardPanel.updateProgress(progress);
                    }
                }
            );

            vscode.window.showInformationMessage('Code history analysis started');

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to start analysis: ${error.message}`);
        }
    });

    let showDashboard = vscode.commands.registerCommand('codeHistorian.showDashboard', () => {
        if (dashboardPanel) {
            dashboardPanel.reveal();
        } else {
            dashboardPanel = new DashboardPanel(context.extensionUri);
        }
    });

    let showHistory = vscode.commands.registerCommand('codeHistorian.showHistory', async (uri: vscode.Uri) => {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const response = await axios.get(`${serverUrl}/api/history/file`, {
                params: {
                    path: uri.fsPath
                },
                headers: {
                    'X-API-Key': apiKey
                }
            });

            // Create and show history panel
            const panel = vscode.window.createWebviewPanel(
                'codeHistorian.history',
                'File History',
                vscode.ViewColumn.Two,
                {
                    enableScripts: true
                }
            );

            panel.webview.html = getHistoryHtml(response.data);

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to load file history: ${error.message}`);
        }
    });

    context.subscriptions.push(startAnalysis, showDashboard, showHistory);
}

export function deactivate() {
    if (webSocketClient) {
        webSocketClient.disconnect();
    }
}

function getHistoryHtml(history: any): string {
    return `
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>File History</title>
            <style>
                body { font-family: var(--vscode-font-family); }
                .history-item {
                    margin: 10px 0;
                    padding: 10px;
                    border: 1px solid var(--vscode-panel-border);
                }
                .timestamp { color: var(--vscode-descriptionForeground); }
                .author { font-weight: bold; }
                .impact { float: right; }
            </style>
        </head>
        <body>
            <div id="history">
                ${history.changes.map((change: any) => `
                    <div class="history-item">
                        <div class="timestamp">${change.timestamp}</div>
                        <div class="author">${change.author}</div>
                        <div class="message">${change.message}</div>
                        <div class="impact">Impact: ${Math.round(change.impactScore * 100)}%</div>
                    </div>
                `).join('')}
            </div>
        </body>
        </html>
    `;
} 