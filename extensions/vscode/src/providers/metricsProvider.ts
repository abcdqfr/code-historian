import * as vscode from 'vscode';
import * as path from 'path';
import axios, { AxiosError } from 'axios';

class MetricItem extends vscode.TreeItem {
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
            light: vscode.Uri.file(path.join(__dirname, '..', '..', 'resources', 'light', 'metric.svg')),
            dark: vscode.Uri.file(path.join(__dirname, '..', '..', 'resources', 'dark', 'metric.svg'))
        };
    }
}

export class MetricsProvider implements vscode.TreeDataProvider<MetricItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<MetricItem | undefined | null | void> = new vscode.EventEmitter<MetricItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<MetricItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {
        this.refresh();
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: MetricItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: MetricItem): Promise<MetricItem[]> {
        if (!element) {
            return this.getRootItems();
        }
        return this.getMetricDetails(element);
    }

    private async getRootItems(): Promise<MetricItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const response = await axios.get<{ metrics: Array<{ name: string; value: number; hasDetails?: boolean }> }>(
                `${serverUrl}/api/metrics/summary`,
                { headers: { 'X-API-Key': apiKey } }
            );

            return response.data.metrics.map(metric => 
                new MetricItem(
                    metric.name,
                    metric.hasDetails ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None,
                    `${metric.value}`
                )
            );

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            vscode.window.showErrorMessage(`Failed to load metrics data: ${errorMessage}`);
            return [];
        }
    }

    private async getMetricDetails(element: MetricItem): Promise<MetricItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const response = await axios.get<{ details: Array<{ name: string; value: string | number }> }>(
                `${serverUrl}/api/metrics/details/${encodeURIComponent(element.label)}`,
                { headers: { 'X-API-Key': apiKey } }
            );

            return response.data.details.map(detail => 
                new MetricItem(
                    detail.name,
                    vscode.TreeItemCollapsibleState.None,
                    `${detail.value}`
                )
            );

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            vscode.window.showErrorMessage(`Failed to load metric details: ${errorMessage}`);
            return [];
        }
    }
} 