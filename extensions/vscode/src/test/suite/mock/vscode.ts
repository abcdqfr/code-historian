// Mock vscode module for testing
export const TreeItemCollapsibleState = {
    None: 0,
    Collapsed: 1,
    Expanded: 2
};

export class EventEmitter<T> {
    private listeners: ((e: T) => any)[] = [];

    fire(data: T): void {
        this.listeners.forEach(listener => listener(data));
    }

    event(listener: (e: T) => any): void {
        this.listeners.push(listener);
    }

    dispose(): void {
        this.listeners = [];
    }
}

export class Uri {
    static file(path: string): Uri {
        return new Uri(path);
    }

    constructor(public fsPath: string) {}
}

export class TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: number
    ) {}
}

export interface WorkspaceConfiguration {
    get<T>(section: string): T | undefined;
    update(section: string, value: any): Thenable<void>;
    has(section: string): boolean;
    inspect<T>(section: string): undefined;
}

export const workspace = {
    getConfiguration: () => ({
        get: () => '',
        update: () => Promise.resolve(),
        has: () => true,
        inspect: () => undefined
    })
};

export const ConfigurationTarget = {
    Global: 1,
    Workspace: 2,
    WorkspaceFolder: 3
}; 