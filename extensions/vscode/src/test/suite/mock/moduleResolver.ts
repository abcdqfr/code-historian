import * as mockVscode from './vscode';

// Mock module resolver
const resolver = {
    require(moduleName: string) {
        if (moduleName === 'vscode') {
            return mockVscode;
        }
        return require(moduleName);
    }
};

// Override require
(global as any).require = resolver.require;

export default resolver; 