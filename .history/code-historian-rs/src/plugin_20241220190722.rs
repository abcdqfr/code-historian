use std::path::{Path, PathBuf};
use libloading::{Library, Symbol};
use serde::{Serialize, Deserialize};
use semver::Version;
use crate::{Result, HistorianError, Category};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: Version,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<PluginDependency>,
    pub supported_languages: Vec<String>,
    pub configuration: Option<PluginConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_req: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub settings: std::collections::HashMap<String, String>,
    pub defaults: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub struct AnalysisContext<'a> {
    pub file_path: &'a Path,
    pub content: &'a str,
    pub diff: Option<&'a str>,
    pub language: Option<&'a str>,
    pub config: Option<&'a PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResult {
    pub categories: Vec<Category>,
    pub patterns: Vec<String>,
    pub metrics: std::collections::HashMap<String, f64>,
    pub annotations: Vec<PluginAnnotation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAnnotation {
    pub line: usize,
    pub message: String,
    pub severity: AnnotationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationSeverity {
    Info,
    Warning,
    Error,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn manifest(&self) -> &PluginManifest;
    fn analyze(&self, context: &AnalysisContext) -> Result<PluginResult>;
    fn supports_language(&self, lang: &str) -> bool;
}

pub struct PluginManager {
    plugins_dir: PathBuf,
    loaded_plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
}

type PluginCreate = unsafe fn() -> *mut dyn Plugin;

impl PluginManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            loaded_plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self) -> Result<()> {
        if !self.plugins_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| {
                ext == std::env::consts::DLL_EXTENSION
            }) {
                self.load_plugin(&path)?;
            }
        }

        Ok(())
    }

    fn load_plugin(&mut self, path: &Path) -> Result<()> {
        unsafe {
            let library = Library::new(path)
                .map_err(|e| HistorianError::Plugin(format!("Failed to load plugin: {}", e)))?;

            let create: Symbol<PluginCreate> = library.get(b"_plugin_create")
                .map_err(|e| HistorianError::Plugin(format!("Plugin entry point not found: {}", e)))?;

            let plugin = Box::from_raw(create());
            
            // Validate plugin manifest
            self.validate_plugin(&plugin)?;

            self.loaded_plugins.push(plugin);
            self.loaded_libraries.push(library);
        }

        Ok(())
    }

    fn validate_plugin(&self, plugin: &Box<dyn Plugin>) -> Result<()> {
        let manifest = plugin.manifest();

        // Check dependencies
        for dep in &manifest.dependencies {
            if !self.has_plugin(&dep.name, &dep.version_req) {
                return Err(HistorianError::Plugin(
                    format!("Missing dependency: {} {}", dep.name, dep.version_req)
                ));
            }
        }

        Ok(())
    }

    pub fn get_plugins(&self) -> &[Box<dyn Plugin>] {
        &self.loaded_plugins
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.loaded_plugins.iter().find(|p| p.name() == name)
    }

    fn has_plugin(&self, name: &str, version_req: &str) -> bool {
        if let Ok(req) = semver::VersionReq::parse(version_req) {
            self.loaded_plugins.iter().any(|p| {
                p.name() == name && req.matches(p.version())
            })
        } else {
            false
        }
    }

    pub fn install_plugin(&mut self, name: &str, source: &Path) -> Result<()> {
        let target = self.plugins_dir.join(format!(
            "{}{}",
            name,
            std::env::consts::DLL_EXTENSION
        ));

        std::fs::copy(source, &target)?;
        self.load_plugin(&target)?;

        Ok(())
    }

    pub fn remove_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(index) = self.loaded_plugins.iter().position(|p| p.name() == name) {
            self.loaded_plugins.remove(index);
            self.loaded_libraries.remove(index);

            let plugin_path = self.plugins_dir.join(format!(
                "{}{}",
                name,
                std::env::consts::DLL_EXTENSION
            ));
            
            if plugin_path.exists() {
                std::fs::remove_file(plugin_path)?;
            }
        }

        Ok(())
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        self.loaded_plugins.clear();
        self.loaded_libraries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path().to_path_buf());
        assert!(manager.get_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manifest() {
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            dependencies: vec![],
            supported_languages: vec!["rust".to_string()],
            configuration: None,
        };

        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, Version::new(1, 0, 0));
    }

    #[test]
    fn test_plugin_result() {
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("complexity".to_string(), 10.0);

        let result = PluginResult {
            categories: vec![Category::Performance],
            patterns: vec!["pattern1".to_string()],
            metrics,
            annotations: vec![],
        };

        assert_eq!(result.categories.len(), 1);
        assert_eq!(result.patterns.len(), 1);
        assert_eq!(result.metrics.len(), 1);
    }
} 
} 