# Code Historian

A powerful tool for analyzing code evolution and creating a comprehensive knowledge base of changes. Originally developed during an AI-assisted refactoring project, Code Historian helps developers understand, document, and learn from their code's history.

## Why Code Historian?

When refactoring or maintaining complex codebases, understanding the evolution of code is crucial. Code Historian:
- Tracks the reasoning behind changes
- Identifies patterns in development
- Helps prevent regression by maintaining knowledge
- Makes code evolution visible and understandable
- Assists in knowledge transfer between team members

## Core Dependencies

### Required Extensions
- **VSCode Local History**: Creates timestamped backups of file changes
  - Provides the `.history` directory structure
  - Maintains chronological file versions
  - Independent of git commits

### Optional Integration
- **Cursor AI**: While not required, Code Historian was developed with AI assistance
  - Can enhance change categorization
  - Helps in pattern recognition
  - Useful for automated improvement suggestions

### System Requirements
- Bash shell environment
- Core Unix tools:
  - `find`: For file discovery
  - `diff`: For change comparison
  - `grep`: For pattern matching
  - `date`: For timestamp processing

## Features in Detail

### 1. Change Timeline Generation
```markdown
### Changes at 2024-12-18 14:05:11
Categories: [Type Safety] [Error Handling] [Documentation]

```diff
@@ -83,6 +83,8 @@
     self._device_init_delay = 2  # seconds
     self._add_entities_callback: Optional[AddEntitiesCallback] = None
     self._entity_cleanup_callbacks: Dict[str, Callable] = {}
+    self._last_error_time = 0.0  # Initialize error time tracking
+    self._error_cooldown = 60.0  # 60 second cooldown between error handling
```
- Added error rate limiting
- Enhanced type safety with Optional and Dict types
```

### 2. Smart Change Categorization
- **Class Structure**: Class definitions, inheritance changes
- **Async Methods**: Asynchronous patterns, coroutines
- **Logging**: Debugging, monitoring, metrics
- **Error Handling**: Exception management, recovery
- **Properties**: Data access patterns
- **Type Safety**: Type hints, validation
- **Resource Management**: Cleanup, memory management
- **Documentation**: Inline docs, comments

### 3. Impact Analysis
- Tracks related changes across files
- Identifies potential breaking changes
- Highlights dependency modifications
- Shows test coverage impact

## Example Output

### Change Analysis
```markdown
## Key Improvements in coordinator.py

1. Type Safety Enhancements
   - Added TypedDict for structured data
   - Implemented Optional for nullable fields
   - Enhanced function signatures

2. Error Handling Improvements
   - Added rate limiting for errors
   - Implemented exponential backoff
   - Enhanced recovery mechanisms

3. Resource Management
   - Improved process cleanup
   - Added async shutdown handling
   - Enhanced memory management
```

### Pattern Recognition
```markdown
Common Change Patterns:
1. Error handling often follows logging improvements
2. Type safety changes cluster with class structure updates
3. Documentation updates correlate with API changes
4. Resource management changes trigger test updates
```

## Usage Examples

### Basic Analysis
```bash
./analyze_history.sh --files model,view,controller
```

### Custom Directories
```bash
./analyze_history.sh --history-dir .vscode/history --source-dir app --output-dir reports
```

### Project-Specific Analysis
```bash
./analyze_history.sh --files "coordinator,sensor,const" --output-dir "docs/changes"
```

## Integration with Development Workflow

### 1. Code Review Support
- Generate change reports before reviews
- Track evolution of complex features
- Document architectural decisions

### 2. Documentation Generation
- Automatic changelog creation
- API evolution tracking
- Breaking change detection

### 3. Knowledge Management
- Preserve context across team changes
- Track feature development history
- Document bug fixes and solutions

## Future Enhancements

### Immediate Goals
1. Git integration for hybrid history
2. Change impact scoring system
3. Regression detection algorithms
4. Code quality metrics tracking

### Long-term Vision
5. Visual timeline generation
6. AI-powered improvement suggestions
7. Multi-language support
8. Custom category definitions
9. Interactive analysis mode
10. Advanced filtering and search

## Contributing

Code Historian welcomes contributions! Areas of interest:
1. Additional change categories
2. Language-specific enhancements
3. Integration with CI/CD
4. Visualization improvements
5. AI/ML integration for pattern detection

## License

MIT License - Feel free to use, modify, and share!