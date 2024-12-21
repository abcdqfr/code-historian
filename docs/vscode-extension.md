# Code Historian VSCode Extension

## Overview

The Code Historian VSCode extension provides real-time code evolution analysis and visualization directly in your editor. It integrates seamlessly with the Code Historian server to provide insights about your codebase's development patterns, team collaboration, and impact analysis.

## Features

### 1. Real-time Dashboard
- Live analysis progress tracking
- Interactive visualizations
- Team activity monitoring
- Project comparison tools

### 2. Sidebar Integration
- Analysis status and progress
- Metrics overview
- Team collaboration insights

### 3. Command Palette
- `Code Historian: Analyze Repository` - Start analysis of current workspace
- `Code Historian: Show Dashboard` - Open the visualization dashboard
- `Code Historian: View Team Activity` - Show team collaboration metrics

## Installation

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Code Historian"
4. Click Install

Or install from the command line:
```bash
code --install-extension code-historian.code-historian-vscode
```

## Configuration

Access settings through VSCode's settings UI or `settings.json`:

```json
{
    "code-historian.serverUrl": "http://localhost:3000",
    "code-historian.autoAnalyze": false,
    "code-historian.refreshInterval": 30
}
```

### Available Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `serverUrl` | URL of the Code Historian server | `http://localhost:3000` |
| `autoAnalyze` | Automatically analyze repository on open | `false` |
| `refreshInterval` | Dashboard refresh interval in seconds | `30` |

## Usage

### Starting Analysis

1. Open a repository in VSCode
2. Open Command Palette (Ctrl+Shift+P)
3. Run "Code Historian: Analyze Repository"
4. View progress in the status bar

### Viewing Dashboard

1. Click the Code Historian icon in the activity bar
2. Select "Show Dashboard" from the tree view
3. Or use Command Palette: "Code Historian: Show Dashboard"

### Monitoring Progress

The extension provides multiple ways to track analysis progress:

1. **Status Bar**
   - Shows current analysis status
   - Click to open dashboard

2. **Activity Bar**
   - Code Historian icon with analysis count
   - Tree view with detailed status

3. **Notifications**
   - Analysis start/completion
   - Error notifications
   - Important events

### Team Collaboration

Monitor team activity through:

1. **Team View**
   - Active team members
   - Current projects
   - Recent activity

2. **Metrics View**
   - Individual contributions
   - Impact analysis
   - Code review stats

### Project Comparison

Compare multiple projects or branches:

1. Select projects in the dashboard
2. View side-by-side metrics
3. Analyze trends and patterns

## Best Practices

1. **Server Configuration**
   - Ensure server URL is correctly configured
   - Check server connectivity before analysis

2. **Performance**
   - Use auto-analyze selectively for large repos
   - Adjust refresh interval based on needs
   - Configure appropriate memory limits

3. **Team Workflow**
   - Keep dashboard open during development
   - Monitor impact scores for code reviews
   - Use metrics for team discussions

## Troubleshooting

### Common Issues

1. **Connection Failed**
   - Check server URL configuration
   - Verify server is running
   - Check network connectivity

2. **Analysis Stuck**
   - Check server logs
   - Verify repository access
   - Check available disk space

3. **Dashboard Not Updating**
   - Check refresh interval setting
   - Verify WebSocket connection
   - Try reloading window

### Error Messages

| Error | Solution |
|-------|----------|
| "Failed to connect to server" | Check server URL and status |
| "Analysis failed" | Check repository permissions |
| "Dashboard error" | Reload VSCode window |

## Support

- GitHub Issues: [code-historian/vscode-extension/issues](https://github.com/code-historian/vscode-extension/issues)
- Documentation: [GitHub Repository](https://github.com/abcdqfr/code-historian/docs/vscode)
- Community: [Discord](https://discord.gg/code-historian)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

## License

MIT License - see [LICENSE](../LICENSE) for details 