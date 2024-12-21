use std::path::PathBuf;
use dialoguer::{theme::ColorfulTheme, Input, Select, MultiSelect, Confirm};
use crate::{Config, Result, HistorianError};

pub struct InteractiveConfig {
    theme: ColorfulTheme,
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }
}

impl InteractiveConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure(&self, mut config: Config) -> Result<Config> {
        println!("\nCode Historian Interactive Configuration\n");

        // Configure history directory
        if config.history_dir.is_none() {
            let use_default = Confirm::with_theme(&self.theme)
                .with_prompt("Use default history directory (.history)?")
                .default(true)
                .interact()?;

            if !use_default {
                let history_dir: String = Input::with_theme(&self.theme)
                    .with_prompt("Enter history directory path")
                    .default(".history".into())
                    .interact_text()?;
                config = config.with_history_dir(PathBuf::from(history_dir));
            }
        }

        // Configure output directory
        if config.output_dir.is_none() {
            let use_default = Confirm::with_theme(&self.theme)
                .with_prompt("Use default output directory (docs/history)?")
                .default(true)
                .interact()?;

            if !use_default {
                let output_dir: String = Input::with_theme(&self.theme)
                    .with_prompt("Enter output directory path")
                    .default("docs/history".into())
                    .interact_text()?;
                config = config.with_output_dir(PathBuf::from(output_dir));
            }
        }

        // Configure ML
        if config.ml_enabled.is_none() {
            let use_ml = Confirm::with_theme(&self.theme)
                .with_prompt("Enable machine learning-based categorization?")
                .default(false)
                .interact()?;
            config = config.with_ml(use_ml);
        }

        // Configure plugins
        if config.plugins.is_empty() {
            let available_plugins = vec![
                "security",
                "performance",
                "documentation",
                "testing",
                "architecture",
            ];

            let selections = MultiSelect::with_theme(&self.theme)
                .with_prompt("Select plugins to enable")
                .items(&available_plugins)
                .interact()?;

            let selected_plugins: Vec<String> = selections
                .iter()
                .map(|&i| available_plugins[i].to_string())
                .collect();

            config = config.with_plugins(selected_plugins);
        }

        // Configure visualization
        if !config.visualization_enabled {
            let use_viz = Confirm::with_theme(&self.theme)
                .with_prompt("Enable visualizations?")
                .default(true)
                .interact()?;
            config = config.with_visualization(use_viz);
        }

        // Configure file pattern
        if config.file_pattern.is_none() {
            let use_pattern = Confirm::with_theme(&self.theme)
                .with_prompt("Apply file pattern filter?")
                .default(false)
                .interact()?;

            if use_pattern {
                let pattern: String = Input::with_theme(&self.theme)
                    .with_prompt("Enter file pattern (e.g., *.rs, *.py)")
                    .interact_text()?;
                config = config.with_pattern(pattern);
            }
        }

        // Configure recursive search
        if !config.recursive {
            let use_recursive = Confirm::with_theme(&self.theme)
                .with_prompt("Enable recursive directory search?")
                .default(true)
                .interact()?;
            config = config.with_recursive(use_recursive);
        }

        Ok(config)
    }

    pub fn confirm_config(&self, config: &Config) -> Result<bool> {
        println!("\nConfiguration Summary:");
        println!("Repository: {}", config.repo_path.display());
        println!("History Directory: {}", config.history_dir.as_ref().map_or("default", |p| p.to_str().unwrap_or("invalid")));
        println!("Output Directory: {}", config.output_dir.as_ref().map_or("default", |p| p.to_str().unwrap_or("invalid")));
        println!("ML Enabled: {}", config.ml_enabled.unwrap_or(false));
        println!("Plugins: {}", if config.plugins.is_empty() { "none" } else { config.plugins.join(", ").as_str() });
        println!("Visualizations: {}", config.visualization_enabled);
        println!("Recursive: {}", config.recursive);
        println!("File Pattern: {}", config.file_pattern.as_deref().unwrap_or("none"));

        Confirm::with_theme(&self.theme)
            .with_prompt("\nProceed with this configuration?")
            .default(true)
            .interact()
            .map_err(|e| HistorianError::Analysis(format!("Failed to get confirmation: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_interactive_config_creation() {
        let config = InteractiveConfig::new();
        assert!(config.theme.values.prompt_prefix.len() > 0);
    }

    // Note: Interactive tests would require mock input/output
    // Real interactive testing should be done manually
} 