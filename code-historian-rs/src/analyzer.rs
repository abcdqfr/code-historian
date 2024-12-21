use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use git2::{Repository, Commit, DiffOptions};
use serde::{Serialize, Deserialize};
use crate::{Result, HistorianError, Config, Category, PluginManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub changes: Vec<Change>,
    pub metrics: Metrics,
    pub patterns: Vec<Pattern>,
    pub cache_info: Option<CacheInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub commit_id: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub file_path: PathBuf,
    pub diff: String,
    pub categories: Vec<Category>,
    pub impact_score: f64,
    pub metrics: HashMap<String, f64>,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_commits: usize,
    pub total_changes: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub category_distribution: HashMap<Category, usize>,
    pub impact_distribution: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub occurrences: usize,
    pub impact: f64,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub last_commit: String,
    pub timestamp: DateTime<Utc>,
    pub paths_analyzed: Vec<PathBuf>,
}

pub struct Analyzer {
    config: Config,
    plugin_manager: PluginManager,
    cache: Option<Analysis>,
}

impl Analyzer {
    pub fn new(config: Config, plugin_manager: PluginManager) -> Self {
        Self {
            config,
            plugin_manager,
            cache: None,
        }
    }

    pub fn analyze(&mut self, repo_path: &Path, paths: Option<Vec<PathBuf>>) -> Result<Analysis> {
        let repo = Repository::open(repo_path)?;
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        // Try to load cache
        if self.config.cache.enabled {
            if let Some(cached) = self.load_cache(repo_path)? {
                if let Some(cache_info) = &cached.cache_info {
                    if cache_info.last_commit == head_commit.id().to_string() {
                        // Cache is still valid
                        if let Some(paths) = &paths {
                            // Check if all requested paths are in cache
                            if paths.iter().all(|p| cache_info.paths_analyzed.contains(p)) {
                                return Ok(cached);
                            }
                        } else {
                            return Ok(cached);
                        }
                    }
                }
            }
        }

        let mut analysis = Analysis {
            changes: Vec::new(),
            metrics: Metrics {
                total_commits: 0,
                total_changes: 0,
                lines_added: 0,
                lines_removed: 0,
                category_distribution: HashMap::new(),
                impact_distribution: HashMap::new(),
            },
            patterns: Vec::new(),
            cache_info: None,
        };

        // Set up diff options
        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(true);
        
        if let Some(paths) = paths {
            for path in paths {
                diff_opts.pathspec(path);
            }
        }

        // Analyze commits
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk {
            let commit_id = oid?;
            let commit = repo.find_commit(commit_id)?;
            
            if let Some(parent) = commit.parent(0).ok() {
                let diff = repo.diff_tree_to_tree(
                    Some(&parent.tree()?),
                    Some(&commit.tree()?),
                    Some(&mut diff_opts),
                )?;

                let changes = self.analyze_commit(&commit, &diff)?;
                analysis.changes.extend(changes);
            }

            analysis.metrics.total_commits += 1;
        }

        // Update metrics
        self.update_metrics(&mut analysis)?;

        // Detect patterns
        self.detect_patterns(&mut analysis)?;

        // Update cache info
        if self.config.cache.enabled {
            analysis.cache_info = Some(CacheInfo {
                last_commit: head_commit.id().to_string(),
                timestamp: Utc::now(),
                paths_analyzed: paths.unwrap_or_else(Vec::new),
            });

            // Save to cache
            self.save_cache(repo_path, &analysis)?;
        }

        Ok(analysis)
    }

    fn analyze_commit(&self, commit: &Commit, diff: &git2::Diff) -> Result<Vec<Change>> {
        let mut changes = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    let file_path = PathBuf::from(path);
                    
                    // Check if file matches include/exclude patterns
                    if self.should_analyze_file(&file_path) {
                        // Get diff as text
                        if let Ok(diff_text) = diff.to_string() {
                            // Create change record
                            let change = Change {
                                commit_id: commit.id().to_string(),
                                author: commit.author().name().unwrap_or("unknown").to_string(),
                                timestamp: DateTime::from_timestamp(commit.time().seconds(), 0)
                                    .unwrap_or_else(|| Utc::now()),
                                message: commit.message().unwrap_or("").to_string(),
                                file_path,
                                diff: diff_text,
                                categories: Vec::new(),
                                impact_score: 0.0,
                                metrics: HashMap::new(),
                                annotations: Vec::new(),
                            };
                            changes.push(change);
                        }
                    }
                }
                true
            },
            None,
            None,
            None,
        )?;

        Ok(changes)
    }

    fn should_analyze_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude patterns
        for pattern in &self.config.analysis.exclude_patterns {
            if glob::Pattern::new(pattern).unwrap().matches(&path_str) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.config.analysis.include_patterns {
            if glob::Pattern::new(pattern).unwrap().matches(&path_str) {
                return true;
            }
        }

        false
    }

    fn update_metrics(&self, analysis: &mut Analysis) -> Result<()> {
        for change in &analysis.changes {
            analysis.metrics.total_changes += 1;

            // Count lines added/removed
            for line in change.diff.lines() {
                if line.starts_with('+') {
                    analysis.metrics.lines_added += 1;
                } else if line.starts_with('-') {
                    analysis.metrics.lines_removed += 1;
                }
            }

            // Update category distribution
            for category in &change.categories {
                *analysis.metrics.category_distribution
                    .entry(category.clone())
                    .or_insert(0) += 1;
            }

            // Update impact distribution
            analysis.metrics.impact_distribution
                .insert(change.file_path.to_string_lossy().into_owned(), change.impact_score);
        }

        Ok(())
    }

    fn detect_patterns(&self, analysis: &mut Analysis) -> Result<()> {
        let mut patterns = HashMap::new();

        for change in &analysis.changes {
            // Analyze with plugins
            let context = crate::plugin::AnalysisContext {
                file_path: &change.file_path,
                content: &change.diff,
                diff: Some(&change.diff),
                language: None, // TODO: Detect language
                config: None,
            };

            for plugin in self.plugin_manager.get_plugins() {
                if let Ok(result) = plugin.analyze(&context) {
                    for pattern in result.patterns {
                        let entry = patterns.entry(pattern).or_insert_with(|| Pattern {
                            name: pattern.clone(),
                            description: String::new(),
                            occurrences: 0,
                            impact: 0.0,
                            examples: Vec::new(),
                        });

                        entry.occurrences += 1;
                        entry.impact += change.impact_score;
                        entry.examples.push(format!(
                            "{}:{} - {}",
                            change.file_path.display(),
                            change.commit_id,
                            change.message
                        ));
                    }
                }
            }
        }

        analysis.patterns = patterns.into_values().collect();
        Ok(())
    }

    fn load_cache(&self, repo_path: &Path) -> Result<Option<Analysis>> {
        let cache_path = self.config.cache_dir()?.join("analysis.json");
        if cache_path.exists() {
            let content = std::fs::read_to_string(cache_path)?;
            Ok(Some(serde_json::from_str(&content)?))
        } else {
            Ok(None)
        }
    }

    fn save_cache(&self, repo_path: &Path, analysis: &Analysis) -> Result<()> {
        let cache_path = self.config.cache_dir()?;
        std::fs::create_dir_all(&cache_path)?;
        
        let content = serde_json::to_string_pretty(analysis)?;
        std::fs::write(cache_path.join("analysis.json"), content)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_analyze_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let plugin_manager = PluginManager::new(temp_dir.path().to_path_buf());
        let analyzer = Analyzer::new(config, plugin_manager);

        assert!(analyzer.should_analyze_file(Path::new("src/main.rs")));
        assert!(analyzer.should_analyze_file(Path::new("Cargo.toml")));
        assert!(!analyzer.should_analyze_file(Path::new("target/debug/main")));
    }

    #[test]
    fn test_cache_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.cache.enabled = true;
        
        let plugin_manager = PluginManager::new(temp_dir.path().to_path_buf());
        let analyzer = Analyzer::new(config, plugin_manager);

        // Test cache save/load
        let analysis = Analysis {
            changes: Vec::new(),
            metrics: Metrics {
                total_commits: 0,
                total_changes: 0,
                lines_added: 0,
                lines_removed: 0,
                category_distribution: HashMap::new(),
                impact_distribution: HashMap::new(),
            },
            patterns: Vec::new(),
            cache_info: Some(CacheInfo {
                last_commit: "test".to_string(),
                timestamp: Utc::now(),
                paths_analyzed: vec![PathBuf::from("test.rs")],
            }),
        };

        analyzer.save_cache(temp_dir.path(), &analysis).unwrap();
        let loaded = analyzer.load_cache(temp_dir.path()).unwrap().unwrap();

        assert_eq!(
            loaded.cache_info.unwrap().last_commit,
            analysis.cache_info.unwrap().last_commit
        );
    }
} 