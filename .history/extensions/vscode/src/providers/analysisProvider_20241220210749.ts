import * as vscode from 'vscode';
import * as path from 'path';
import axios from 'axios';

export class AnalysisProvider implements vscode.TreeDataProvider<AnalysisItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<AnalysisItem | undefined | null | void> = new vscode.EventEmitter<AnalysisItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<AnalysisItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {}

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
        return this.getAnalysisItems(element);
    }

    private async getRootItems(): Promise<AnalysisItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                return [new AnalysisItem('Configuration Required', 'Configure server URL and API key in settings', vscode.TreeItemCollapsibleState.None)];
            }

            const response = await axios.get(`${serverUrl}/api/analysis/status`, {
                headers: {
                    'X-API-Key': apiKey
                }
            });

            const items: AnalysisItem[] = [];

            if (response.data.lastAnalysis) {
                items.push(new AnalysisItem(
                    'Last Analysis',
                    `${response.data.lastAnalysis.timestamp}`,
                    vscode.TreeItemCollapsibleState.Expanded,
                    {
                        command: 'codeHistorian.showDashboard',
                        title: 'Show Dashboard',
                        arguments: []
                    }
                ));
            }

            items.push(new AnalysisItem(
                'Start New Analysis',
                'Click to analyze repository',
                vscode.TreeItemCollapsibleState.None,
                {
                    command: 'codeHistorian.startAnalysis',
                    title: 'Start Analysis',
                    arguments: []
                }
            ));

            return items;

        } catch (error) {
            return [new AnalysisItem('Error', error.message, vscode.TreeItemCollapsibleState.None)];
        }
    }

    private async getAnalysisItems(element: AnalysisItem): Promise<AnalysisItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                return [];
            }

            const response = await axios.get(`${serverUrl}/api/analysis/details`, {
                headers: {
                    'X-API-Key': apiKey
                }
            });

            return [
                new AnalysisItem('Files Analyzed', response.data.filesAnalyzed.toString(), vscode.TreeItemCollapsibleState.None),
                new AnalysisItem('Total Commits', response.data.totalCommits.toString(), vscode.TreeItemCollapsibleState.None),
                new AnalysisItem('Total Authors', response.data.totalAuthors.toString(), vscode.TreeItemCollapsibleState.None),
                new AnalysisItem('Analysis Duration', response.data.duration, vscode.TreeItemCollapsibleState.None)
            ];

        } catch (error) {
            return [new AnalysisItem('Error', error.message, vscode.TreeItemCollapsibleState.None)];
        }
    }
}

class AnalysisItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        private description: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.tooltip = description;
        this.description = description;
    }

    iconPath = {
        light: path.join(__filename, '..', '..', '..', 'resources', 'light', 'analysis.svg'),
        dark: path.join(__filename, '..', '..', '..', 'resources', 'dark', 'analysis.svg')
    };

    contextValue = 'analysisItem';
} 