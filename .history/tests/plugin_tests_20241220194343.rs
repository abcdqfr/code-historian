use std::path::PathBuf;
use assert_fs::TempDir;
use code_historian::{Config, PluginManager, Plugin, AnalysisContext, PluginResult};

struct TestPlugin {
    name: String,
    dependencies: Vec<String>,
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn analyze(&self, _context: &AnalysisContext) -> PluginResult {
        PluginResult::default()
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

#[test]
fn test_plugin_loading() {
    let temp = TempDir::new().unwrap();
    let plugins_dir = temp.path().join("plugins");
    std::fs::create_dir(&plugins_dir).unwrap();

    // Create a test plugin
    let plugin_code = r#"
    use code_historian::plugin::{Plugin, AnalysisContext, PluginResult};

    pub struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn analyze(&self, _context: &AnalysisContext) -> PluginResult {
            PluginResult::default()
        }
    }
    "#;

    std::fs::write(plugins_dir.join("test_plugin.rs"), plugin_code).unwrap();

    let config = Config {
        repo_path: PathBuf::from("."),
        history_dir: PathBuf::from(".code-historian"),
        output_dir: PathBuf::from("output"),
        plugins: vec!["test-plugin".to_string()],
        ml_enabled: false,
    };

    let plugin_manager = PluginManager::new();
    plugin_manager.load_plugins(&plugins_dir).unwrap();

    assert!(plugin_manager.get_plugin("test-plugin").is_some());
}

#[test]
fn test_plugin_execution() {
    let plugin = TestPlugin {
        name: "test-plugin".to_string(),
        dependencies: vec![],
    };

    let context = AnalysisContext {
        repo_path: PathBuf::from("."),
        history_dir: PathBuf::from(".code-historian"),
        output_dir: PathBuf::from("output"),
    };

    let result = plugin.analyze(&context);
    assert!(result.is_ok());
}

#[test]
fn test_plugin_dependencies() {
    let plugin_a = TestPlugin {
        name: "plugin-a".to_string(),
        dependencies: vec!["plugin-b".to_string()],
    };

    let plugin_b = TestPlugin {
        name: "plugin-b".to_string(),
        dependencies: vec![],
    };

    let mut plugin_manager = PluginManager::new();
    plugin_manager.register_plugin(Box::new(plugin_a));
    plugin_manager.register_plugin(Box::new(plugin_b));

    // Verify dependency resolution
    let execution_order = plugin_manager.resolve_dependencies().unwrap();
    assert!(
        execution_order.iter().position(|p| p.name() == "plugin-b")
            < execution_order.iter().position(|p| p.name() == "plugin-a")
    );
}

#[test]
fn test_plugin_dependency_cycle() {
    let plugin_a = TestPlugin {
        name: "plugin-a".to_string(),
        dependencies: vec!["plugin-b".to_string()],
    };

    let plugin_b = TestPlugin {
        name: "plugin-b".to_string(),
        dependencies: vec!["plugin-a".to_string()],
    };

    let mut plugin_manager = PluginManager::new();
    plugin_manager.register_plugin(Box::new(plugin_a));
    plugin_manager.register_plugin(Box::new(plugin_b));

    // Verify that cyclic dependencies are detected
    assert!(plugin_manager.resolve_dependencies().is_err());
}

#[test]
fn test_plugin_missing_dependency() {
    let plugin = TestPlugin {
        name: "plugin-a".to_string(),
        dependencies: vec!["non-existent-plugin".to_string()],
    };

    let mut plugin_manager = PluginManager::new();
    plugin_manager.register_plugin(Box::new(plugin));

    // Verify that missing dependencies are detected
    assert!(plugin_manager.resolve_dependencies().is_err());
}

#[test]
fn test_plugin_dynamic_loading() {
    let temp = TempDir::new().unwrap();
    let plugins_dir = temp.path().join("plugins");
    std::fs::create_dir(&plugins_dir).unwrap();

    // Create multiple test plugins
    let plugin_a_code = r#"
    use code_historian::plugin::{Plugin, AnalysisContext, PluginResult};

    pub struct PluginA;

    impl Plugin for PluginA {
        fn name(&self) -> &str {
            "plugin-a"
        }

        fn analyze(&self, _context: &AnalysisContext) -> PluginResult {
            PluginResult::default()
        }

        fn dependencies(&self) -> &[String] {
            &["plugin-b"]
        }
    }
    "#;

    let plugin_b_code = r#"
    use code_historian::plugin::{Plugin, AnalysisContext, PluginResult};

    pub struct PluginB;

    impl Plugin for PluginB {
        fn name(&self) -> &str {
            "plugin-b"
        }

        fn analyze(&self, _context: &AnalysisContext) -> PluginResult {
            PluginResult::default()
        }
    }
    "#;

    std::fs::write(plugins_dir.join("plugin_a.rs"), plugin_a_code).unwrap();
    std::fs::write(plugins_dir.join("plugin_b.rs"), plugin_b_code).unwrap();

    let plugin_manager = PluginManager::new();
    plugin_manager.load_plugins(&plugins_dir).unwrap();

    assert!(plugin_manager.get_plugin("plugin-a").is_some());
    assert!(plugin_manager.get_plugin("plugin-b").is_some());

    // Verify dependency resolution after dynamic loading
    let execution_order = plugin_manager.resolve_dependencies().unwrap();
    assert!(
        execution_order.iter().position(|p| p.name() == "plugin-b")
            < execution_order.iter().position(|p| p.name() == "plugin-a")
    );
} 