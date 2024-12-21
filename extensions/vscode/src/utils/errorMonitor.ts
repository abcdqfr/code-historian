import * as vscode from 'vscode';
import * as ts from 'typescript';

export interface TypeScriptError {
    file: string;
    line: number;
    character: number;
    message: string;
    code: number;
    category?: ts.DiagnosticCategory;
    start?: number;
    length?: number;
    messageText?: string | ts.DiagnosticMessageChain;
}

export class ErrorMonitor {
    private static instance: ErrorMonitor;
    private diagnosticCollection: vscode.DiagnosticCollection;
    private errors: Map<string, TypeScriptError[]>;

    private constructor() {
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('typescript');
        this.errors = new Map();
    }

    public static getInstance(): ErrorMonitor {
        if (!ErrorMonitor.instance) {
            ErrorMonitor.instance = new ErrorMonitor();
        }
        return ErrorMonitor.instance;
    }

    public trackError(error: TypeScriptError): void {
        error.category = error.category || ts.DiagnosticCategory.Error;
        error.start = error.start || error.character;
        error.length = error.length || 1;
        error.messageText = error.messageText || error.message;

        const errors = this.errors.get(error.file) || [];
        errors.push(error);
        this.errors.set(error.file, errors);
        this.updateDiagnostics(error.file);
    }

    public clearErrors(file: string): void {
        this.errors.delete(file);
        this.diagnosticCollection.delete(vscode.Uri.file(file));
    }

    public getErrors(file: string): TypeScriptError[] {
        return this.errors.get(file) || [];
    }

    public getAllErrors(): Map<string, TypeScriptError[]> {
        return new Map(this.errors);
    }

    private updateDiagnostics(file: string): void {
        const errors = this.errors.get(file) || [];
        const diagnostics = errors.map(error => {
            const range = new vscode.Range(
                error.line - 1,
                error.character,
                error.line - 1,
                error.character + (error.length || 1)
            );
            return new vscode.Diagnostic(
                range,
                error.message,
                vscode.DiagnosticSeverity.Error
            );
        });
        this.diagnosticCollection.set(vscode.Uri.file(file), diagnostics);
    }

    public static parseCompilerOutput(output: string): TypeScriptError[] {
        const errors: TypeScriptError[] = [];
        const lines = output.split('\n');
        
        for (const line of lines) {
            const match = line.match(/^(.+)\((\d+),(\d+)\):\s+error\s+TS(\d+):\s+(.+)$/);
            if (match) {
                errors.push({
                    file: match[1],
                    line: parseInt(match[2], 10),
                    character: parseInt(match[3], 10),
                    code: parseInt(match[4], 10),
                    message: match[5],
                    category: ts.DiagnosticCategory.Error,
                    start: parseInt(match[3], 10),
                    length: 1,
                    messageText: match[5]
                });
            }
        }
        
        return errors;
    }
} 