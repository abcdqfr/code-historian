# Code Historian Architecture

## Project Structure

```mermaid
graph TD
    subgraph Core
        A[Main] --> B[Analyzer]
        B --> C[Git Integration]
        B --> D[ML Engine]
        B --> E[Plugin System]
    end
    
    subgraph Output
        F[Report Generator]
        G[Visualizations]
        H[Timeline]
    end
    
    subgraph Analysis
        I[Category Detection]
        J[Pattern Recognition]
        K[Impact Scoring]
    end
    
    B --> F
    B --> G
    G --> H
    B --> I
    B --> J
    B --> K
```

## Directory Structure

```
code-historian-rs/
├── Cargo.toml               # Project dependencies and metadata
├── docs/                    # Documentation directory
│   ├── ARCHITECTURE.md      # This file
│   └── PROGRESS.md         # Development progress tracking
├── src/
│   ├── main.rs             # Entry point and CLI handling
│   ├── lib.rs              # Core types and traits
│   ├── analyzer.rs         # Code analysis engine
│   ├── git.rs              # Git repository interaction
│   ├── ml.rs               # Machine learning categorization
│   ├── plugin.rs           # Plugin system implementation
│   ├── report.rs           # Report generation
│   └── visualization.rs     # Data visualization
└── tests/                   # Integration tests
    └── integration/        # Test scenarios
```

## Component Interaction

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Analyzer
    participant Git
    participant ML
    participant Plugins
    participant Output

    User->>CLI: Run analysis
    CLI->>Analyzer: Configure analysis
    Analyzer->>Git: Fetch repository history
    loop Each Commit
        Git-->>Analyzer: Commit data
        Analyzer->>ML: Categorize changes
        Analyzer->>Plugins: Apply plugins
    end
    Analyzer->>Output: Generate reports
    Analyzer->>Output: Create visualizations
    Output-->>User: Final results
```

## Data Flow

```mermaid
flowchart LR
    A[Git Repository] --> B[Raw Commits]
    B --> C[Diff Analysis]
    C --> D[Categorization]
    D --> E[Pattern Detection]
    E --> F[Impact Scoring]
    F --> G[Report Generation]
    F --> H[Visualization]
    
    subgraph Processing
        C
        D
        E
        F
    end
    
    subgraph Output
        G
        H
    end
```

## Core Components

### Analyzer Engine
The analyzer engine is the heart of Code Historian, responsible for:
- Processing Git commits
- Categorizing changes
- Detecting patterns
- Calculating impact scores

```mermaid
classDiagram
    class Analyzer {
        +analyze(config: Config) Result<Analysis>
        +categorize(diff: str) Result<Vec<Category>>
        +calculate_impact(change: Change) f64
    }
    
    class CodeAnalyzer {
        -plugins: Vec<Box<dyn Plugin>>
        -ml_model: Option<ChangeClassifier>
        +new(plugins: Vec<Box<dyn Plugin>>, use_ml: bool) Result<Self>
        -analyze_commit(repo: &GitRepo, commit: &Commit) Result<Change>
        -calculate_metrics(changes: &[Change]) Metrics
        -detect_patterns(changes: &[Change]) Vec<Pattern>
    }
    
    Analyzer <|.. CodeAnalyzer
```

### Plugin System
The plugin system allows for extensible analysis capabilities:

```mermaid
classDiagram
    class Plugin {
        <<interface>>
        +name() str
        +analyze(context: AnalysisContext) Result<PluginResult>
        +supports_language(lang: str) bool
    }
    
    class SecurityAnalyzer {
        +name() str
        +analyze(context: AnalysisContext) Result<PluginResult>
        +supports_language(lang: str) bool
    }
    
    class PerformanceAnalyzer {
        +name() str
        +analyze(context: AnalysisContext) Result<PluginResult>
        +supports_language(lang: str) bool
    }
    
    Plugin <|.. SecurityAnalyzer
    Plugin <|.. PerformanceAnalyzer
```

## Configuration

The program supports various configuration options through both CLI and configuration files:

```rust
pub struct Config {
    pub repo_path: PathBuf,
    pub output_dir: PathBuf,
    pub plugins: Vec<String>,
    pub ml_enabled: bool,
    pub visualization_enabled: bool,
    pub recursive: bool,
    pub file_pattern: Option<String>,
}
```

## History Directory Detection

The program now includes functionality to detect and work with `.history` directories:

```mermaid
flowchart TD
    A[Start] --> B{Check .history}
    B -->|Found| C[Use existing history]
    B -->|Not found| D[Prompt user]
    D -->|Path provided| E[Validate path]
    D -->|No path| F[Create new .history]
    E -->|Valid| G[Use provided history]
    E -->|Invalid| D
    G --> H[Continue analysis]
    C --> H
    F --> H
```

## Output Formats

The program generates various output formats:

1. **Markdown Reports**
   - Summary statistics
   - Category distribution
   - Pattern analysis
   - Impact assessment

2. **Visualizations**
   - Timeline graphs
   - Category distribution charts
   - Impact score timelines

3. **JSON Data**
   - Machine-readable format
   - Integration-friendly
   - Complete analysis data

## Future Enhancements

```mermaid
gantt
    title Development Roadmap
    dateFormat YYYY-MM-DD
    
    section Core
    Enhanced ML Integration    :2024-01-01, 30d
    Advanced Pattern Detection :2024-02-01, 45d
    
    section Features
    Real-time Analysis        :2024-03-15, 30d
    Custom Plugin API         :2024-04-15, 45d
    
    section Integration
    CI/CD Integration        :2024-06-01, 30d
    IDE Plugins             :2024-07-01, 60d
```

## Performance Considerations

The program implements various optimizations:

1. **Parallel Processing**
   - Commit analysis
   - Pattern detection
   - Plugin execution

2. **Caching**
   - Git object caching
   - Analysis results
   - ML model predictions

3. **Memory Management**
   - Streaming large repositories
   - Efficient diff handling
   - Resource cleanup

## Error Handling

```mermaid
flowchart LR
    A[Error Source] --> B{Error Type}
    B -->|Git| C[Git Error Handler]
    B -->|Analysis| D[Analysis Error Handler]
    B -->|Plugin| E[Plugin Error Handler]
    B -->|IO| F[IO Error Handler]
    
    C --> G[Error Recovery]
    D --> G
    E --> G
    F --> G
    
    G -->|Recoverable| H[Continue]
    G -->|Fatal| I[Exit]
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Code style
- Testing requirements
- Documentation standards
- Pull request process 