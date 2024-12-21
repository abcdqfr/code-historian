use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use thiserror::Error;

pub mod analyzer;
pub mod config;
pub mod git;
pub mod interactive;
pub mod ml;
pub mod plugin;
pub mod report;
pub mod visualization;
pub mod watch;

#[derive(Debug, Error)]
pub enum HistorianError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Analysis error: {0}")]
    Analysis(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Visualization error: {0}")]
    Visualization(String),
}

pub type Result<T> = std::result::Result<T, HistorianError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repo_path: PathBuf,
    pub output_dir: Option<PathBuf>,
    pub history_dir: Option<PathBuf>,
    pub plugins: Vec<String>,
    pub ml_enabled: Option<bool>,
    pub visualization_enabled: bool,
    pub recursive: bool,
    pub file_pattern: Option<String>,
}

impl Config {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            output_dir: None,
            history_dir: None,
            plugins: Vec::new(),
            ml_enabled: None,
            visualization_enabled: false,
            recursive: false,
            file_pattern: None,
        }
    }

    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = Some(dir);
        self
    }

    pub fn with_history_dir(mut self, dir: PathBuf) -> Self {
        self.history_dir = Some(dir);
        self
    }

    pub fn with_plugins(mut self, plugins: Vec<String>) -> Self {
        self.plugins = plugins;
        self
    }

    pub fn with_ml(mut self, enabled: bool) -> Self {
        self.ml_enabled = Some(enabled);
        self
    }

    pub fn with_visualization(mut self, enabled: bool) -> Self {
        self.visualization_enabled = enabled;
        self
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.file_pattern = Some(pattern);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub commit_id: String,
    pub message: String,
    pub diff: String,
    pub categories: Vec<Category>,
    pub impact_score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Architecture,
    Api,
    Logic,
    Data,
    ErrorHandling,
    Logging,
    Documentation,
    Testing,
    Performance,
    Security,
    Refactoring,
    Dependencies,
    Configuration,
    UiUx,
    Accessibility,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Analysis {
    pub changes: Vec<Change>,
    pub metrics: Metrics,
    pub patterns: Vec<Pattern>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub total_commits: usize,
    pub total_changes: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub category_distribution: std::collections::HashMap<Category, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub occurrences: usize,
    pub impact: f64,
}

pub trait Analyzer {
    fn analyze(&self, config: &Config) -> Result<Analysis>;
    fn categorize(&self, diff: &str) -> Result<Vec<Category>>;
    fn calculate_impact(&self, change: &Change) -> f64;
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, context: &plugin::AnalysisContext) -> Result<plugin::PluginResult>;
    fn supports_language(&self, lang: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_equality() {
        assert_eq!(Category::Architecture, Category::Architecture);
        assert_ne!(Category::Api, Category::Logic);
    }

    #[test]
    fn test_config_creation() {
        let config = Config {
            repo_path: PathBuf::from("/tmp/repo"),
            output_dir: Some(PathBuf::from("/tmp/output")),
            history_dir: Some(PathBuf::from("/tmp/history")),
            plugins: vec!["security".to_string()],
            ml_enabled: Some(true),
            visualization_enabled: true,
            recursive: true,
            file_pattern: Some("*.rs".to_string()),
        };
        assert!(config.ml_enabled.unwrap());
        assert_eq!(config.plugins.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let config = Config {
            repo_path: PathBuf::from("/tmp/repo"),
            output_dir: Some(PathBuf::from("/tmp/output")),
            history_dir: Some(PathBuf::from("/tmp/history")),
            plugins: vec!["security".to_string()],
            ml_enabled: Some(true),
            visualization_enabled: true,
            recursive: true,
            file_pattern: Some("*.rs".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.repo_path, deserialized.repo_path);
        assert_eq!(config.plugins, deserialized.plugins);
    }
} 