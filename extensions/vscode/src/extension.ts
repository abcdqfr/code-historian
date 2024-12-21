import * as vscode from 'vscode';
import { MetricsProvider } from './providers/metricsProvider';
import { AnalysisProvider } from './providers/analysisProvider';
import { TeamProvider } from './providers/teamProvider';
import { DashboardPanel } from './panels/dashboardPanel';

export async function activate(context: vscode.ExtensionContext) {
    const metricsProvider = new MetricsProvider();
    const analysisProvider = new AnalysisProvider();
    const teamProvider = new TeamProvider();

    // Register tree data providers
    vscode.window.registerTreeDataProvider('codeHistorianMetrics', metricsProvider);
    vscode.window.registerTreeDataProvider('codeHistorianAnalysis', analysisProvider);
    vscode.window.registerTreeDataProvider('codeHistorianTeam', teamProvider);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('codeHistorian.refreshMetrics', () => {
            metricsProvider.refresh();
        }),
        vscode.commands.registerCommand('codeHistorian.refreshAnalysis', () => {
            analysisProvider.refresh();
        }),
        vscode.commands.registerCommand('codeHistorian.refreshTeam', () => {
            teamProvider.refresh();
        }),
        vscode.commands.registerCommand('codeHistorian.showDashboard', () => {
            DashboardPanel.createOrShow(context.extensionUri);
        }),
        vscode.commands.registerCommand('codeHistorian.showAnalysisDetails', async (itemName: string) => {
            const panel = DashboardPanel.createOrShow(context.extensionUri);
            await panel.showAnalysisDetails(itemName);
        })
    );

    // Initialize configuration
    const config = vscode.workspace.getConfiguration('codeHistorian');
    if (!config.get('serverUrl')) {
        await config.update('serverUrl', 'http://localhost:3000', vscode.ConfigurationTarget.Global);
    }

    // Watch for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('codeHistorian')) {
                metricsProvider.refresh();
                analysisProvider.refresh();
                teamProvider.refresh();
            }
        })
    );

    // Watch for active editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor) {
                metricsProvider.refresh();
                analysisProvider.refresh();
            }
        })
    );

    // Watch for document changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument(e => {
            if (e.document === vscode.window.activeTextEditor?.document) {
                metricsProvider.refresh();
                analysisProvider.refresh();
            }
        })
    );

    // Initial refresh
    metricsProvider.refresh();
    analysisProvider.refresh();
    teamProvider.refresh();
}

export function deactivate(): Thenable<void> {
    // Clean up resources
    return vscode.workspace.getConfiguration('codeHistorian')
        .update('serverUrl', undefined, vscode.ConfigurationTarget.Global)
        .then(() => undefined);
} 