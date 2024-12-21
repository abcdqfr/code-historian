# TypeScript Error Auto-Fixing System

## Overview

This system automatically detects and fixes TypeScript errors in real-time. It's built as a VS Code extension that monitors your TypeScript files and provides automatic fixes for common errors.

## Features

1. **Real-time Error Detection**
   - Monitors all TypeScript files in your workspace
   - Detects errors as you type
   - Shows errors in the Problems panel

2. **Automatic Error Fixing**
   - Missing imports
   - Type definition installation
   - Property mismatches
   - Type casting issues
   - Missing properties
   - Argument count mismatches

3. **Smart Fix Verification**
   - Verifies fixes actually worked
   - Shows success messages
   - Retries if needed

## Installation

1. Clone the repository:
   ```bash
   git clone <your-repo>
   cd extensions/vscode
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Build the extension:
   ```bash
   npm run compile
   ```

4. Start the TypeScript watcher:
   ```bash
   npm run watch
   ```

## Usage

### 1. Command Palette
1. Press `Cmd/Ctrl+Shift+P`
2. Type "Fix TypeScript Errors"
3. Press Enter

### 2. Context Menu
- Right-click in any TypeScript file
- Select "Fix TypeScript Errors"

### 3. Auto-Fix Mode
1. Open VS Code settings
2. Enable "Code Historian: Auto Fix TypeScript Errors"
3. Errors will be fixed automatically as you type

## Error Types Handled

| Error Code | Description | Fix Type |
|------------|-------------|----------|
| 2304 | Cannot find name 'X' | Auto-import |
| 7016 | Could not find declaration file | Install @types package |
| 2339 | Property does not exist | Add property to interface |
| 2322 | Type mismatch | Add type cast |
| 2741 | Property is missing | Add missing property |
| 2554 | Wrong number of arguments | Add placeholder arguments |

## Examples

### 1. Missing Import
```typescript
// Before
const response = await axios.get('/api');

// After (automatically added)
import axios from 'axios';
const response = await axios.get('/api');
```

### 2. Missing Property
```typescript
// Before
interface User {
    name: string;
}
const user: User = { name: "John", age: 30 };

// After (automatically fixed)
interface User {
    name: string;
    age: number;  // Added automatically
}
const user: User = { name: "John", age: 30 };
```

### 3. Type Mismatch
```typescript
// Before
const value: string = 42;

// After (automatically fixed)
const value: string = String(42);
```

## Configuration

In your VS Code settings:

```json
{
    "codeHistorian.autoFixTypeScriptErrors": true,
    "codeHistorian.enableNotifications": true
}
```

## Architecture

### 1. Error Monitor (`errorMonitor.ts`)
```typescript
class ErrorMonitor {
    // Tracks TypeScript errors in real-time
    public trackError(error: TypeScriptError): void;
    // Clears errors for a file
    public clearErrors(file: string): void;
    // Gets all current errors
    public getAllErrors(): Map<string, TypeScriptError[]>;
}
```

### 2. Error Fixer (`errorFixer.ts`)
```typescript
class ErrorFixer {
    // Fixes all current errors
    public async fixAllErrors(): Promise<void>;
    // Verifies if a fix worked
    private async verifyFix(file: string, error: ts.Diagnostic): Promise<boolean>;
    // Gets suggested fixes for an error
    private async getSuggestedFixes(diagnostic: DiagnosticWithFile): Promise<FixAction[]>;
}
```

## Best Practices

1. **Before Starting**
   - Ensure all dependencies are installed
   - Configure TypeScript properly
   - Set up your `tsconfig.json`

2. **During Development**
   - Keep the Problems panel open
   - Watch for fix notifications
   - Review automatic fixes

3. **Troubleshooting**
   - Check the Output panel for detailed logs
   - Verify TypeScript version compatibility
   - Check for conflicting extensions

## Common Issues

1. **Fix Not Working**
   - Ensure the file is saved
   - Check if the error is supported
   - Try running the fix command again

2. **Missing Type Definitions**
   - Run `npm install` again
   - Check your package.json
   - Verify npm registry access

3. **Performance Issues**
   - Reduce the number of open files
   - Increase VS Code memory limit
   - Disable other type checking extensions

## Contributing

1. **Adding New Error Types**
```typescript
private async getSuggestedFixes(diagnostic: DiagnosticWithFile): Promise<FixAction[]> {
    switch (diagnostic.code) {
        case YOUR_ERROR_CODE:
            return this.createYourCustomFix(diagnostic);
    }
}
```

2. **Testing**
```bash
npm run test
```

3. **Building**
```bash
npm run compile
```

## Resources

- [TypeScript Error Codes](https://github.com/microsoft/TypeScript/blob/main/src/compiler/diagnosticMessages.json)
- [VS Code Extension API](https://code.visualstudio.com/api)
- [TypeScript Compiler API](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API)

## Support

- File issues on GitHub
- Join our Discord community
- Check the FAQ

## License

MIT License - See LICENSE file for details 