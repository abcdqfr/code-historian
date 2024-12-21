import * as vscode from 'vscode';
import * as ts from 'typescript';
import { ErrorMonitor, TypeScriptError } from './errorMonitor';

interface FixAction {
    title: string;
    edit: vscode.WorkspaceEdit;
}

interface DiagnosticWithFile extends ts.Diagnostic {
    file: ts.SourceFile;
}

export class ErrorFixer {
    private static instance: ErrorFixer;
    private errorMonitor: ErrorMonitor;

    private constructor() {
        this.errorMonitor = ErrorMonitor.getInstance();
    }

    public static getInstance(): ErrorFixer {
        if (!ErrorFixer.instance) {
            ErrorFixer.instance = new ErrorFixer();
        }
        return ErrorFixer.instance;
    }

    public async fixAllErrors(): Promise<void> {
        const allErrors = this.errorMonitor.getAllErrors();
        
        for (const [file, errors] of allErrors) {
            for (const error of errors) {
                const diagnostic = this.convertToDiagnostic(error, file);
                const fixes = await this.getSuggestedFixes(diagnostic);
                if (fixes.length > 0) {
                    await vscode.workspace.applyEdit(fixes[0].edit);
                    // Verify if the fix worked
                    await this.verifyFix(file, diagnostic);
                }
            }
        }
    }

    private convertToDiagnostic(error: TypeScriptError, filePath: string): DiagnosticWithFile {
        return {
            category: error.category || ts.DiagnosticCategory.Error,
            code: error.code,
            file: ts.createSourceFile(
                filePath,
                '',
                ts.ScriptTarget.Latest,
                true
            ),
            start: error.start || error.character,
            length: error.length || 1,
            messageText: error.messageText || error.message
        };
    }

    private async verifyFix(file: string, originalError: ts.Diagnostic): Promise<boolean> {
        // Wait a bit for the fix to be applied
        await new Promise(resolve => setTimeout(resolve, 100));
        
        // Check if the error still exists
        const currentErrors = this.errorMonitor.getErrors(file);
        const errorStillExists = currentErrors.some(
            error => error.code === originalError.code && 
                     error.start === originalError.start &&
                     error.message === originalError.messageText.toString()
        );
        
        if (!errorStillExists) {
            void vscode.window.showInformationMessage(`Fixed error: ${originalError.messageText}`);
            return true;
        }
        
        return false;
    }

    private async getSuggestedFixes(diagnostic: DiagnosticWithFile): Promise<FixAction[]> {
        const fixes: FixAction[] = [];
        
        switch (diagnostic.code) {
            case 2304: // Cannot find name 'X'
                fixes.push(await this.createImportFix(diagnostic));
                break;
            case 7016: // Could not find declaration file
                fixes.push(await this.createTypeDefinitionFix(diagnostic));
                break;
            case 2339: // Property does not exist
                fixes.push(await this.createPropertyFix(diagnostic));
                break;
            case 2322: // Type mismatch
                fixes.push(await this.createTypeCastFix(diagnostic));
                break;
            case 2741: // Property is missing in type but required
                fixes.push(await this.createMissingPropertyFix(diagnostic));
                break;
            case 2554: // Expected N arguments but got M
                fixes.push(await this.createArgumentFix(diagnostic));
                break;
        }
        
        return fixes;
    }

    private async createImportFix(diagnostic: DiagnosticWithFile): Promise<FixAction> {
        const edit = new vscode.WorkspaceEdit();
        const missingName = diagnostic.messageText.toString().match(/Cannot find name '(\w+)'/)?.[1];
        
        if (missingName) {
            const importStatement = `import { ${missingName} } from '${this.guessModuleName(missingName)}';\n`;
            const position = new vscode.Position(0, 0);
            edit.insert(vscode.Uri.file(diagnostic.file.fileName), position, importStatement);
        }
        
        return {
            title: `Import ${missingName}`,
            edit
        };
    }

    private async createTypeDefinitionFix(diagnostic: DiagnosticWithFile): Promise<FixAction> {
        const edit = new vscode.WorkspaceEdit();
        const moduleName = diagnostic.messageText.toString().match(/for module '(.+)'/)?.[1];
        
        if (moduleName) {
            const terminal = vscode.window.createTerminal('Type Definition Install');
            terminal.sendText(`npm install --save-dev @types/${moduleName}`);
            terminal.dispose();
        }
        
        return {
            title: `Install @types/${moduleName}`,
            edit
        };
    }

    private async createPropertyFix(diagnostic: DiagnosticWithFile): Promise<FixAction> {
        const edit = new vscode.WorkspaceEdit();
        const propertyName = diagnostic.messageText.toString().match(/Property '(\w+)' does not exist/)?.[1];
        
        if (propertyName) {
            const interfacePosition = await this.findInterfacePosition(diagnostic.file.fileName);
            if (interfacePosition) {
                edit.insert(
                    vscode.Uri.file(diagnostic.file.fileName),
                    interfacePosition,
                    `    ${propertyName}: any; // TODO: Specify correct type\n`
                );
            }
        }
        
        return {
            title: `Add property ${propertyName}`,
            edit
        };
    }

    private async createTypeCastFix(diagnostic: DiagnosticWithFile): Promise<FixAction> {
        const edit = new vscode.WorkspaceEdit();
        const match = diagnostic.messageText.toString().match(/Type '(.+)' is not assignable to type '(.+)'/);
        
        if (match) {
            const [, fromType, toType] = match;
            const position = this.getDiagnosticPosition(diagnostic);
            if (position) {
                edit.insert(
                    vscode.Uri.file(diagnostic.file.fileName),
                    position,
                    `as ${toType}`
                );
            }
        }
        
        return {
            title: 'Add type cast',
            edit
        };
    }

    private async createMissingPropertyFix(diagnostic: DiagnosticWithFile): Promise<FixAction> {
        const edit = new vscode.WorkspaceEdit();
        const match = diagnostic.messageText.toString().match(/Property '(\w+)' is missing/);
        
        if (match) {
            const [, propertyName] = match;
            const position = this.getDiagnosticPosition(diagnostic);
            if (position) {
                edit.insert(
                    vscode.Uri.file(diagnostic.file.fileName),
                    position,
                    `, ${propertyName}: undefined // TODO: Provide correct value`
                );
            }
        }
        
        return {
            title: 'Add missing property',
            edit
        };
    }

    private async createArgumentFix(diagnostic: DiagnosticWithFile): Promise<FixAction> {
        const edit = new vscode.WorkspaceEdit();
        const match = diagnostic.messageText.toString().match(/Expected (\d+) arguments, but got (\d+)/);
        
        if (match) {
            const [, expected, actual] = match;
            const position = this.getDiagnosticPosition(diagnostic);
            if (position) {
                const missingArgs = parseInt(expected) - parseInt(actual);
                const placeholders = Array(missingArgs).fill('undefined').join(', ');
                edit.insert(
                    vscode.Uri.file(diagnostic.file.fileName),
                    position,
                    `, ${placeholders}`
                );
            }
        }
        
        return {
            title: 'Add missing arguments',
            edit
        };
    }

    private guessModuleName(name: string): string {
        // Common module mappings
        const moduleMap: Record<string, string> = {
            'vscode': 'vscode',
            'path': 'path',
            'fs': 'fs',
            'axios': 'axios',
            'WebSocket': 'ws',
            'ts': 'typescript'
        };
        
        return moduleMap[name] || name.toLowerCase();
    }

    private async findInterfacePosition(file: string): Promise<vscode.Position | undefined> {
        const document = await vscode.workspace.openTextDocument(file);
        const text = document.getText();
        const interfaceMatch = text.match(/interface\s+\w+\s*{/);
        
        if (interfaceMatch) {
            const offset = interfaceMatch.index! + interfaceMatch[0].length;
            return document.positionAt(offset);
        }
        
        return undefined;
    }

    private getDiagnosticPosition(diagnostic: DiagnosticWithFile): vscode.Position | undefined {
        if (diagnostic.start !== undefined) {
            const document = vscode.workspace.textDocuments.find(
                doc => doc.uri.fsPath === diagnostic.file.fileName
            );
            if (document) {
                return document.positionAt(diagnostic.start);
            }
        }
        return undefined;
    }
} 