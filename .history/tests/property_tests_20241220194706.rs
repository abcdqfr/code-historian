use proptest::prelude::*;
use std::path::PathBuf;
use code_historian::{Config, PluginManager};
use serde_json::Value;

proptest! {
    #[test]
    fn test_config_validation(
        repo_path in ".*",
        history_dir in ".*",
        output_dir in ".*",
        plugins in prop::collection::vec(any::<String>(), 0..10),
        ml_enabled in any::<bool>(),
    ) {
        let config = Config {
            repo_path: PathBuf::from(repo_path),
            history_dir: PathBuf::from(history_dir),
            output_dir: PathBuf::from(output_dir),
            plugins,
            ml_enabled,
        };

        // Config should always be serializable
        prop_assert!(serde_json::to_string(&config).is_ok());

        // Paths should be valid
        prop_assert!(!config.repo_path.to_string_lossy().contains('\0'));
        prop_assert!(!config.history_dir.to_string_lossy().contains('\0'));
        prop_assert!(!config.output_dir.to_string_lossy().contains('\0'));

        // Plugin names should be valid
        for plugin in &config.plugins {
            prop_assert!(!plugin.contains('\0'));
            prop_assert!(!plugin.is_empty());
        }
    }

    #[test]
    fn test_directory_structure(
        base_dir in ".*",
        subdirs in prop::collection::vec("[a-zA-Z0-9_-]+", 0..5),
    ) {
        let base = PathBuf::from(base_dir);
        
        // Base directory should be valid
        prop_assert!(!base.to_string_lossy().contains('\0'));

        // Subdirectories should be valid
        for subdir in &subdirs {
            let path = base.join(subdir);
            prop_assert!(!path.to_string_lossy().contains('\0'));
            prop_assert!(path.starts_with(&base));
        }
    }

    #[test]
    fn test_plugin_manifest(
        name in "[a-zA-Z0-9_-]+",
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
        description in ".*",
        dependencies in prop::collection::vec("[a-zA-Z0-9_-]+", 0..5),
    ) {
        let manifest = json!({
            "name": name,
            "version": version,
            "description": description,
            "dependencies": dependencies,
        });

        // Manifest should be valid JSON
        prop_assert!(serde_json::to_string(&manifest).is_ok());

        // Name should be valid
        prop_assert!(!name.is_empty());
        prop_assert!(!name.contains('\0'));

        // Version should be semver
        prop_assert!(version.split('.').count() == 3);
        for part in version.split('.') {
            prop_assert!(part.parse::<u32>().is_ok());
        }

        // Description should be valid
        prop_assert!(!description.contains('\0'));

        // Dependencies should be valid
        for dep in &dependencies {
            prop_assert!(!dep.is_empty());
            prop_assert!(!dep.contains('\0'));
        }
    }
}

#[test]
fn test_config_edge_cases() {
    let config = Config {
        repo_path: PathBuf::from(""),
        history_dir: PathBuf::from("."),
        output_dir: PathBuf::from("/"),
        plugins: vec![],
        ml_enabled: false,
    };

    assert!(serde_json::to_string(&config).is_ok());
}

#[test]
fn test_directory_structure_edge_cases() {
    let base = PathBuf::from(".");
    assert!(!base.to_string_lossy().contains('\0'));
    assert!(base.join("").starts_with(&base));
    assert!(base.join(".").starts_with(&base));
    assert!(base.join("/").starts_with(&base));
}

#[test]
fn test_plugin_manifest_edge_cases() {
    let manifest = json!({
        "name": "",
        "version": "0.0.0",
        "description": "",
        "dependencies": [],
    });

    assert!(serde_json::to_string(&manifest).is_ok());
} 