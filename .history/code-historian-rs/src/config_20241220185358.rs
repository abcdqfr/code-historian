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
    pub default_output_dir: PathBuf,
    pub preferred_plugins: Vec<String>,
    pub visualization: VisualizationConfig,
    pub auto_create_dirs: bool,
    pub default_ml_enabled: bool,
}

impl Default for HistorianConfig {
    fn default() -> Self {
        Self {
            default_output_dir: PathBuf::from("docs/code-history"),
            preferred_plugins: Vec::new(),
            visualization: VisualizationConfig::default(),
            auto_create_dirs: true,
            default_ml_enabled: false,
        }
    }
}

impl HistorianConfig {
    pub fn load() -> Result<Self> {
        // Try loading in order: project, user, system
        Self::load_project_config()
            .or_else(|_| Self::load_user_config())
            .or_else(|_| Self::load_system_config())
            .or_else(|_| Ok(Self::default()))
    }

    pub fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::project_config_path() {
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

    fn load_project_config() -> Result<Self> {
        if let Some(path) = Self::project_config_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                Ok(toml::from_str(&content)
                    .map_err(|e| HistorianError::Analysis(format!("Failed to parse config: {}", e)))?)
            } else {
                Err(HistorianError::Analysis("No project config found".to_string()))
            }
        } else {
            Err(HistorianError::Analysis("Could not determine project config path".to_string()))
        }
    }

    fn load_user_config() -> Result<Self> {
        if let Some(path) = Self::user_config_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                Ok(toml::from_str(&content)
                    .map_err(|e| HistorianError::Analysis(format!("Failed to parse config: {}", e)))?)
            } else {
                Err(HistorianError::Analysis("No user config found".to_string()))
            }
        } else {
            Err(HistorianError::Analysis("Could not determine user config path".to_string()))
        }
    }

    fn load_system_config() -> Result<Self> {
        if let Some(path) = Self::system_config_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                Ok(toml::from_str(&content)
                    .map_err(|e| HistorianError::Analysis(format!("Failed to parse config: {}", e)))?)
            } else {
                Err(HistorianError::Analysis("No system config found".to_string()))
            }
        } else {
            Err(HistorianError::Analysis("Could not determine system config path".to_string()))
        }
    }

    pub fn project_config_path() -> Option<PathBuf> {
        std::env::current_dir().ok().map(|dir| dir.join(".code-historian/config.toml"))
    }

    pub fn user_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "code-historian", "code-historian")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
    }

    pub fn system_config_path() -> Option<PathBuf> {
        Some(PathBuf::from("/etc/code-historian/config.toml"))
    }

    pub fn find_historian_dir() -> Option<PathBuf> {
        std::env::current_dir().ok().and_then(|start_path| {
            let mut current = start_path.as_path();
            loop {
                let historian_dir = current.join(".code-historian");
                if historian_dir.is_dir() {
                    return Some(historian_dir);
                }
                match current.parent() {
                    Some(parent) => current = parent,
                    None => break,
                }
            }
            None
        })
    }

    pub fn merge_with_cli(&self, cli_config: crate::Config) -> crate::Config {
        crate::Config {
            repo_path: cli_config.repo_path,
            output_dir: cli_config.output_dir.or_else(|| Some(self.default_output_dir.clone())),
            plugins: if cli_config.plugins.is_empty() {
                self.preferred_plugins.clone()
            } else {
                cli_config.plugins
            },
            ml_enabled: Some(cli_config.ml_enabled.unwrap_or(self.default_ml_enabled)),
            visualization_enabled: cli_config.visualization_enabled,
            recursive: cli_config.recursive,
            file_pattern: cli_config.file_pattern,
        }
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "output_dir" => Some(self.default_output_dir.to_string_lossy().into_owned()),
            "ml_enabled" => Some(self.default_ml_enabled.to_string()),
            "auto_create_dirs" => Some(self.auto_create_dirs.to_string()),
            "visualization.theme" => Some(self.visualization.theme.clone()),
            "visualization.timeline_style" => Some(self.visualization.timeline_style.clone()),
            "visualization.chart_style" => Some(self.visualization.chart_style.clone()),
            _ => None,
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "output_dir" => self.default_output_dir = PathBuf::from(value),
            "ml_enabled" => self.default_ml_enabled = value.parse()
                .map_err(|_| HistorianError::Analysis("Invalid boolean value".to_string()))?,
            "auto_create_dirs" => self.auto_create_dirs = value.parse()
                .map_err(|_| HistorianError::Analysis("Invalid boolean value".to_string()))?,
            "visualization.theme" => self.visualization.theme = value.to_string(),
            "visualization.timeline_style" => self.visualization.timeline_style = value.to_string(),
            "visualization.chart_style" => self.visualization.chart_style = value.to_string(),
            _ => return Err(HistorianError::Analysis(format!("Unknown config key: {}", key))),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = HistorianConfig::default();
        assert_eq!(config.default_output_dir, PathBuf::from("docs/code-history"));
        assert!(config.preferred_plugins.is_empty());
        assert!(config.auto_create_dirs);
        assert!(!config.default_ml_enabled);
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
        assert_eq!(config.default_output_dir, deserialized.default_output_dir);
    }

    #[test]
    fn test_config_get_set() {
        let mut config = HistorianConfig::default();
        
        // Test get
        assert_eq!(config.get_value("ml_enabled"), Some("false".to_string()));
        
        // Test set
        config.set_value("ml_enabled", "true").unwrap();
        assert_eq!(config.get_value("ml_enabled"), Some("true".to_string()));
        
        // Test invalid key
        assert!(config.set_value("invalid_key", "value").is_err());
    }
} 