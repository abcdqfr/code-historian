use clap::{Parser, Subcommand};
use code_historian::{
    Analysis, Config, HistorianError, ReportGenerator, Result,
    analyzer::Analyzer,
    config::load_config,
    plugin::PluginManager,
};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Code Historian project
    Init {
        /// Path to initialize (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Analyze repository and generate reports
    Analyze {
        /// Path to analyze (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output format (markdown, json, or html)
        #[arg(short, long, default_value = "html")]
        format: String,

        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Watch repository for changes
    Watch {
        /// Path to watch (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List installed plugins
    List,
    /// Install a plugin
    Install {
        /// Plugin path or name
        path: PathBuf,
    },
    /// Remove a plugin
    Remove {
        /// Plugin name
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration
    let config = load_config()?;

    // Initialize plugin manager
    let plugin_manager = PluginManager::new(&config)?;

    match cli.command {
        Commands::Init { path } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            initialize_project(&path)?;
        }

        Commands::Analyze { path, format, output } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let output = output.unwrap_or_else(|| path.join(".code-historian/reports"));

            // Create analyzer
            let analyzer = Analyzer::new(&config, &plugin_manager)?;

            // Perform analysis
            info!("Analyzing repository at {}", path.display());
            let analysis = analyzer.analyze(&path)?;

            // Generate reports
            generate_reports(&analysis, &format, &output)?;
        }

        Commands::Watch { path } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            watch_repository(&path, &config, &plugin_manager).await?;
        }

        Commands::Config { action } => {
            match action {
                ConfigCommands::Get { key } => {
                    if let Some(value) = config.get(&key) {
                        println!("{}: {}", key, value);
                    } else {
                        println!("Configuration key '{}' not found", key);
                    }
                }
                ConfigCommands::Set { key, value } => {
                    config.set(&key, &value)?;
                    println!("Set {} = {}", key, value);
                }
            }
        }

        Commands::Plugin { action } => {
            match action {
                PluginCommands::List => {
                    for plugin in plugin_manager.list_plugins()? {
                        println!("{} ({})", plugin.name, plugin.version);
                    }
                }
                PluginCommands::Install { path } => {
                    plugin_manager.install_plugin(&path)?;
                    println!("Plugin installed successfully");
                }
                PluginCommands::Remove { name } => {
                    plugin_manager.remove_plugin(&name)?;
                    println!("Plugin removed successfully");
                }
            }
        }
    }

    Ok(())
}

fn initialize_project(path: &Path) -> Result<()> {
    info!("Initializing Code Historian project at {}", path.display());

    // Create project directory structure
    let historian_dir = path.join(".code-historian");
    std::fs::create_dir_all(&historian_dir)?;
    std::fs::create_dir_all(historian_dir.join("reports"))?;
    std::fs::create_dir_all(historian_dir.join("cache"))?;
    std::fs::create_dir_all(historian_dir.join("plugins"))?;

    // Create default configuration
    let config_path = historian_dir.join("config.toml");
    if !config_path.exists() {
        std::fs::write(
            &config_path,
            include_str!("../templates/default_config.toml"),
        )?;
    }

    info!("Project initialized successfully");
    Ok(())
}

fn generate_reports(analysis: &Analysis, format: &str, output_dir: &Path) -> Result<()> {
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)?;

    match format.to_lowercase().as_str() {
        "markdown" => {
            let report_path = output_dir.join("report.md");
            let generator = ReportGenerator::new()?;
            generator.generate_markdown(analysis, &report_path)?;
            info!("Generated Markdown report at {}", report_path.display());
        }
        "json" => {
            let report_path = output_dir.join("report.json");
            let generator = ReportGenerator::new()?;
            generator.generate_json(analysis, &report_path)?;
            info!("Generated JSON report at {}", report_path.display());
        }
        "html" => {
            let report_path = output_dir.join("report.html");
            let generator = ReportGenerator::new()?;
            generator.generate_html(analysis, &report_path)?;
            info!("Generated HTML report at {}", report_path.display());
        }
        _ => {
            return Err(HistorianError::InvalidArgument(format!(
                "Unsupported output format: {}",
                format
            )));
        }
    }

    Ok(())
}

async fn watch_repository(path: &Path, config: &Config, plugin_manager: &PluginManager) -> Result<()> {
    use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    info!("Watching repository at {}", path.display());

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        NotifyConfig::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    let analyzer = Analyzer::new(config, plugin_manager)?;
    let mut last_analysis = None;

    loop {
        match rx.recv() {
            Ok(_) => {
                // Debounce events
                tokio::time::sleep(Duration::from_secs(1)).await;
                while rx.try_recv().is_ok() {}

                info!("Changes detected, analyzing repository...");
                match analyzer.analyze(path) {
                    Ok(analysis) => {
                        if let Some(prev) = &last_analysis {
                            if !analysis.has_significant_changes(prev) {
                                continue;
                            }
                        }

                        let output_dir = path.join(".code-historian/reports");
                        if let Err(e) = generate_reports(&analysis, "html", &output_dir) {
                            warn!("Failed to generate reports: {}", e);
                        }

                        last_analysis = Some(analysis);
                    }
                    Err(e) => warn!("Analysis failed: {}", e),
                }
            }
            Err(e) => {
                warn!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use predicates::prelude::*;

    #[test]
    fn test_initialize_project() {
        let temp = assert_fs::TempDir::new().unwrap();
        initialize_project(temp.path()).unwrap();

        temp.child(".code-historian").assert(predicate::path::exists());
        temp.child(".code-historian/reports").assert(predicate::path::exists());
        temp.child(".code-historian/cache").assert(predicate::path::exists());
        temp.child(".code-historian/plugins").assert(predicate::path::exists());
        temp.child(".code-historian/config.toml").assert(predicate::path::exists());
    }

    #[test]
    fn test_generate_reports() {
        let temp = assert_fs::TempDir::new().unwrap();
        let output_dir = temp.path().join("reports");

        let analysis = Analysis::default();

        generate_reports(&analysis, "html", &output_dir).unwrap();
        temp.child("reports/report.html").assert(predicate::path::exists());

        generate_reports(&analysis, "markdown", &output_dir).unwrap();
        temp.child("reports/report.md").assert(predicate::path::exists());

        generate_reports(&analysis, "json", &output_dir).unwrap();
        temp.child("reports/report.json").assert(predicate::path::exists());
    }
}
