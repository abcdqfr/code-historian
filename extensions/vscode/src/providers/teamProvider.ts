import * as vscode from 'vscode';
import * as path from 'path';
import axios, { AxiosError } from 'axios';

interface TeamMember {
    name: string;
    email: string;
    commits: number;
    impact: number;
}

class TeamItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly description?: string,
        public readonly command?: vscode.Command,
        private readonly members: TeamMember[] = []
    ) {
        super(label, collapsibleState);
        this.description = description;
        this.command = command;
        this.iconPath = {
            light: vscode.Uri.file(path.join(__dirname, '..', '..', 'resources', 'light', 'team.svg')),
            dark: vscode.Uri.file(path.join(__dirname, '..', '..', 'resources', 'dark', 'team.svg'))
        };
    }

    getMembers(): TeamMember[] {
        return this.members;
    }
}

export class TeamProvider implements vscode.TreeDataProvider<TeamItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TeamItem | undefined | null | void> = new vscode.EventEmitter<TeamItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TeamItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {
        this.refresh();
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TeamItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: TeamItem): Promise<TeamItem[]> {
        if (!element) {
            return this.getRootItems();
        }
        return this.getMemberItems(element);
    }

    private async getRootItems(): Promise<TeamItem[]> {
        try {
            const config = vscode.workspace.getConfiguration('codeHistorian');
            const serverUrl = config.get<string>('serverUrl');
            const apiKey = config.get<string>('apiKey');

            if (!serverUrl || !apiKey) {
                throw new Error('Server URL and API key must be configured');
            }

            const response = await axios.get<{ teams: Array<{ name: string; members: TeamMember[] }> }>(
                `${serverUrl}/api/team/summary`,
                { headers: { 'X-API-Key': apiKey } }
            );

            return response.data.teams.map(team => 
                new TeamItem(
                    team.name,
                    vscode.TreeItemCollapsibleState.Collapsed,
                    `${team.members.length} members`,
                    undefined,
                    team.members
                )
            );

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            vscode.window.showErrorMessage(`Failed to load team data: ${errorMessage}`);
            return [];
        }
    }

    private getMemberItems(team: TeamItem): TeamItem[] {
        return team.getMembers().map(member => 
            new TeamItem(
                member.name,
                vscode.TreeItemCollapsibleState.None,
                `${member.commits} commits, Impact: ${Math.round(member.impact * 100)}%`
            )
        );
    }
} 