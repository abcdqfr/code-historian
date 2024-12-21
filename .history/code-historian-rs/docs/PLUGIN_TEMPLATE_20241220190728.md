# Code Historian Plugin Development Guide

## Overview

This guide explains how to create plugins for Code Historian. Plugins can extend the analyzer's capabilities by adding new analysis types, pattern detection, or language support.

## Plugin Structure

### 1. Plugin Manifest

Every plugin must provide a manifest in `plugin.toml`:

```toml
[plugin]
name = "my-analyzer"
version = "0.1.0"
description = "My custom code analyzer"
author = "Your Name <your.email@example.com>"

[dependencies]
# Optional dependencies on other plugins
security-analyzer = "^1.0.0"

[languages]
supported = ["rust", "python", "javascript"]

[config]
# Default configuration values
[config.defaults]
threshold = "0.8"
max_depth = "5"

[config.settings]
# Configuration schema
threshold = { type = "float", min = "0.0", max = "1.0" }
max_depth = { type = "integer", min = "1", max = "10" }
```

### 2. Plugin Implementation

Here's a template for implementing a plugin in Rust:

```rust
use code_historian::{
    Plugin, PluginManifest, AnalysisContext, PluginResult,
    Category, PluginAnnotation, AnnotationSeverity,
};
use semver::Version;
use std::collections::HashMap;

#[derive(Default)]
pub struct MyAnalyzer {
    manifest: PluginManifest,
}

#[no_mangle]
pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
    let plugin = MyAnalyzer::new();
    Box::into_raw(Box::new(plugin))
}

impl MyAnalyzer {
    pub fn new() -> Self {
        // Load manifest from plugin.toml
        let manifest = toml::from_str(include_str!("plugin.toml"))
            .expect("Failed to load plugin manifest");
        
        Self { manifest }
    }
}

impl Plugin for MyAnalyzer {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn version(&self) -> &Version {
        &self.manifest.version
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<PluginResult, Box<dyn std::error::Error>> {
        let mut categories = Vec::new();
        let mut patterns = Vec::new();
        let mut metrics = HashMap::new();
        let mut annotations = Vec::new();

        // Implement your analysis logic here
        // Example:
        if let Some(content) = &context.content {
            // Pattern detection
            if content.contains("TODO") {
                patterns.push("todo-comment".to_string());
                annotations.push(PluginAnnotation {
                    line: 1, // Calculate actual line number
                    message: "Found TODO comment".to_string(),
                    severity: AnnotationSeverity::Info,
                });
            }

            // Metrics calculation
            let complexity = calculate_complexity(content);
            metrics.insert("complexity".to_string(), complexity);

            // Category assignment
            if complexity > 10.0 {
                categories.push(Category::Performance);
            }
        }

        Ok(PluginResult {
            categories,
            patterns,
            metrics,
            annotations,
        })
    }

    fn supports_language(&self, lang: &str) -> bool {
        self.manifest.supported_languages.contains(&lang.to_string())
    }
}

fn calculate_complexity(content: &str) -> f64 {
    // Implement your complexity calculation
    0.0
}
```

### 3. Building the Plugin

Create a `Cargo.toml` for your plugin:

```toml
[package]
name = "my-analyzer"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
code-historian = { version = "0.1" }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
semver = "1.0"
```

## Plugin Guidelines

### 1. Performance

- Keep memory usage minimal
- Use efficient algorithms
- Cache results when appropriate
- Release resources properly

### 2. Error Handling

- Handle all potential errors gracefully
- Provide meaningful error messages
- Don't panic or crash
- Log errors appropriately

### 3. Configuration

- Validate all configuration values
- Provide sensible defaults
- Document configuration options
- Handle missing configuration gracefully

### 4. Testing

Create comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let analyzer = MyAnalyzer::new();
        let context = AnalysisContext {
            file_path: Path::new("test.rs"),
            content: "fn main() {\n    // TODO: implement\n}",
            diff: None,
            language: Some("rust"),
            config: None,
        };

        let result = analyzer.analyze(&context).unwrap();
        assert!(!result.patterns.is_empty());
        assert!(!result.annotations.is_empty());
    }

    #[test]
    fn test_language_support() {
        let analyzer = MyAnalyzer::new();
        assert!(analyzer.supports_language("rust"));
        assert!(!analyzer.supports_language("unknown"));
    }
}
```

## Installation

1. Build your plugin:
   ```bash
   cargo build --release
   ```

2. Install the plugin:
   ```bash
   code-historian plugin install my-analyzer ./target/release/libmy_analyzer.so
   ```

## Best Practices

1. **Documentation**
   - Document your plugin's purpose
   - Explain configuration options
   - Provide usage examples
   - List supported languages

2. **Versioning**
   - Follow semantic versioning
   - Document breaking changes
   - Maintain compatibility when possible
   - Test with different Code Historian versions

3. **Security**
   - Validate all inputs
   - Handle file access safely
   - Respect file permissions
   - Don't execute arbitrary code

4. **Integration**
   - Work well with other plugins
   - Follow Code Historian conventions
   - Use standard categories when possible
   - Provide meaningful metrics

## Example Plugins

1. **Security Analyzer**
   - Detects security patterns
   - Checks for vulnerabilities
   - Analyzes dependencies
   - Reports security metrics

2. **Performance Analyzer**
   - Calculates complexity metrics
   - Detects performance patterns
   - Analyzes resource usage
   - Suggests optimizations

3. **Documentation Analyzer**
   - Checks documentation coverage
   - Validates doc comments
   - Analyzes API documentation
   - Reports documentation quality

## Support

- Report issues on GitHub
- Join the community chat
- Read the documentation
- Check the FAQ 