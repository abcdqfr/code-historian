import * as vscode from 'vscode';
import axios from 'axios';

export class MetricsProvider implements vscode.TreeDataProvider<MetricItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<MetricItem | undefined | null | void> = new vscode.EventEmitter<MetricItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<MetricItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private metrics: MetricItem[] = [];

    constructor() {
        this.fetchMetrics();
    }

    refresh(): void {
        this.fetchMetrics();
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: MetricItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: MetricItem): Thenable<MetricItem[]> {
        if (element) {
            return Promise.resolve(element.children || []);
        }
        return Promise.resolve(this.metrics);
    }

    private async fetchMetrics() {
        try {
            const config = vscode.workspace.getConfiguration('code-historian');
            const serverUrl = config.get<string>('serverUrl');
            const response = await axios.get(`${serverUrl}/api/metrics`);
            
            this.metrics = [
                new MetricItem(
                    'Performance',
                    'performance',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    [
                        new MetricItem(
                            `Repository Processing: ${response.data.performance.repo_processing} KB/s`,
                            'speed',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Memory Usage: ${response.data.performance.memory_usage} MB`,
                            'memory',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Plugin Load Time: ${response.data.performance.plugin_load_time} ms`,
                            'time',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Chart Generation: ${response.data.performance.chart_generation} ms`,
                            'chart',
                            vscode.TreeItemCollapsibleState.None
                        )
                    ]
                ),
                new MetricItem(
                    'Code Quality',
                    'quality',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    [
                        new MetricItem(
                            `Test Coverage: ${response.data.quality.test_coverage}%`,
                            'test',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Documentation: ${response.data.quality.documentation}%`,
                            'docs',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Code Review: ${response.data.quality.code_review}%`,
                            'review',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Performance Score: ${response.data.quality.performance}%`,
                            'score',
                            vscode.TreeItemCollapsibleState.None
                        )
                    ]
                ),
                new MetricItem(
                    'Impact Analysis',
                    'impact',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    [
                        new MetricItem(
                            `High Impact Changes: ${response.data.impact.high_impact}`,
                            'high',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Medium Impact Changes: ${response.data.impact.medium_impact}`,
                            'medium',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Low Impact Changes: ${response.data.impact.low_impact}`,
                            'low',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new MetricItem(
                            `Average Impact Score: ${response.data.impact.avg_score.toFixed(2)}`,
                            'average',
                            vscode.TreeItemCollapsibleState.None
                        )
                    ]
                )
            ];
        } catch (error) {
            console.error('Failed to fetch metrics:', error);
            vscode.window.showErrorMessage('Failed to fetch metrics');
        }
    }
}

class MetricItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly type: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly children?: MetricItem[]
    ) {
        super(label, collapsibleState);

        this.tooltip = label;
        
        switch (type) {
            case 'performance':
                this.iconPath = new vscode.ThemeIcon('dashboard');
                break;
            case 'quality':
                this.iconPath = new vscode.ThemeIcon('shield');
                break;
            case 'impact':
                this.iconPath = new vscode.ThemeIcon('pulse');
                break;
            case 'speed':
                this.iconPath = new vscode.ThemeIcon('rocket');
                break;
            case 'memory':
                this.iconPath = new vscode.ThemeIcon('database');
                break;
            case 'time':
                this.iconPath = new vscode.ThemeIcon('clock');
                break;
            case 'chart':
                this.iconPath = new vscode.ThemeIcon('graph');
                break;
            case 'test':
                this.iconPath = new vscode.ThemeIcon('beaker');
                break;
            case 'docs':
                this.iconPath = new vscode.ThemeIcon('book');
                break;
            case 'review':
                this.iconPath = new vscode.ThemeIcon('checklist');
                break;
            case 'score':
                this.iconPath = new vscode.ThemeIcon('star');
                break;
            case 'high':
                this.iconPath = new vscode.ThemeIcon('arrow-up');
                break;
            case 'medium':
                this.iconPath = new vscode.ThemeIcon('arrow-right');
                break;
            case 'low':
                this.iconPath = new vscode.ThemeIcon('arrow-down');
                break;
            case 'average':
                this.iconPath = new vscode.ThemeIcon('symbol-numeric');
                break;
        }
    }
} 