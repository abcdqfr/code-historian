use std::path::{Path, PathBuf};
use std::fs;
use assert_fs::TempDir;
use git2::{Repository, Signature};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConfig {
    pub repo_path: PathBuf,
    pub history_dir: PathBuf,
    pub output_dir: PathBuf,
    pub plugins: Vec<String>,
    pub ml_enabled: bool,
}

pub struct TestRepository {
    pub path: PathBuf,
    pub repo: Repository,
    temp_dir: TempDir,
}

impl TestRepository {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        
        // Create initial commit
        let signature = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();

        Self {
            path: temp_dir.path().to_path_buf(),
            repo,
            temp_dir,
        }
    }

    pub fn add_file(&self, name: &str, content: &str) {
        let file_path = self.path.join(name);
        fs::write(&file_path, content).unwrap();
        
        let mut index = self.repo.index().unwrap();
        index.add_path(Path::new(name)).unwrap();
        index.write().unwrap();
        
        let signature = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();
        let parent = self.repo.head().unwrap().peel_to_commit().unwrap();
        
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Add {}", name),
            &tree,
            &[&parent],
        ).unwrap();
    }

    pub fn modify_file(&self, name: &str, content: &str) {
        let file_path = self.path.join(name);
        fs::write(&file_path, content).unwrap();
        
        let mut index = self.repo.index().unwrap();
        index.add_path(Path::new(name)).unwrap();
        index.write().unwrap();
        
        let signature = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();
        let parent = self.repo.head().unwrap().peel_to_commit().unwrap();
        
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Modify {}", name),
            &tree,
            &[&parent],
        ).unwrap();
    }
}

pub fn create_test_config() -> TestConfig {
    TestConfig {
        repo_path: PathBuf::from("test-repo"),
        history_dir: PathBuf::from(".code-historian"),
        output_dir: PathBuf::from("output"),
        plugins: vec!["test-plugin".to_string()],
        ml_enabled: true,
    }
}

pub fn create_test_plugin() -> String {
    r#"
    use code_historian::plugin::{Plugin, AnalysisContext, PluginResult};

    pub struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn analyze(&self, context: &AnalysisContext) -> PluginResult {
            // Test plugin implementation
            PluginResult::default()
        }
    }
    "#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        let repo = TestRepository::new();
        assert!(repo.path.exists());
        assert!(repo.path.join(".git").exists());
    }

    #[test]
    fn test_file_operations() {
        let repo = TestRepository::new();
        
        // Add a file
        repo.add_file("test.txt", "Hello, World!");
        assert!(repo.path.join("test.txt").exists());
        
        // Modify the file
        repo.modify_file("test.txt", "Modified content");
        let content = fs::read_to_string(repo.path.join("test.txt")).unwrap();
        assert_eq!(content, "Modified content");
    }

    #[test]
    fn test_config_creation() {
        let config = create_test_config();
        assert_eq!(config.repo_path, PathBuf::from("test-repo"));
        assert_eq!(config.history_dir, PathBuf::from(".code-historian"));
        assert!(config.plugins.contains(&"test-plugin".to_string()));
    }

    #[test]
    fn test_plugin_creation() {
        let plugin_code = create_test_plugin();
        assert!(plugin_code.contains("impl Plugin for TestPlugin"));
        assert!(plugin_code.contains("fn name(&self) -> &str"));
        assert!(plugin_code.contains("fn analyze(&self, context: &AnalysisContext)"));
    }
} 