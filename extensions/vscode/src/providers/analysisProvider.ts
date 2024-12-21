import * as vscode from 'vscode';

export class AnalysisItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly description?: string
    ) {
        super(label, collapsibleState);
        this.description = description;
        this.iconPath = {
            light: vscode.Uri.file(__dirname + '/../resources/light/analysis.svg'),
            dark: vscode.Uri.file(__dirname + '/../resources/dark/analysis.svg')
        };

        if (collapsibleState === vscode.TreeItemCollapsibleState.Collapsed) {
            this.command = {
                command: 'codeHistorian.showAnalysisDetails',
                title: 'Show Analysis Details',
                arguments: [this]
            };
        }
    }
}

export class AnalysisProvider implements vscode.TreeDataProvider<AnalysisItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<AnalysisItem | undefined> = new vscode.EventEmitter<AnalysisItem | undefined>();
    readonly onDidChangeTreeData: vscode.Event<AnalysisItem | undefined> = this._onDidChangeTreeData.event;

    constructor() {
        // Initialize provider
        this.refresh();
    }

    refresh(): void {
        this._onDidChangeTreeData.fire(undefined);
    }

    getTreeItem(element: AnalysisItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: AnalysisItem): Promise<AnalysisItem[]> {
        const config = vscode.workspace.getConfiguration('codeHistorian');
        const serverUrl = config.get<string>('serverUrl');

        if (!serverUrl) {
            return [new AnalysisItem('Server URL not configured', vscode.TreeItemCollapsibleState.None)];
        }

        try {
            if (!element) {
                // Root items
                return [
                    new AnalysisItem('Code Quality', vscode.TreeItemCollapsibleState.Collapsed),
                    new AnalysisItem('Technical Debt', vscode.TreeItemCollapsibleState.Collapsed)
                ];
            } else {
                // Detail items
                return [
                    new AnalysisItem('Detail 1', vscode.TreeItemCollapsibleState.None, 'Value 1'),
                    new AnalysisItem('Detail 2', vscode.TreeItemCollapsibleState.None, 'Value 2')
                ];
            }
        } catch (error) {
            return [new AnalysisItem('Error fetching analysis data', vscode.TreeItemCollapsibleState.None)];
        }
    }
} 