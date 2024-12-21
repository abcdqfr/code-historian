use clap::Parser;
use code_historian::{
    analyzer::CodeAnalyzer,
    Config,
    visualization::Visualizer,
    Result, HistorianError,
};
use std::path::{PathBuf, Path};
use tracing::{info, warn, error};
use std::io::{self, Write};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the repository to analyze
    #[arg(short, long)]
    repo: PathBuf,

    /// Output directory for reports and visualizations
    #[arg(short, long, default_value = "docs/history")]
    output: PathBuf,

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

    /// Path to history directory (optional)
    #[arg(short = 'H', long)]
    history: Option<PathBuf>,
}

fn find_history_dir(start_path: &Path) -> Option<PathBuf> {
    let history_dir = start_path.join(".history");
    if history_dir.exists() && history_dir.is_dir() {
        Some(history_dir)
    } else {
        start_path.parent()
            .and_then(|parent| find_history_dir(parent))
    }
}

fn prompt_for_history_dir() -> io::Result<PathBuf> {
    print!("No .history directory found. Please specify a path (or press Enter to create one): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if input.is_empty() {
        Ok(PathBuf::from(".history"))
    } else {
        Ok(PathBuf::from(input))
    }
}

fn validate_history_dir(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            // Check if we can write to the directory
            let test_file = path.join("test_write");
            match std::fs::write(&test_file, b"test") {
                Ok(_) => {
                    std::fs::remove_file(test_file)?;
                    Ok(())
                },
                Err(e) => Err(HistorianError::Analysis(
                    format!("History directory is not writable: {}", e)
                )),
            }
        } else {
            Err(HistorianError::Analysis(
                "Specified history path exists but is not a directory".to_string()
            ))
        }
    } else {
        // Try to create the directory
        std::fs::create_dir_all(path)?;
        Ok(())
    }
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

    // Initialize progress bars
    let progress = ProgressManager::new();

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle history directory
    progress.scan_progress.set_message("Checking history directory...");
    let history_dir = if let Some(dir) = cli.history {
        dir
    } else {
        match find_history_dir(&cli.repo) {
            Some(dir) => {
                info!("Found existing history directory: {}", dir.display());
                dir
            },
            None => {
                let dir = prompt_for_history_dir()?;
                info!("Using history directory: {}", dir.display());
                dir
            }
        }
    };

    // Validate and create history directory if needed
    validate_history_dir(&history_dir)?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&cli.output)?;

    // Parse plugins
    let plugins = cli.plugins
        .map(|p| p.split(',').map(String::from).collect())
        .unwrap_or_else(Vec::new);

    // Create configuration
    let config = Config {
        repo_path: cli.repo,
        output_dir: cli.output.clone(),
        history_dir: history_dir.clone(),
        plugins,
        ml_enabled: cli.ml,
        visualization_enabled: cli.visualize,
        recursive: cli.recursive,
        file_pattern: cli.pattern,
    };

    info!("Starting code analysis...");
    progress.start_scanning();

    // Create analyzer
    let analyzer = CodeAnalyzer::new(vec![], config.ml_enabled)?;

    // Perform analysis
    progress.finish_scanning();
    progress.start_analysis(100); // TODO: Get actual commit count
    let analysis = analyzer.analyze(&config)?;
    progress.finish_analysis();

    info!(
        "Analysis complete: {} commits, {} changes",
        analysis.metrics.total_commits,
        analysis.metrics.total_changes
    );

    // Generate visualizations if enabled
    if config.visualization_enabled {
        info!("Generating visualizations...");
        progress.start_visualization();
        
        let visualizer = Visualizer::new(&config.output_dir)?;

        // Generate timeline
        match visualizer.generate_timeline(&analysis) {
            Ok(path) => {
                info!("Timeline generated: {}", path);
                progress.inc_visualization();
            },
            Err(e) => warn!("Failed to generate timeline: {}", e),
        }

        // Generate category distribution
        match visualizer.generate_category_distribution(&analysis) {
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
    }

    // Generate markdown report
    generate_report(&config.output_dir, &analysis)?;

    info!("Analysis complete! Results saved in {}", config.output_dir.display());
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
