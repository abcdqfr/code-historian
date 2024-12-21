import * as vscode from 'vscode';
import axios from 'axios';

export class AnalysisProvider implements vscode.TreeDataProvider<AnalysisItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<AnalysisItem | undefined | null | void> = new vscode.EventEmitter<AnalysisItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<AnalysisItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private analyses: AnalysisItem[] = [];

    constructor() {
        this.fetchAnalyses();
    }

    refresh(): void {
        this.fetchAnalyses();
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: AnalysisItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: AnalysisItem): Thenable<AnalysisItem[]> {
        if (element) {
            return Promise.resolve(element.children || []);
        }
        return Promise.resolve(this.analyses);
    }

    private async fetchAnalyses() {
        try {
            const config = vscode.workspace.getConfiguration('code-historian');
            const serverUrl = config.get<string>('serverUrl');
            const response = await axios.get(`${serverUrl}/api/analysis`);
            
            this.analyses = response.data.map((analysis: any) => {
                const item = new AnalysisItem(
                    analysis.repository,
                    analysis.status,
                    vscode.TreeItemCollapsibleState.Collapsed
                );

                item.children = [
                    new AnalysisItem(
                        `Progress: ${Math.round(analysis.progress * 100)}%`,
                        'progress',
                        vscode.TreeItemCollapsibleState.None
                    ),
                    new AnalysisItem(
                        `Started: ${new Date(analysis.started_at).toLocaleString()}`,
                        'time',
                        vscode.TreeItemCollapsibleState.None
                    )
                ];

                if (analysis.metrics) {
                    item.children.push(
                        new AnalysisItem(
                            `Changes: ${analysis.metrics.total_changes}`,
                            'changes',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new AnalysisItem(
                            `Impact Score: ${analysis.metrics.impact_score.toFixed(2)}`,
                            'impact',
                            vscode.TreeItemCollapsibleState.None
                        )
                    );
                }

                return item;
            });
        } catch (error) {
            console.error('Failed to fetch analyses:', error);
            vscode.window.showErrorMessage('Failed to fetch analyses');
        }
    }
}

class AnalysisItem extends vscode.TreeItem {
    children?: AnalysisItem[];

    constructor(
        public readonly label: string,
        public readonly type: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);

        this.tooltip = label;
        
        switch (type) {
            case 'running':
                this.iconPath = new vscode.ThemeIcon('sync~spin');
                break;
            case 'completed':
                this.iconPath = new vscode.ThemeIcon('check');
                break;
            case 'failed':
                this.iconPath = new vscode.ThemeIcon('error');
                break;
            case 'progress':
                this.iconPath = new vscode.ThemeIcon('graph');
                break;
            case 'time':
                this.iconPath = new vscode.ThemeIcon('clock');
                break;
            case 'changes':
                this.iconPath = new vscode.ThemeIcon('git-commit');
                break;
            case 'impact':
                this.iconPath = new vscode.ThemeIcon('pulse');
                break;
        }
    }
} 