import * as vscode from 'vscode';
import * as path from 'path';
import axios from 'axios';

export class MetricsProvider implements vscode.TreeDataProvider<MetricItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<MetricItem | undefined | null | void> = new vscode.EventEmitter<MetricItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<MetricItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: MetricItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: MetricItem): Promise<MetricItem[]> {
        if (!element) {
            return this.getRootMetrics();
        }
        return this.getMetricDetails(element);
    }

    private async getRootMetrics(): Promise<MetricItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                return [new MetricItem('Configuration Required', 'Configure server URL and API key in settings', vscode.TreeItemCollapsibleState.None)];
            }

            const response = await axios.get(`${serverUrl}/api/metrics/summary`, {
                headers: {
                    'X-API-Key': apiKey
                }
            });

            return [
                new MetricItem(
                    'Code Churn',
                    'Changes over time',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    {
                        value: response.data.codeChurn,
                        trend: response.data.codeChurnTrend
                    }
                ),
                new MetricItem(
                    'Team Collaboration',
                    'Team metrics',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    {
                        value: response.data.teamScore,
                        trend: response.data.teamScoreTrend
                    }
                ),
                new MetricItem(
                    'Impact Analysis',
                    'Code impact metrics',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    {
                        value: response.data.impactScore,
                        trend: response.data.impactScoreTrend
                    }
                ),
                new MetricItem(
                    'Custom Metrics',
                    'User-defined metrics',
                    vscode.TreeItemCollapsibleState.Collapsed
                )
            ];

        } catch (error) {
            return [new MetricItem('Error', error.message, vscode.TreeItemCollapsibleState.None)];
        }
    }

    private async getMetricDetails(metric: MetricItem): Promise<MetricItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                return [];
            }

            const response = await axios.get(`${serverUrl}/api/metrics/details/${metric.label}`, {
                headers: {
                    'X-API-Key': apiKey
                }
            });

            return Object.entries(response.data).map(([key, value]: [string, any]) => 
                new MetricItem(
                    key,
                    value.toString(),
                    vscode.TreeItemCollapsibleState.None,
                    {
                        value: value,
                        trend: value.trend
                    }
                )
            );

        } catch (error) {
            return [new MetricItem('Error', error.message, vscode.TreeItemCollapsibleState.None)];
        }
    }
}

interface MetricData {
    value?: number;
    trend?: 'up' | 'down' | 'stable';
}

class MetricItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        private description: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        private metricData?: MetricData
    ) {
        super(label, collapsibleState);
        this.tooltip = `${description}\n${this.getMetricDetails()}`;
        this.description = this.getDescription();
    }

    private getMetricDetails(): string {
        if (!this.metricData) {
            return '';
        }

        const parts = [];
        if (this.metricData.value !== undefined) {
            parts.push(`Value: ${this.metricData.value}`);
        }
        if (this.metricData.trend) {
            parts.push(`Trend: ${this.metricData.trend}`);
        }
        return parts.join('\n');
    }

    private getDescription(): string {
        if (!this.metricData || this.metricData.value === undefined) {
            return this.description;
        }
        return `${this.description} (${this.metricData.value})`;
    }

    iconPath = {
        light: path.join(__filename, '..', '..', '..', 'resources', 'light', 'metric.svg'),
        dark: path.join(__filename, '..', '..', '..', 'resources', 'dark', 'metric.svg')
    };

    contextValue = 'metricItem';
} 