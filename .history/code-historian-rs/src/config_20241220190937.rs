use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use crate::{Result, HistorianError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub core: CoreConfig,
    pub analysis: AnalysisConfig,
    pub plugins: PluginsConfig,
    pub watch: WatchConfig,
    pub cache: CacheConfig,
    pub visualization: VisualizationConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub documentation: DocumentationConfig,
    pub ml: MlConfig,
    pub reporting: ReportingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub output_dir: PathBuf,
    pub history_dir: PathBuf,
    pub ml_enabled: bool,
    pub visualization_enabled: bool,
    pub output_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub recursive: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub min_impact_score: f64,
    pub max_changes_per_commit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    pub enabled: bool,
    pub auto_load: bool,
    pub directory: PathBuf,
    pub enabled_plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    pub enabled: bool,
    pub debounce: u64,
    pub auto_analyze: bool,
    pub auto_visualize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub directory: PathBuf,
    pub max_size: usize,
    pub ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub directory: PathBuf,
    pub theme: String,
    pub interactive: bool,
    pub max_data_points: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub severity_threshold: String,
    pub scan_sensitive: bool,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enabled: bool,
    pub complexity_threshold: usize,
    pub memory_threshold: usize,
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub enabled: bool,
    pub coverage_threshold: f64,
    pub required_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    pub model_dir: PathBuf,
    pub min_confidence: f64,
    pub auto_train: bool,
    pub max_training_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub format: String,
    pub sections: Vec<String>,
    pub detailed: bool,
    pub max_size: usize,
}

impl Config {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let mut config = Self::default();
        config.load_all(repo_path)?;
        Ok(config)
    }

    pub fn load_all(&mut self, repo_path: &Path) -> Result<()> {
        // Load system config
        if let Some(system_config) = self.load_system_config()? {
            self.merge(system_config);
        }

        // Load user config
        if let Some(user_config) = self.load_user_config()? {
            self.merge(user_config);
        }

        // Load project config
        if let Some(project_config) = self.load_project_config(repo_path)? {
            self.merge(project_config);
        }

        Ok(())
    }

    fn load_system_config() -> Result<Option<Config>> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "code-historian", "code-historian") {
            let config_path = proj_dirs.config_dir().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(config_path)?;
                return Ok(Some(toml::from_str(&content)?));
            }
        }
        Ok(None)
    }

    fn load_user_config() -> Result<Option<Config>> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "code-historian", "code-historian") {
            let config_path = proj_dirs.preference_dir().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(config_path)?;
                return Ok(Some(toml::from_str(&content)?));
            }
        }
        Ok(None)
    }

    fn load_project_config(&self, repo_path: &Path) -> Result<Option<Config>> {
        let config_path = repo_path
            .join(&self.core.history_dir)
            .join("config.toml");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(Some(toml::from_str(&content)?))
        } else {
            Ok(None)
        }
    }

    pub fn save(&self, repo_path: &Path) -> Result<()> {
        let config_path = repo_path
            .join(&self.core.history_dir)
            .join("config.toml");
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| HistorianError::Config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(config_path, content)?;
        Ok(())
    }

    pub fn merge(&mut self, other: Config) {
        self.core = other.core;
        self.analysis = other.analysis;
        self.plugins = other.plugins;
        self.watch = other.watch;
        self.cache = other.cache;
        self.visualization = other.visualization;
        self.security = other.security;
        self.performance = other.performance;
        self.documentation = other.documentation;
        self.ml = other.ml;
        self.reporting = other.reporting;
    }

    pub fn plugins_dir(&self) -> Result<PathBuf> {
        Ok(self.core.history_dir.join(&self.plugins.directory))
    }

    pub fn cache_dir(&self) -> Result<PathBuf> {
        Ok(self.core.history_dir.join(&self.cache.directory))
    }

    pub fn output_dir(&self) -> Result<PathBuf> {
        Ok(self.core.output_dir.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(include_str!("../templates/default_config.toml"))
            .expect("Failed to parse default configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.core.output_dir, PathBuf::from("docs/history"));
        assert_eq!(config.core.history_dir, PathBuf::from(".code-historian"));
        assert!(config.core.ml_enabled);
        assert!(config.core.visualization_enabled);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let config = Config::default();
        config.save(repo_path).unwrap();

        let loaded_config = Config::new(repo_path).unwrap();
        assert_eq!(loaded_config.core.output_dir, config.core.output_dir);
        assert_eq!(loaded_config.core.history_dir, config.core.history_dir);
    }

    #[test]
    fn test_config_merge() {
        let mut base_config = Config::default();
        let mut other_config = Config::default();

        other_config.core.output_dir = PathBuf::from("custom/output");
        other_config.core.ml_enabled = false;

        base_config.merge(other_config);

        assert_eq!(base_config.core.output_dir, PathBuf::from("custom/output"));
        assert!(!base_config.core.ml_enabled);
    }
} 