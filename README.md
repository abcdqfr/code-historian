# Code Historian

A powerful tool for analyzing the evolution of code in software projects. Code Historian tracks changes in source files, categorizes them according to common software patterns, and generates detailed reports with visual timelines.

## Features

- **Flexible File Analysis**: Analyze specific files or use pattern matching to process multiple files
- **Recursive Directory Support**: Search for files in subdirectories
- **Smart Change Detection**: Automatically categorizes changes into 15 different categories:
  - Architecture (structural changes)
  - API (interface changes)
  - Logic (flow modifications)
  - Data (structure changes)
  - Error Handling
  - Logging
  - Documentation
  - Testing
  - Performance
  - Security
  - Refactoring
  - Dependencies
  - Configuration
  - UI/UX
  - Accessibility

- **Visual Timelines**: Generate visual representations of code evolution using Graphviz
- **Detailed Statistics**: Track lines added, removed, and net changes
- **Git-Style Diffs**: View changes in familiar diff format
- **Markdown Reports**: Generate comprehensive markdown reports with links to visual timelines

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/code-historian.git
cd code-historian
```

2. Run the installation script:
```bash
./install.sh
```

The installer will:
- Check for required dependencies (Python 3, pip, Graphviz)
- Install necessary Python packages
- Set up the command-line tool

### Dependencies

- Python 3.6+
- Graphviz
- Python graphviz package (installed automatically)

For Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install python3 python3-pip graphviz
```

For Fedora:
```bash
sudo dnf install python3 python3-pip graphviz
```

For macOS:
```bash
brew install python3 graphviz
```

## Usage

### Basic Usage

Analyze specific files:
```bash
code-historian --files myfile,otherfile --ext py
```

Analyze files with pattern matching:
```bash
code-historian --recursive --pattern "*.py"
```

Generate visual timelines:
```bash
code-historian --files myfile --ext py --timeline
```

### Command Line Options

```
Options:
  -h, --help                Show this help message
  --history-dir DIR         Set history directory (default: .history)
  --source-dir DIR          Set source directory (default: src)
  --output-dir DIR          Set output directory (default: docs/history)
  --files FILE1,FILE2,...   Specify files to analyze (without extension)
  --ext EXTENSION           File extension to analyze (e.g., py, js, cpp)
  --recursive               Recursively search in subdirectories
  --pattern PATTERN         File pattern to match (e.g., '*.py', 'test_*.js')
  --timeline               Generate visual timeline using Graphviz
```

### Example Workflow

1. Track changes in Python files:
```bash
code-historian --recursive --pattern "*.py" --timeline
```

2. Analyze specific components:
```bash
code-historian --files coordinator,sensor --ext py --timeline
```

3. View results:
- Check `docs/history/SUMMARY.md` for an overview
- Browse individual change reports in `docs/history/`
- View visual timelines in PNG format

## Output Format

The tool generates:
1. A markdown file for each analyzed file with:
   - Detailed change history
   - Git-style diffs
   - Change statistics
   - Categorized changes
2. Visual timeline (when `--timeline` is used) showing:
   - Change sequence
   - Top categories per change
   - Lines added/removed
3. A summary markdown file linking to all reports and timelines

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.