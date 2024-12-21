use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use git2::{Repository, Commit, DiffOptions, Oid};
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
    pub incremental_state: Option<IncrementalState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalState {
    pub analyzed_commits: Vec<String>,
    pub last_analysis_time: DateTime<Utc>,
    pub partial_metrics: Metrics,
    pub known_patterns: Vec<Pattern>,
}

pub struct Analyzer {
    config: Config,
    plugin_manager: PluginManager,
    cache: Option<Analysis>,
    incremental_state: Option<IncrementalState>,
}

impl Analyzer {
    pub fn new(config: Config, plugin_manager: PluginManager) -> Self {
        Self {
            config,
            plugin_manager,
            cache: None,
            incremental_state: None,
        }
    }

    pub fn analyze(&mut self, repo_path: &Path, paths: Option<Vec<PathBuf>>) -> Result<Analysis> {
        let repo = Repository::open(repo_path)?;
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        // Try to load cache and incremental state
        if self.config.cache.enabled {
            if let Some(cached) = self.load_cache(repo_path)? {
                if let Some(cache_info) = &cached.cache_info {
                    // Load incremental state if available
                    if let Some(inc_state) = &cache_info.incremental_state {
                        self.incremental_state = Some(inc_state.clone());
                    }

                    // Check if cache is still valid
                    if cache_info.last_commit == head_commit.id().to_string() {
                        if let Some(paths) = &paths {
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

        let mut analysis = if let Some(inc_state) = &self.incremental_state {
            // Start from previous state
            Analysis {
                changes: Vec::new(),
                metrics: inc_state.partial_metrics.clone(),
                patterns: inc_state.known_patterns.clone(),
                cache_info: None,
            }
        } else {
            // Start fresh
            Analysis {
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
            }
        };

        // Set up diff options
        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(true);
        
        if let Some(paths) = &paths {
            for path in paths {
                diff_opts.pathspec(path);
            }
        }

        // Get analyzed commits from incremental state
        let analyzed_commits: Vec<String> = self.incremental_state
            .as_ref()
            .map(|s| s.analyzed_commits.clone())
            .unwrap_or_default();

        // Analyze commits
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk {
            let commit_id = oid?;
            let commit = repo.find_commit(commit_id)?;
            
            // Skip already analyzed commits
            if analyzed_commits.contains(&commit.id().to_string()) {
                continue;
            }

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

        // Update cache info and incremental state
        if self.config.cache.enabled {
            let mut analyzed_commits = analyzed_commits;
            analyzed_commits.extend(
                analysis.changes
                    .iter()
                    .map(|c| c.commit_id.clone())
            );

            let incremental_state = IncrementalState {
                analyzed_commits,
                last_analysis_time: Utc::now(),
                partial_metrics: analysis.metrics.clone(),
                known_patterns: analysis.patterns.clone(),
            };

            analysis.cache_info = Some(CacheInfo {
                last_commit: head_commit.id().to_string(),
                timestamp: Utc::now(),
                paths_analyzed: paths.unwrap_or_else(Vec::new),
                incremental_state: Some(incremental_state.clone()),
            });

            // Update internal state
            self.incremental_state = Some(incremental_state);

            // Save to cache
            self.save_cache(repo_path, &analysis)?;
        }

        Ok(analysis)
    }

    // ... existing methods ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_incremental_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            cache: crate::config::CacheConfig {
                enabled: true,
                directory: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        };

        let plugin_manager = PluginManager::new();
        let mut analyzer = Analyzer::new(config, plugin_manager);

        // First analysis
        let analysis1 = analyzer.analyze(temp_dir.path(), None).unwrap();

        // Modify repository...

        // Second analysis should be incremental
        let analysis2 = analyzer.analyze(temp_dir.path(), None).unwrap();

        assert!(analysis2.cache_info.is_some());
        assert!(analysis2.cache_info.unwrap().incremental_state.is_some());
    }
} 