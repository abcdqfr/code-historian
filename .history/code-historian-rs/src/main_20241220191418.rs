use clap::{Parser, Subcommand};
use code_historian::{Config, HistorianError, Result, PluginManager};
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional path to the repository (defaults to current directory)
    #[arg(global = true)]
    path: Option<PathBuf>,

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

    /// Analyze repository changes
    Analyze {
        /// Enable watch mode for continuous analysis
        #[arg(short, long)]
        watch: bool,

        /// Output format (json, markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Configure Code Historian
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List installed plugins
    List,

    /// Install a plugin
    Install {
        /// Name of the plugin
        name: String,
        /// Path to the plugin file
        path: PathBuf,
    },

    /// Remove a plugin
    Remove {
        /// Name of the plugin to remove
        name: String,
    },

    /// Show plugin information
    Info {
        /// Name of the plugin
        name: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Show current configuration
    Show,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Get the repository path
    let repo_path = cli.path.unwrap_or_else(|| std::env::current_dir().unwrap());

    // Create configuration
    let mut config = Config::new(&repo_path)?;

    // Initialize plugin manager
    let plugins_dir = config.plugins_dir()?;
    let mut plugin_manager = PluginManager::new(plugins_dir);
    plugin_manager.load_plugins()?;

    // Process commands
    match cli.command {
        Commands::Init { path } => {
            let target_path = path.unwrap_or(repo_path);
            init_repository(&target_path)?;
        }

        Commands::Analyze { watch, format } => {
            if watch {
                analyze_with_watch(&repo_path, &format, &plugin_manager).await?;
            } else {
                analyze_repository(&repo_path, &format, &plugin_manager)?;
            }
        }

        Commands::Plugin { command } => {
            match command {
                PluginCommands::List => {
                    list_plugins(&plugin_manager)?;
                }
                PluginCommands::Install { name, path } => {
                    install_plugin(&mut plugin_manager, &name, &path)?;
                }
                PluginCommands::Remove { name } => {
                    remove_plugin(&mut plugin_manager, &name)?;
                }
                PluginCommands::Info { name } => {
                    show_plugin_info(&plugin_manager, &name)?;
                }
            }
        }

        Commands::Config { command } => {
            match command {
                ConfigCommands::Get { key } => {
                    get_config(&config, &key)?;
                }
                ConfigCommands::Set { key, value } => {
                    set_config(&mut config, &key, &value)?;
                }
                ConfigCommands::Show => {
                    show_config(&config)?;
                }
            }
        }
    }

    Ok(())
}

fn init_repository(path: &Path) -> Result<()> {
    info!("Initializing repository at {:?}", path);
    let pb = create_progress_bar("Initializing");

    // Create necessary directories
    let historian_dir = path.join(".code-historian");
    std::fs::create_dir_all(&historian_dir)?;
    std::fs::create_dir_all(historian_dir.join("plugins"))?;
    std::fs::create_dir_all(historian_dir.join("cache"))?;
    std::fs::create_dir_all(historian_dir.join("reports"))?;

    // Create default configuration
    let config_path = historian_dir.join("config.toml");
    if !config_path.exists() {
        std::fs::write(config_path, include_str!("../templates/default_config.toml"))?;
    }

    pb.finish_with_message("Repository initialized successfully");
    Ok(())
}

async fn analyze_with_watch(path: &Path, format: &str, plugin_manager: &PluginManager) -> Result<()> {
    info!("Starting analysis in watch mode");
    let pb = create_progress_bar("Watching for changes");

    // TODO: Implement watch mode with notify crate

    Ok(())
}

fn analyze_repository(path: &Path, format: &str, plugin_manager: &PluginManager) -> Result<()> {
    info!("Analyzing repository at {:?}", path);
    let pb = create_progress_bar("Analyzing");

    // TODO: Implement repository analysis

    pb.finish_with_message("Analysis completed successfully");
    Ok(())
}

fn list_plugins(plugin_manager: &PluginManager) -> Result<()> {
    println!("\nInstalled plugins:");
    println!("------------------");

    for plugin in plugin_manager.get_plugins() {
        let manifest = plugin.manifest();
        println!(
            "{} v{} - {}",
            manifest.name,
            manifest.version,
            manifest.description
        );
    }

    Ok(())
}

fn install_plugin(plugin_manager: &mut PluginManager, name: &str, path: &Path) -> Result<()> {
    info!("Installing plugin {} from {:?}", name, path);
    let pb = create_progress_bar("Installing plugin");

    plugin_manager.install_plugin(name, path)?;

    pb.finish_with_message("Plugin installed successfully");
    Ok(())
}

fn remove_plugin(plugin_manager: &mut PluginManager, name: &str) -> Result<()> {
    info!("Removing plugin {}", name);
    let pb = create_progress_bar("Removing plugin");

    plugin_manager.remove_plugin(name)?;

    pb.finish_with_message("Plugin removed successfully");
    Ok(())
}

fn show_plugin_info(plugin_manager: &PluginManager, name: &str) -> Result<()> {
    if let Some(plugin) = plugin_manager.get_plugin(name) {
        let manifest = plugin.manifest();
        println!("\nPlugin Information:");
        println!("------------------");
        println!("Name: {}", manifest.name);
        println!("Version: {}", manifest.version);
        println!("Author: {}", manifest.author);
        println!("Description: {}", manifest.description);
        println!("\nSupported Languages:");
        for lang in &manifest.supported_languages {
            println!("- {}", lang);
        }
        if !manifest.dependencies.is_empty() {
            println!("\nDependencies:");
            for dep in &manifest.dependencies {
                println!("- {} {}", dep.name, dep.version_req);
            }
        }
    } else {
        return Err(HistorianError::PluginNotFound(name.to_string()));
    }

    Ok(())
}

fn get_config(config: &Config, key: &str) -> Result<()> {
    // TODO: Implement config get
    Ok(())
}

fn set_config(config: &mut Config, key: &str, value: &str) -> Result<()> {
    // TODO: Implement config set
    Ok(())
}

fn show_config(config: &Config) -> Result<()> {
    // TODO: Implement config show
    Ok(())
}

fn create_progress_bar(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(message.to_string());
    pb
}
