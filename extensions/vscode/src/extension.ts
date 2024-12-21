import * as vscode from 'vscode';
import { DashboardPanel } from './panels/dashboardPanel';
import { ErrorMonitor } from './utils/errorMonitor';
import { ErrorFixer } from './utils/errorFixer';
import * as ts from 'typescript';

export function activate(context: vscode.ExtensionContext) {
    console.log('Code Historian extension is now active');

    // Initialize error monitoring system
    const errorMonitor = ErrorMonitor.getInstance();
    const errorFixer = ErrorFixer.getInstance();

    // Register auto-fix command
    let disposable = vscode.commands.registerCommand('codeHistorian.fixTypeScriptErrors', async () => {
        await errorFixer.fixAllErrors();
    });
    context.subscriptions.push(disposable);

    // Register dashboard command
    disposable = vscode.commands.registerCommand('codeHistorian.showDashboard', () => {
        DashboardPanel.createOrShow(context.extensionUri);
    });
    context.subscriptions.push(disposable);

    // Register analysis command
    disposable = vscode.commands.registerCommand('codeHistorian.startAnalysis', async () => {
        const progress = await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: "Analyzing repository...",
            cancellable: true
        }, async (progress, token) => {
            token.onCancellationRequested(() => {
                console.log("User canceled the analysis");
            });

            progress.report({ increment: 0 });

            try {
                // Simulate analysis progress
                for (let i = 0; i < 100; i += 10) {
                    if (token.isCancellationRequested) {
                        break;
                    }
                    await new Promise(resolve => setTimeout(resolve, 1000));
                    progress.report({ increment: 10, message: `${i + 10}% complete` });
                }

                if (!token.isCancellationRequested) {
                    vscode.window.showInformationMessage('Analysis complete!');
                    DashboardPanel.createOrShow(context.extensionUri);
                }
            } catch (error) {
                const errorMessage = error instanceof Error ? error.message : 'Unknown error';
                vscode.window.showErrorMessage(`Analysis failed: ${errorMessage}`);
            }
        });
    });
    context.subscriptions.push(disposable);

    // Watch for TypeScript errors
    const watcher = vscode.workspace.createFileSystemWatcher('**/*.ts');
    context.subscriptions.push(watcher);

    // Auto-fix setting
    const config = vscode.workspace.getConfiguration('codeHistorian');
    const autoFix = config.get<boolean>('autoFixTypeScriptErrors');

    watcher.onDidChange(async uri => {
        const document = await vscode.workspace.openTextDocument(uri);
        const diagnostics = vscode.languages.getDiagnostics(uri);
        
        // Clear old errors
        errorMonitor.clearErrors(uri.fsPath);

        // Track new errors
        for (const diagnostic of diagnostics) {
            if (diagnostic.severity === vscode.DiagnosticSeverity.Error) {
                errorMonitor.trackError({
                    file: uri.fsPath,
                    line: diagnostic.range.start.line + 1,
                    character: diagnostic.range.start.character,
                    message: diagnostic.message,
                    code: diagnostic.code as number,
                    category: ts.DiagnosticCategory.Error,
                    start: diagnostic.range.start.character,
                    length: diagnostic.range.end.character - diagnostic.range.start.character,
                    messageText: diagnostic.message
                });
            }
        }

        // Auto-fix if enabled
        if (autoFix) {
            await errorFixer.fixAllErrors();
        }
    });
}

export function deactivate() {} 