import * as vscode from 'vscode';
import axios from 'axios';
import WebSocket from 'ws';
import { AnalysisProvider } from './providers/analysisProvider';
import { MetricsProvider } from './providers/metricsProvider';
import { TeamProvider } from './providers/teamProvider';
import { DashboardPanel } from './panels/dashboardPanel';

let dashboardPanel: DashboardPanel | undefined;
let statusBarItem: vscode.StatusBarItem;
let webSocket: WebSocket | undefined;

export function activate(context: vscode.ExtensionContext) {
    // Register providers
    const analysisProvider = new AnalysisProvider();
    const metricsProvider = new MetricsProvider();
    const teamProvider = new TeamProvider();

    vscode.window.registerTreeDataProvider('code-historian-analysis', analysisProvider);
    vscode.window.registerTreeDataProvider('code-historian-metrics', metricsProvider);
    vscode.window.registerTreeDataProvider('code-historian-team', teamProvider);

    // Create status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
    statusBarItem.text = "$(graph) Code Historian";
    statusBarItem.command = 'code-historian.showDashboard';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Register commands
    let analyzeCommand = vscode.commands.registerCommand('code-historian.analyze', async () => {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        try {
            const config = vscode.workspace.getConfiguration('code-historian');
            const serverUrl = config.get<string>('serverUrl');

            const response = await axios.post(`${serverUrl}/api/analysis/start`, {
                repository: workspaceFolder.uri.fsPath,
                branch: 'main',
                team_members: []
            });

            if (response.data.id) {
                vscode.window.showInformationMessage('Analysis started successfully');
                connectWebSocket(response.data.id);
                analysisProvider.refresh();
            }
        } catch (error) {
            vscode.window.showErrorMessage('Failed to start analysis');
            console.error(error);
        }
    });

    let showDashboardCommand = vscode.commands.registerCommand('code-historian.showDashboard', () => {
        if (dashboardPanel) {
            dashboardPanel.reveal();
        } else {
            dashboardPanel = new DashboardPanel(context.extensionUri);
        }
    });

    context.subscriptions.push(analyzeCommand, showDashboardCommand);

    // Auto-analyze if enabled
    const config = vscode.workspace.getConfiguration('code-historian');
    if (config.get<boolean>('autoAnalyze')) {
        vscode.commands.executeCommand('code-historian.analyze');
    }
}

function connectWebSocket(analysisId: string) {
    const config = vscode.workspace.getConfiguration('code-historian');
    const serverUrl = config.get<string>('serverUrl');
    const wsUrl = serverUrl?.replace('http', 'ws');

    webSocket = new WebSocket(`${wsUrl}/ws/analysis/${analysisId}`);

    webSocket.on('message', (data: string) => {
        try {
            const update = JSON.parse(data);
            updateUI(update);
        } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
        }
    });

    webSocket.on('error', (error: Error) => {
        console.error('WebSocket error:', error);
        vscode.window.showErrorMessage('Lost connection to Code Historian server');
    });
}

function updateUI(update: any) {
    // Update status bar
    if (update.progress !== undefined) {
        statusBarItem.text = `$(graph) Code Historian: ${Math.round(update.progress * 100)}%`;
    }

    // Update dashboard
    if (dashboardPanel) {
        dashboardPanel.update(update);
    }

    // Refresh tree views
    const analysisProvider = vscode.window.createTreeView('code-historian-analysis', {
        treeDataProvider: new AnalysisProvider()
    });
    const metricsProvider = vscode.window.createTreeView('code-historian-metrics', {
        treeDataProvider: new MetricsProvider()
    });
    const teamProvider = vscode.window.createTreeView('code-historian-team', {
        treeDataProvider: new TeamProvider()
    });

    analysisProvider.refresh();
    metricsProvider.refresh();
    teamProvider.refresh();
}

export function deactivate() {
    if (webSocket) {
        webSocket.close();
    }
    if (dashboardPanel) {
        dashboardPanel.dispose();
    }
} 