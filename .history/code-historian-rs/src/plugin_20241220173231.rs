use std::collections::HashMap;
use crate::{Result, Category, HistorianError};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, context: &AnalysisContext) -> Result<PluginResult>;
    fn supports_language(&self, lang: &str) -> bool;
}

#[derive(Debug)]
pub struct AnalysisContext<'a> {
    pub diff: &'a str,
    pub file_path: &'a str,
    pub language: &'a str,
    pub commit_message: &'a str,
}

#[derive(Debug)]
pub struct PluginResult {
    pub categories: Vec<Category>,
    pub metrics: HashMap<String, f64>,
    pub annotations: Vec<String>,
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn analyze(&self, context: &AnalysisContext) -> Result<Vec<PluginResult>> {
        let mut results = Vec::new();
        
        for plugin in &self.plugins {
            if plugin.supports_language(context.language) {
                match plugin.analyze(context) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!("Plugin '{}' failed: {}", plugin.name(), e);
                    }
                }
            }
        }
        
        Ok(results)
    }
}

// Example built-in plugins

pub struct SecurityAnalyzer;

impl Plugin for SecurityAnalyzer {
    fn name(&self) -> &str {
        "security"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<PluginResult> {
        let mut categories = Vec::new();
        let mut metrics = HashMap::new();
        let mut annotations = Vec::new();

        // Check for security-related patterns
        let patterns = [
            "password", "token", "secret", "auth", "crypt",
            "hash", "salt", "key", "certificate", "vulnerability",
        ];

        let mut security_matches = 0;
        for pattern in patterns {
            if context.diff.contains(pattern) {
                security_matches += 1;
                annotations.push(format!("Found security-related pattern: {}", pattern));
            }
        }

        if security_matches > 0 {
            categories.push(Category::Security);
            metrics.insert("security_patterns".to_string(), security_matches as f64);
        }

        Ok(PluginResult {
            categories,
            metrics,
            annotations,
        })
    }

    fn supports_language(&self, _lang: &str) -> bool {
        // Security analysis is language-agnostic
        true
    }
}

pub struct PerformanceAnalyzer;

impl Plugin for PerformanceAnalyzer {
    fn name(&self) -> &str {
        "performance"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<PluginResult> {
        let mut categories = Vec::new();
        let mut metrics = HashMap::new();
        let mut annotations = Vec::new();

        // Check for performance-related patterns
        let patterns = [
            "optimize", "performance", "slow", "fast",
            "cache", "memory", "leak", "profile",
            "benchmark", "latency", "throughput",
        ];

        let mut perf_matches = 0;
        for pattern in patterns {
            if context.diff.contains(pattern) {
                perf_matches += 1;
                annotations.push(format!("Found performance-related pattern: {}", pattern));
            }
        }

        if perf_matches > 0 {
            categories.push(Category::Performance);
            metrics.insert("performance_patterns".to_string(), perf_matches as f64);
        }

        Ok(PluginResult {
            categories,
            metrics,
            annotations,
        })
    }

    fn supports_language(&self, _lang: &str) -> bool {
        // Performance analysis is language-agnostic
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_analyzer() {
        let analyzer = SecurityAnalyzer;
        let context = AnalysisContext {
            diff: "Added password validation\nFixed token handling",
            file_path: "auth.rs",
            language: "rust",
            commit_message: "Security improvements",
        };

        let result = analyzer.analyze(&context).unwrap();
        assert!(result.categories.contains(&Category::Security));
        assert!(!result.annotations.is_empty());
    }

    #[test]
    fn test_performance_analyzer() {
        let analyzer = PerformanceAnalyzer;
        let context = AnalysisContext {
            diff: "Optimized database queries\nAdded caching layer",
            file_path: "db.rs",
            language: "rust",
            commit_message: "Performance improvements",
        };

        let result = analyzer.analyze(&context).unwrap();
        assert!(result.categories.contains(&Category::Performance));
        assert!(!result.annotations.is_empty());
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        manager.register_plugin(Box::new(SecurityAnalyzer));
        manager.register_plugin(Box::new(PerformanceAnalyzer));

        let context = AnalysisContext {
            diff: "Added password validation\nOptimized queries",
            file_path: "auth.rs",
            language: "rust",
            commit_message: "Security and performance improvements",
        };

        let results = manager.analyze(&context).unwrap();
        assert_eq!(results.len(), 2);
    }
} 