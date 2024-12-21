# Code Historian Help

## Overview

Code Historian is a tool for analyzing code evolution in software projects. It tracks changes, identifies patterns, and generates insightful visualizations to help understand how code evolves over time.

## Basic Usage

```bash
# Initialize tracking in current directory
code-historian init

# Analyze current directory
code-historian

# Analyze specific path
code-historian analyze /path/to/repo

# Watch for changes
code-historian watch
```

## Commands

### Core Commands

#### `code-historian [path]`
Analyze repository at current directory or specified path.
```bash
code-historian              # Analyze current directory
code-historian /path/repo   # Analyze specific repository
```

#### `code-historian init [options]`
Initialize repository for tracking.
```bash
code-historian init                                # Initialize empty
code-historian init --import-history /path/history # Import existing history
```

#### `code-historian analyze [options]`
Perform detailed analysis with specific options.
```bash
code-historian analyze                  # Basic analysis
code-historian analyze --ml            # Use ML categorization
code-historian analyze --visualize     # Generate visualizations
code-historian analyze --recursive     # Analyze subdirectories
code-historian analyze --pattern "*.rs" # Analyze specific files
```

#### `code-historian watch [options]`
Monitor repository for changes in real-time.
```bash
code-historian watch              # Basic monitoring
code-historian watch --visualize  # With visualization updates
code-historian watch --debounce 5 # Custom update interval
```

### Configuration Commands

#### `code-historian config`
Manage configuration settings.
```bash
code-historian config init           # Initialize configuration
code-historian config get <key>      # Get configuration value
code-historian config set <key> <value> # Set configuration value
```

### Plugin Commands

#### `code-historian plugin`
Manage analysis plugins.
```bash
code-historian plugin list          # List available plugins
code-historian plugin install <name> # Install plugin
code-historian plugin remove <name>  # Remove plugin
```

## Configuration

### Directory Structure
```
.code-historian/           # Project-specific data
├── config.toml           # Project configuration
├── plugins/             # Project plugins
└── cache/              # Analysis cache

~/.config/code-historian/ # User configuration
/etc/code-historian/      # System configuration
```

### Configuration File (config.toml)
```toml
# Output settings
default_output_dir = "docs/code-history"
preferred_plugins = ["security", "performance"]
default_ml_enabled = true

[visualization]
theme = "modern"
timeline_style = "compact"
chart_style = "dark"
```

## Output Formats

### 1. Reports
Generated in `docs/code-history/` by default:
```
docs/code-history/
├── REPORT.md           # Main analysis report
├── timeline.png        # Visual timeline
├── categories.png      # Category distribution
├── impact.png         # Impact analysis
└── data.json          # Raw analysis data
```

### 2. Visualizations
- Timeline graphs showing code evolution
- Category distribution charts
- Impact analysis visualizations
- Pattern relationship graphs

### 3. Data Formats
- Markdown for human reading
- JSON for machine processing
- HTML for web viewing

## Analysis Categories

### 1. Structural Changes
- Architecture modifications
- Component relationships
- Code organization

### 2. Functional Changes
- API modifications
- Logic updates
- Data handling

### 3. Quality Changes
- Documentation
- Testing
- Error handling
- Performance
- Security

### 4. Maintenance Changes
- Refactoring
- Dependencies
- Configuration

## Examples

### Basic Analysis
```bash
# Initialize and analyze current project
cd your-project
code-historian init
code-historian

# Generate visualizations
code-historian analyze --visualize

# Watch for changes
code-historian watch
```

### Advanced Usage
```bash
# ML-based analysis with custom output
code-historian analyze \
  --ml \
  --visualize \
  --output custom/path \
  --pattern "*.rs"

# Real-time monitoring with custom settings
code-historian watch \
  --visualize \
  --debounce 5
```

### Plugin Usage
```bash
# Install and use specific plugins
code-historian plugin install security-analyzer
code-historian analyze --plugins "security,performance"
```

## Environment Variables

- `CODE_HISTORIAN_CONFIG`: Path to custom config file
- `CODE_HISTORIAN_PLUGINS`: Additional plugin directories
- `CODE_HISTORIAN_OUTPUT`: Default output directory
- `CODE_HISTORIAN_ML_MODEL`: Custom ML model path

## Exit Codes

- 0: Success
- 1: General error
- 2: Configuration error
- 3: Analysis error
- 4: Plugin error
- 5: IO error

## See Also

- [CORE_PURPOSE.md](CORE_PURPOSE.md) - Core purpose and evolution
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture
- [PLUGINS.md](PLUGINS.md) - Plugin development guide 