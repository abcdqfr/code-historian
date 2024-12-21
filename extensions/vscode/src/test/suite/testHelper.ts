import { TreeItemCollapsibleState, EventEmitter, Uri, TreeItem, WorkspaceConfiguration } from 'vscode';
import * as sinon from 'sinon';

// Mock vscode module
export const vscode = {
    TreeItemCollapsibleState,
    EventEmitter,
    Uri,
    TreeItem,
    workspace: {
        getConfiguration: () => ({
            get: (section: string) => '',
            update: (section: string, value: any) => Promise.resolve(),
            has: (section: string) => true,
            inspect: (section: string) => undefined
        })
    },
    ConfigurationTarget: {
        Global: 1,
        Workspace: 2,
        WorkspaceFolder: 3
    }
};

// Mock workspace configuration type
export interface MockWorkspaceConfig extends WorkspaceConfiguration {
    get: sinon.SinonStub;
    update: sinon.SinonStub;
    has: sinon.SinonStub;
    inspect: sinon.SinonStub;
} 