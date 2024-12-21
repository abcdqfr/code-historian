use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use crate::{Result, HistorianError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub theme: String,
    pub timeline_style: String,
    pub chart_style: String,
    pub color_scheme: Vec<String>,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            timeline_style: "standard".to_string(),
            chart_style: "modern".to_string(),
            color_scheme: vec![
                "#4C9AFF".to_string(),
                "#F66D44".to_string(),
                "#6C8EBF".to_string(),
                "#8DB600".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorianConfig {
    pub default_history_dir: PathBuf,
    pub default_output_dir: PathBuf,
    pub preferred_plugins: Vec<String>,
    pub visualization: VisualizationConfig,
    pub auto_create_dirs: bool,
    pub default_ml_enabled: bool,
}

impl Default for HistorianConfig {
    fn default() -> Self {
        Self {
            default_history_dir: PathBuf::from(".history"),
            default_output_dir: PathBuf::from("docs/history"),
            preferred_plugins: Vec::new(),
            visualization: VisualizationConfig::default(),
            auto_create_dirs: true,
            default_ml_enabled: false,
        }
    }
}

impl HistorianConfig {
    pub fn load() -> Result<Self> {
        if let Some(config_path) = Self::config_path() {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                Ok(toml::from_str(&content)
                    .map_err(|e| HistorianError::Analysis(format!("Failed to parse config: {}", e)))?)
            } else {
                let config = Self::default();
                config.save()?;
                Ok(config)
            }
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::config_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(self)
                .map_err(|e| HistorianError::Analysis(format!("Failed to serialize config: {}", e)))?;
            std::fs::write(&config_path, content)?;
            Ok(())
        } else {
            Err(HistorianError::Analysis("Could not determine config path".to_string()))
        }
    }

    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "code-historian", "code-historian")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
    }

    pub fn merge_with_cli(&self, cli_config: crate::Config) -> crate::Config {
        crate::Config {
            repo_path: cli_config.repo_path,
            output_dir: cli_config.output_dir.clone().or_else(|| Some(self.default_output_dir.clone())).unwrap(),
            history_dir: cli_config.history_dir.clone().or_else(|| Some(self.default_history_dir.clone())).unwrap(),
            plugins: if cli_config.plugins.is_empty() {
                self.preferred_plugins.clone()
            } else {
                cli_config.plugins
            },
            ml_enabled: cli_config.ml_enabled.unwrap_or(self.default_ml_enabled),
            visualization_enabled: cli_config.visualization_enabled,
            recursive: cli_config.recursive,
            file_pattern: cli_config.file_pattern,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = HistorianConfig::default();
        assert_eq!(config.default_history_dir, PathBuf::from(".history"));
        assert_eq!(config.default_output_dir, PathBuf::from("docs/history"));
        assert!(config.preferred_plugins.is_empty());
    }

    #[test]
    fn test_visualization_config() {
        let config = VisualizationConfig::default();
        assert_eq!(config.theme, "default");
        assert_eq!(config.timeline_style, "standard");
        assert_eq!(config.chart_style, "modern");
        assert!(!config.color_scheme.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = HistorianConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: HistorianConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(config.default_history_dir, deserialized.default_history_dir);
    }
} 