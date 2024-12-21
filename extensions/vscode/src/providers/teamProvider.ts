import * as vscode from 'vscode';
import axios from 'axios';

export class TeamProvider implements vscode.TreeDataProvider<TeamItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TeamItem | undefined | null | void> = new vscode.EventEmitter<TeamItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TeamItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private teamMembers: TeamItem[] = [];

    constructor() {
        this.fetchTeamMembers();
    }

    refresh(): void {
        this.fetchTeamMembers();
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TeamItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: TeamItem): Thenable<TeamItem[]> {
        if (element) {
            return Promise.resolve(element.children || []);
        }
        return Promise.resolve(this.teamMembers);
    }

    private async fetchTeamMembers() {
        try {
            const config = vscode.workspace.getConfiguration('code-historian');
            const serverUrl = config.get<string>('serverUrl');
            const response = await axios.get(`${serverUrl}/api/team/members`);
            
            this.teamMembers = response.data.map((member: any) => {
                const item = new TeamItem(
                    member.name,
                    member.role,
                    vscode.TreeItemCollapsibleState.Collapsed
                );

                item.children = [
                    new TeamItem(
                        `Role: ${member.role}`,
                        'role',
                        vscode.TreeItemCollapsibleState.None
                    ),
                    new TeamItem(
                        `Active Projects: ${member.active_projects.length}`,
                        'projects',
                        vscode.TreeItemCollapsibleState.None
                    )
                ];

                // Add project details
                if (member.active_projects.length > 0) {
                    const projectsItem = new TeamItem(
                        'Projects',
                        'project-list',
                        vscode.TreeItemCollapsibleState.Collapsed
                    );

                    projectsItem.children = member.active_projects.map((project: string) => 
                        new TeamItem(
                            project,
                            'project',
                            vscode.TreeItemCollapsibleState.None
                        )
                    );

                    item.children.push(projectsItem);
                }

                // Add recent activity
                if (member.recent_activity && member.recent_activity.length > 0) {
                    const activityItem = new TeamItem(
                        'Recent Activity',
                        'activity-list',
                        vscode.TreeItemCollapsibleState.Collapsed
                    );

                    activityItem.children = member.recent_activity.map((activity: any) =>
                        new TeamItem(
                            `${activity.action} (${new Date(activity.timestamp).toLocaleString()})`,
                            'activity',
                            vscode.TreeItemCollapsibleState.None
                        )
                    );

                    item.children.push(activityItem);
                }

                // Add metrics
                if (member.metrics) {
                    const metricsItem = new TeamItem(
                        'Metrics',
                        'metrics-list',
                        vscode.TreeItemCollapsibleState.Collapsed
                    );

                    metricsItem.children = [
                        new TeamItem(
                            `Commits: ${member.metrics.total_commits}`,
                            'commits',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new TeamItem(
                            `Changes: ${member.metrics.total_changes}`,
                            'changes',
                            vscode.TreeItemCollapsibleState.None
                        ),
                        new TeamItem(
                            `Impact Score: ${member.metrics.impact_score.toFixed(2)}`,
                            'impact',
                            vscode.TreeItemCollapsibleState.None
                        )
                    ];

                    item.children.push(metricsItem);
                }

                return item;
            });
        } catch (error) {
            console.error('Failed to fetch team members:', error);
            vscode.window.showErrorMessage('Failed to fetch team members');
        }
    }
}

class TeamItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly type: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly children?: TeamItem[]
    ) {
        super(label, collapsibleState);

        this.tooltip = label;
        
        switch (type) {
            case 'role':
                this.iconPath = new vscode.ThemeIcon('person');
                break;
            case 'projects':
                this.iconPath = new vscode.ThemeIcon('folder');
                break;
            case 'project-list':
                this.iconPath = new vscode.ThemeIcon('list-tree');
                break;
            case 'project':
                this.iconPath = new vscode.ThemeIcon('repo');
                break;
            case 'activity-list':
                this.iconPath = new vscode.ThemeIcon('history');
                break;
            case 'activity':
                this.iconPath = new vscode.ThemeIcon('git-commit');
                break;
            case 'metrics-list':
                this.iconPath = new vscode.ThemeIcon('graph');
                break;
            case 'commits':
                this.iconPath = new vscode.ThemeIcon('git-commit');
                break;
            case 'changes':
                this.iconPath = new vscode.ThemeIcon('diff');
                break;
            case 'impact':
                this.iconPath = new vscode.ThemeIcon('pulse');
                break;
        }

        // Add command for project items
        if (type === 'project') {
            this.command = {
                title: 'Open Project',
                command: 'code-historian.openProject',
                arguments: [this.label]
            };
        }
    }
} 