use clap::{Parser, Subcommand};
use code_historian::{
    analyzer::CodeAnalyzer,
    Config,
    config::HistorianConfig,
    interactive::InteractiveConfig,
    visualization::Visualizer,
    watch::WatchManager,
    Result, HistorianError,
};
use std::path::{PathBuf, Path};
use tracing::{info, warn, error};
use std::io::{self, Write};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional path to analyze (defaults to current directory)
    #[arg(global = true)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to config file
    #[arg(short = 'C', long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize repository for tracking
    Init {
        /// Import existing history
        #[arg(long)]
        import_history: Option<PathBuf>,
    },
    
    /// Analyze repository
    Analyze {
        /// Enable machine learning-based categorization
        #[arg(short, long)]
        ml: bool,

        /// Enable visualization generation
        #[arg(short, long)]
        visualize: bool,

        /// Search recursively in subdirectories
        #[arg(short, long)]
        recursive: bool,

        /// File pattern to match (e.g., "*.rs", "*.py")
        #[arg(short, long)]
        pattern: Option<String>,

        /// Plugins to use (comma-separated)
        #[arg(short, long)]
        plugins: Option<String>,

        /// Custom output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Watch repository for changes
    Watch {
        /// Watch mode debounce duration in seconds
        #[arg(long, default_value = "2")]
        debounce: u64,
        
        /// Enable visualization updates
        #[arg(short, long)]
        visualize: bool,
    },
    
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Initialize configuration
    Init,
    /// Get configuration value
    Get {
        key: String,
    },
    /// Set configuration value
    Set {
        key: String,
        value: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List available plugins
    List,
    /// Install plugin
    Install {
        name: String,
    },
    /// Remove plugin
    Remove {
        name: String,
    },
}

struct ProgressManager {
    multi_progress: MultiProgress,
    scan_progress: ProgressBar,
    analysis_progress: ProgressBar,
    viz_progress: ProgressBar,
}

impl ProgressManager {
    fn new() -> Self {
        let multi_progress = MultiProgress::new();
        
        let scan_progress = multi_progress.add(ProgressBar::new_spinner());
        scan_progress.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        
        let analysis_progress = multi_progress.add(ProgressBar::new(100));
        analysis_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        
        let viz_progress = multi_progress.add(ProgressBar::new(3));
        viz_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );

        Self {
            multi_progress,
            scan_progress,
            analysis_progress,
            viz_progress,
        }
    }

    fn start_scanning(&self) {
        self.scan_progress.set_message("Scanning repository...");
        self.scan_progress.enable_steady_tick(100);
    }

    fn finish_scanning(&self) {
        self.scan_progress.finish_with_message("Repository scan complete");
    }

    fn start_analysis(&self, total: u64) {
        self.analysis_progress.set_length(total);
        self.analysis_progress.set_message("Analyzing commits...");
    }

    fn inc_analysis(&self) {
        self.analysis_progress.inc(1);
    }

    fn finish_analysis(&self) {
        self.analysis_progress.finish_with_message("Analysis complete");
    }

    fn start_visualization(&self) {
        self.viz_progress.set_message("Generating visualizations...");
    }

    fn inc_visualization(&self) {
        self.viz_progress.inc(1);
    }

    fn finish_visualization(&self) {
        self.viz_progress.finish_with_message("Visualizations complete");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Get repository path (current directory if not specified)
    let repo_path = cli.path.unwrap_or_else(|| std::env::current_dir().unwrap());

    // Load configuration
    let config = if let Some(config_path) = cli.config {
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content)
            .map_err(|e| HistorianError::Analysis(format!("Failed to parse config: {}", e)))?
    } else {
        HistorianConfig::load()?
    };

    match cli.command.unwrap_or(Commands::Analyze { 
        ml: false,
        visualize: false,
        recursive: false,
        pattern: None,
        plugins: None,
        output: None,
    }) {
        Commands::Init { import_history } => {
            initialize_repository(&repo_path, import_history)?;
        }
        
        Commands::Analyze { 
            ml,
            visualize,
            recursive,
            pattern,
            plugins,
            output,
        } => {
            analyze_repository(
                &repo_path,
                &config,
                ml,
                visualize,
                recursive,
                pattern,
                plugins,
                output,
            ).await?;
        }
        
        Commands::Watch { debounce, visualize } => {
            watch_repository(&repo_path, &config, debounce, visualize).await?;
        }
        
        Commands::Config { command } => {
            handle_config_command(command)?;
        }
        
        Commands::Plugin { command } => {
            handle_plugin_command(command)?;
        }
    }

    Ok(())
}

fn initialize_repository(repo_path: &Path, import_history: Option<PathBuf>) -> Result<()> {
    let historian_dir = repo_path.join(".code-historian");
    
    if historian_dir.exists() {
        return Err(HistorianError::Analysis(
            "Repository is already initialized".to_string()
        ));
    }

    // Create directory structure
    std::fs::create_dir_all(&historian_dir)?;
    std::fs::create_dir_all(historian_dir.join("plugins"))?;
    std::fs::create_dir_all(historian_dir.join("cache"))?;
    
    // Create default config
    let config = HistorianConfig::default();
    let config_path = historian_dir.join("config.toml");
    let content = toml::to_string_pretty(&config)
        .map_err(|e| HistorianError::Analysis(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(config_path, content)?;

    info!("Initialized empty Code Historian repository in {}", historian_dir.display());

    // Import history if specified
    if let Some(import_path) = import_history {
        info!("Importing history from {}", import_path.display());
        // TODO: Implement history import
    }

    Ok(())
}

async fn analyze_repository(
    repo_path: &Path,
    config: &HistorianConfig,
    ml_enabled: bool,
    visualization_enabled: bool,
    recursive: bool,
    pattern: Option<String>,
    plugins: Option<String>,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    // Create CLI config
    let cli_config = Config::new(repo_path.to_path_buf())
        .with_output_dir(output_dir.unwrap_or_else(|| PathBuf::from("docs/history")))
        .with_plugins(plugins
            .map(|p| p.split(',').map(String::from).collect())
            .unwrap_or_else(Vec::new))
        .with_ml(ml_enabled)
        .with_visualization(visualization_enabled)
        .with_recursive(recursive)
        .with_pattern(pattern.unwrap_or_default());

    // Merge configs
    let config = config.merge_with_cli(cli_config);

    // Initialize progress manager
    let progress = ProgressManager::new();

    // Create analyzer
    let analyzer = CodeAnalyzer::new(vec![], config.ml_enabled.unwrap_or(false))?;

    // Perform analysis
    info!("Starting code analysis...");
    progress.start_scanning();
    progress.finish_scanning();
    progress.start_analysis(100); // TODO: Get actual commit count
    let analysis = analyzer.analyze(&config)?;
    progress.finish_analysis();

    // Generate visualizations if enabled
    if config.visualization_enabled {
        generate_visualizations(&config, &analysis, &progress)?;
    }

    // Generate report
    generate_report(config.output_dir.as_ref().unwrap(), &analysis)?;

    info!("Analysis complete! Results saved in {}", config.output_dir.unwrap().display());
    Ok(())
}

async fn watch_repository(
    repo_path: &Path,
    config: &HistorianConfig,
    debounce: u64,
    visualize: bool,
) -> Result<()> {
    let cli_config = Config::new(repo_path.to_path_buf())
        .with_visualization(visualize);

    let config = config.merge_with_cli(cli_config);
    let analyzer = CodeAnalyzer::new(vec![], config.ml_enabled.unwrap_or(false))?;

    let watch_manager = WatchManager::new(config, analyzer)
        .with_debounce(Duration::from_secs(debounce));
        
    watch_manager.watch().await
}

fn handle_config_command(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Init => {
            // TODO: Implement config initialization
            Ok(())
        }
        ConfigCommands::Get { key } => {
            // TODO: Implement config get
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            // TODO: Implement config set
            Ok(())
        }
    }
}

fn handle_plugin_command(command: PluginCommands) -> Result<()> {
    match command {
        PluginCommands::List => {
            // TODO: Implement plugin list
            Ok(())
        }
        PluginCommands::Install { name } => {
            // TODO: Implement plugin install
            Ok(())
        }
        PluginCommands::Remove { name } => {
            // TODO: Implement plugin remove
            Ok(())
        }
    }
}

fn generate_visualizations(
    config: &Config,
    analysis: &code_historian::Analysis,
    progress: &ProgressManager,
) -> Result<()> {
    info!("Generating visualizations...");
    progress.start_visualization();
    
    let visualizer = Visualizer::new(config.output_dir.as_ref().unwrap())?;

    // Generate timeline
    match visualizer.generate_timeline(analysis) {
        Ok(path) => {
            info!("Timeline generated: {}", path);
            progress.inc_visualization();
        },
        Err(e) => warn!("Failed to generate timeline: {}", e),
    }

    // Generate category distribution
    match visualizer.generate_category_distribution(analysis) {
        Ok(path) => {
            info!("Category distribution generated: {}", path);
            progress.inc_visualization();
        },
        Err(e) => warn!("Failed to generate category distribution: {}", e),
    }

    // Generate impact timeline
    match visualizer.generate_impact_timeline(&analysis.changes) {
        Ok(path) => {
            info!("Impact timeline generated: {}", path);
            progress.inc_visualization();
        },
        Err(e) => warn!("Failed to generate impact timeline: {}", e),
    }

    progress.finish_visualization();
    Ok(())
}

fn generate_report(output_dir: &PathBuf, analysis: &code_historian::Analysis) -> Result<()> {
    let report_path = output_dir.join("REPORT.md");
    let mut report = String::new();

    // Add header
    report.push_str("# Code Analysis Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", chrono::Local::now()));

    // Add summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Total commits analyzed: {}\n", analysis.metrics.total_commits));
    report.push_str(&format!("- Total changes: {}\n", analysis.metrics.total_changes));
    report.push_str(&format!("- Lines added: {}\n", analysis.metrics.lines_added));
    report.push_str(&format!("- Lines removed: {}\n", analysis.metrics.lines_removed));
    report.push_str("\n");

    // Add category distribution
    report.push_str("## Category Distribution\n\n");
    report.push_str("| Category | Count | Percentage |\n");
    report.push_str("|----------|--------|------------|\n");

    let total = analysis.metrics.total_changes as f64;
    for (category, count) in &analysis.metrics.category_distribution {
        let percentage = (*count as f64 / total) * 100.0;
        report.push_str(&format!(
            "| {:?} | {} | {:.1}% |\n",
            category, count, percentage
        ));
    }
    report.push_str("\n");

    // Add patterns section
    report.push_str("## Detected Patterns\n\n");
    for pattern in &analysis.patterns {
        report.push_str(&format!("### {}\n", pattern.name));
        report.push_str(&format!("- Description: {}\n", pattern.description));
        report.push_str(&format!("- Occurrences: {}\n", pattern.occurrences));
        report.push_str(&format!("- Impact Score: {:.2}\n", pattern.impact));
        report.push_str("\n");
    }

    // Add visualizations section if they exist
    report.push_str("## Visualizations\n\n");
    if output_dir.join("timeline.png").exists() {
        report.push_str("- [Timeline](timeline.png)\n");
    }
    if output_dir.join("category_distribution.png").exists() {
        report.push_str("- [Category Distribution](category_distribution.png)\n");
    }
    if output_dir.join("impact_timeline.png").exists() {
        report.push_str("- [Impact Timeline](impact_timeline.png)\n");
    }
    report.push_str("\n");

    // Write report to file
    std::fs::write(report_path, report)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_report_generation() {
        let output_dir = PathBuf::from("/tmp/test_report");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut distribution = HashMap::new();
        distribution.insert(code_historian::Category::Architecture, 10);
        distribution.insert(code_historian::Category::Security, 5);

        let analysis = code_historian::Analysis {
            changes: vec![],
            metrics: code_historian::Metrics {
                total_commits: 15,
                total_changes: 15,
                lines_added: 100,
                lines_removed: 50,
                category_distribution: distribution,
            },
            patterns: vec![],
        };

        let result = generate_report(&output_dir, &analysis);
        assert!(result.is_ok());
        assert!(output_dir.join("REPORT.md").exists());
    }
}
