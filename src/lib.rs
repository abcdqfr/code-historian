pub mod benchmarks;
pub use benchmarks::{Benchmarker, BenchmarkResults};

/// A change in the codebase, representing a single modification to a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// The timestamp when the change was made.
    pub timestamp: DateTime<Utc>,
    /// The path to the file that was changed, relative to the repository root.
    pub file: String,
    /// The number of lines added in this change.
    pub lines_added: usize,
    /// The number of lines removed in this change.
    pub lines_removed: usize,
    /// The category of the change (e.g., "feature", "bugfix", "refactor").
    pub category: String,
    /// The impact score of the change, ranging from 0.0 to 1.0.
    pub impact_score: f64,
}

/// Configuration for the Code Historian tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The path to the repository to analyze.
    pub repo_path: PathBuf,
    /// The path to the history directory.
    pub history_dir: PathBuf,
    /// The path to the output directory for reports.
    pub output_dir: PathBuf,
    /// List of enabled plugins.
    pub plugins: Vec<String>,
    /// Whether machine learning features are enabled.
    pub ml_enabled: bool,
}

/// Results of analyzing a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// List of changes found in the repository.
    pub changes: Vec<Change>,
    /// The start time of the analysis period.
    pub start_time: DateTime<Utc>,
    /// The end time of the analysis period.
    pub end_time: DateTime<Utc>,
}

/// Options for generating a timeline visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineOptions {
    /// The width of the timeline in pixels.
    pub width: u32,
    /// The height of the timeline in pixels.
    pub height: u32,
    /// Whether to show change categories.
    pub show_categories: bool,
    /// Whether to show impact scores.
    pub show_impact_scores: bool,
}

/// Options for rendering charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    /// The width of the chart in pixels.
    pub width: u32,
    /// The height of the chart in pixels.
    pub height: u32,
    /// The type of chart to render (e.g., "pie", "bar", "line").
    pub chart_type: String,
    /// Whether to show the chart legend.
    pub show_legend: bool,
}

/// Options for interactive visualizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveOptions {
    /// Whether to enable zooming.
    pub enable_zoom: bool,
    /// Whether to enable tooltips.
    pub enable_tooltips: bool,
    /// Whether to enable filtering.
    pub enable_filtering: bool,
    /// Whether to enable sorting.
    pub enable_sorting: bool,
}

/// A trait that must be implemented by all Code Historian plugins.
pub trait Plugin {
    /// Returns the name of the plugin.
    fn name(&self) -> &str;

    /// Analyzes the repository using this plugin.
    ///
    /// # Arguments
    ///
    /// * `context` - The analysis context containing repository information.
    ///
    /// # Returns
    ///
    /// A `PluginResult` containing the analysis results or an error.
    fn analyze(&self, context: &AnalysisContext) -> PluginResult;

    /// Returns a list of plugin dependencies.
    ///
    /// # Returns
    ///
    /// A slice containing the names of plugins that must be run before this one.
    fn dependencies(&self) -> &[String] {
        &[]
    }
}

/// Context provided to plugins during analysis.
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// The path to the repository being analyzed.
    pub repo_path: PathBuf,
    /// The path to the history directory.
    pub history_dir: PathBuf,
    /// The path to the output directory.
    pub output_dir: PathBuf,
}

/// Results from a plugin's analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginResult {
    /// List of changes detected by the plugin.
    pub changes: Vec<Change>,
    /// Additional metadata produced by the plugin.
    pub metadata: HashMap<String, Value>,
}

// ... existing code ... 