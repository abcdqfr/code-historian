//! Plugin system for Code Historian.
//!
//! This module provides the plugin system that allows extending Code Historian's functionality.
//! Plugins can analyze code changes, detect patterns, and contribute to the final report.
//!
//! # Writing Plugins
//!
//! To create a plugin:
//!
//! 1. Implement the `Plugin` trait
//! 2. Package your plugin as a Rust library
//! 3. Place the compiled library in the plugins directory
//!
//! # Example
//!
//! ```rust
//! use code_historian::plugin::{Plugin, AnalysisContext, PluginResult};
//!
//! pub struct ComplexityPlugin;
//!
//! impl Plugin for ComplexityPlugin {
//!     fn name(&self) -> &str {
//!         "complexity"
//!     }
//!
//!     fn analyze(&self, context: &AnalysisContext) -> PluginResult {
//!         // Analyze code complexity
//!         PluginResult::default()
//!     }
//!
//!     fn dependencies(&self) -> &[String] {
//!         &["git".to_string()]
//!     }
//! }
//! ```

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Manager for loading and running plugins.
#[derive(Default)]
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    /// Creates a new plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Loads plugins from the specified directory.
    ///
    /// # Arguments
    ///
    /// * `plugins_dir` - Path to the directory containing plugin libraries
    ///
    /// # Returns
    ///
    /// Result indicating success or failure of loading plugins
    pub fn load_plugins(&mut self, plugins_dir: &Path) -> Result<(), PluginError> {
        // Implementation details...
        Ok(())
    }

    /// Registers a plugin with the manager.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The plugin to register
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    /// Gets a reference to a plugin by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the plugin to retrieve
    ///
    /// # Returns
    ///
    /// Option containing a reference to the plugin if found
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Resolves plugin dependencies and returns them in execution order.
    ///
    /// # Returns
    ///
    /// Result containing a Vec of plugin references in dependency order
    pub fn resolve_dependencies(&self) -> Result<Vec<&dyn Plugin>, PluginError> {
        // Implementation details...
        Ok(vec![])
    }
}

/// Error type for plugin operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Plugin could not be loaded
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    /// Plugin dependencies could not be resolved
    #[error("Failed to resolve plugin dependencies: {0}")]
    DependencyError(String),

    /// Plugin execution failed
    #[error("Plugin execution failed: {0}")]
    ExecutionError(String),
}

/// Manifest file format for plugins.
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Name of the plugin
    pub name: String,
    /// Version of the plugin
    pub version: String,
    /// Description of the plugin's functionality
    pub description: String,
    /// List of plugin dependencies
    pub dependencies: Vec<String>,
    /// Minimum required Code Historian version
    pub min_version: String,
}

/// Configuration for a plugin.
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin-specific settings
    pub settings: HashMap<String, Value>,
}

/// Metadata about a plugin.
#[derive(Debug)]
pub struct PluginInfo {
    /// Name of the plugin
    pub name: String,
    /// Version of the plugin
    pub version: String,
    /// Description of the plugin
    pub description: String,
    /// Whether the plugin is currently enabled
    pub enabled: bool,
    /// List of plugin dependencies
    pub dependencies: Vec<String>,
}

/// Builder for creating plugin configurations.
pub struct PluginConfigBuilder {
    config: PluginConfig,
}

impl PluginConfigBuilder {
    /// Creates a new plugin configuration builder.
    pub fn new() -> Self {
        Self {
            config: PluginConfig {
                enabled: true,
                settings: HashMap::new(),
            },
        }
    }

    /// Sets whether the plugin is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Adds a setting to the plugin configuration.
    pub fn setting<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.config.settings.insert(key.to_string(), value.into());
        self
    }

    /// Builds the plugin configuration.
    pub fn build(self) -> PluginConfig {
        self.config
    }
}

impl Default for PluginConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
} 