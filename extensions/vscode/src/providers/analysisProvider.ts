import * as vscode from 'vscode';
import * as path from 'path';
import axios, { AxiosError } from 'axios';

class AnalysisItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly description?: string,
        public readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.description = description;
        this.command = command;
        this.iconPath = {
            light: vscode.Uri.file(path.join(__dirname, '..', '..', 'resources', 'light', 'analysis.svg')),
            dark: vscode.Uri.file(path.join(__dirname, '..', '..', 'resources', 'dark', 'analysis.svg'))
        };
    }
}

export class AnalysisProvider implements vscode.TreeDataProvider<AnalysisItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<AnalysisItem | undefined | null | void> = new vscode.EventEmitter<AnalysisItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<AnalysisItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {
        this.refresh();
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: AnalysisItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: AnalysisItem): Promise<AnalysisItem[]> {
        if (!element) {
            return this.getRootItems();
        }
        return this.getAnalysisDetails(element);
    }

    private async getRootItems(): Promise<AnalysisItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const response = await axios.get<{ items: Array<{ name: string; value: number; hasDetails?: boolean }> }>(
                `${serverUrl}/api/analysis/summary`,
                { headers: { 'X-API-Key': apiKey } }
            );

            return response.data.items.map(item => 
                new AnalysisItem(
                    item.name,
                    item.hasDetails ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None,
                    `${item.value}`,
                    item.hasDetails ? {
                        command: 'codeHistorian.showAnalysisDetails',
                        title: 'Show Details',
                        arguments: [item.name]
                    } : undefined
                )
            );

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            vscode.window.showErrorMessage(`Failed to load analysis data: ${errorMessage}`);
            return [];
        }
    }

    private async getAnalysisDetails(element: AnalysisItem): Promise<AnalysisItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const response = await axios.get<{ details: Array<{ name: string; value: string | number }> }>(
                `${serverUrl}/api/analysis/details/${encodeURIComponent(element.label)}`,
                { headers: { 'X-API-Key': apiKey } }
            );

            return response.data.details.map(detail => 
                new AnalysisItem(
                    detail.name,
                    vscode.TreeItemCollapsibleState.None,
                    `${detail.value}`
                )
            );

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            vscode.window.showErrorMessage(`Failed to load analysis details: ${errorMessage}`);
            return [];
        }
    }
} 