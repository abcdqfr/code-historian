use std::collections::HashMap;
use git2::{Repository, Diff, DiffOptions};
use crate::{
    Analyzer, Analysis, Change, Category, Config, Metrics, Pattern,
    Result, HistorianError, Plugin, AnalysisContext, PluginResult,
};

pub struct CodeAnalyzer {
    plugins: Vec<Box<dyn Plugin>>,
    ml_model: Option<crate::ml::ChangeClassifier>,
}

impl CodeAnalyzer {
    pub fn new(plugins: Vec<Box<dyn Plugin>>, use_ml: bool) -> Result<Self> {
        let ml_model = if use_ml {
            Some(crate::ml::ChangeClassifier::new()?)
        } else {
            None
        };

        Ok(Self {
            plugins,
            ml_model,
        })
    }

    fn analyze_commit(&self, repo: &Repository, commit: &git2::Commit) -> Result<Change> {
        let parent = commit.parent(0).map_err(|e| HistorianError::Git(e))?;
        let tree = commit.tree().map_err(|e| HistorianError::Git(e))?;
        let parent_tree = parent.tree().map_err(|e| HistorianError::Git(e))?;

        let mut diff_opts = DiffOptions::new();
        let diff = repo.diff_tree_to_tree(
            Some(&parent_tree),
            Some(&tree),
            Some(&mut diff_opts),
        ).map_err(|e| HistorianError::Git(e))?;

        let diff_text = self.get_diff_text(&diff)?;
        let categories = self.categorize(&diff_text)?;

        let change = Change {
            timestamp: commit.time().seconds().into(),
            author: commit.author().name().unwrap_or("unknown").to_string(),
            commit_id: commit.id().to_string(),
            message: commit.message().unwrap_or("").to_string(),
            diff: diff_text,
            categories,
            impact_score: 0.0, // Will be calculated later
        };

        Ok(change)
    }

    fn get_diff_text(&self, diff: &Diff) -> Result<String> {
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            use std::str;
            if let Ok(text) = str::from_utf8(line.content()) {
                diff_text.push_str(text);
            }
            true
        }).map_err(|e| HistorianError::Git(e))?;
        Ok(diff_text)
    }

    fn calculate_metrics(&self, changes: &[Change]) -> Metrics {
        let mut metrics = Metrics {
            total_commits: changes.len(),
            total_changes: 0,
            lines_added: 0,
            lines_removed: 0,
            category_distribution: HashMap::new(),
        };

        for change in changes {
            // Count lines added/removed
            for line in change.diff.lines() {
                if line.starts_with('+') {
                    metrics.lines_added += 1;
                } else if line.starts_with('-') {
                    metrics.lines_removed += 1;
                }
            }

            // Update category distribution
            for category in &change.categories {
                *metrics.category_distribution.entry(category.clone()).or_insert(0) += 1;
            }

            metrics.total_changes += 1;
        }

        metrics
    }

    fn detect_patterns(&self, changes: &[Change]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        let mut pattern_map: HashMap<String, (usize, f64)> = HashMap::new();

        for change in changes {
            // Look for common patterns in commit messages
            if change.message.contains("refactor") {
                let entry = pattern_map.entry("Refactoring".to_string()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += change.impact_score;
            }
            if change.message.contains("fix") {
                let entry = pattern_map.entry("Bug Fix".to_string()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += change.impact_score;
            }
            // Add more pattern detection logic here
        }

        for (name, (occurrences, impact)) in pattern_map {
            patterns.push(Pattern {
                name,
                description: "Detected pattern".to_string(), // Add proper descriptions
                occurrences,
                impact: impact / occurrences as f64,
            });
        }

        patterns
    }
}

impl Analyzer for CodeAnalyzer {
    fn analyze(&self, config: &Config) -> Result<Analysis> {
        let repo = Repository::open(&config.repo_path)
            .map_err(|e| HistorianError::Git(e))?;

        let mut revwalk = repo.revwalk()
            .map_err(|e| HistorianError::Git(e))?;
        revwalk.push_head()
            .map_err(|e| HistorianError::Git(e))?;

        let mut changes = Vec::new();

        for oid in revwalk {
            let oid = oid.map_err(|e| HistorianError::Git(e))?;
            let commit = repo.find_commit(oid)
                .map_err(|e| HistorianError::Git(e))?;

            let change = self.analyze_commit(&repo, &commit)?;
            changes.push(change);
        }

        // Calculate metrics and detect patterns
        let metrics = self.calculate_metrics(&changes);
        let patterns = self.detect_patterns(&changes);

        Ok(Analysis {
            changes,
            metrics,
            patterns,
        })
    }

    fn categorize(&self, diff: &str) -> Result<Vec<Category>> {
        let mut categories = Vec::new();

        // Use ML model if available
        if let Some(ref model) = self.ml_model {
            categories = model.predict_categories(diff)?;
        } else {
            // Fallback to rule-based categorization
            if diff.contains("class") || diff.contains("struct") {
                categories.push(Category::Architecture);
            }
            if diff.contains("fn") || diff.contains("pub") {
                categories.push(Category::Api);
            }
            if diff.contains("if") || diff.contains("match") {
                categories.push(Category::Logic);
            }
            // Add more rules here
        }

        // Run plugins
        for plugin in &self.plugins {
            let context = AnalysisContext {
                config: &Config {
                    repo_path: std::path::PathBuf::new(),
                    output_dir: std::path::PathBuf::new(),
                    plugins: vec![],
                    ml_enabled: false,
                    visualization_enabled: false,
                    recursive: false,
                    file_pattern: None,
                },
                change: &Change {
                    timestamp: std::time::SystemTime::now().into(),
                    author: String::new(),
                    commit_id: String::new(),
                    message: String::new(),
                    diff: diff.to_string(),
                    categories: vec![],
                    impact_score: 0.0,
                },
                repository: &Repository::open_from_env()?,
            };

            let result = plugin.analyze(&context)?;
            categories.extend(result.categories);
        }

        Ok(categories)
    }

    fn calculate_impact(&self, change: &Change) -> f64 {
        let mut impact = 0.0;

        // Basic impact calculation based on changes
        let lines_changed = change.diff.lines().count() as f64;
        impact += lines_changed * 0.1;

        // Adjust based on categories
        for category in &change.categories {
            impact += match category {
                Category::Architecture => 2.0,
                Category::Security => 1.8,
                Category::Performance => 1.5,
                _ => 1.0,
            };
        }

        // Normalize impact score
        impact.min(10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_basic() {
        let analyzer = CodeAnalyzer::new(vec![], false).unwrap();
        let diff = "
            + pub fn new_function() {
            +     if true {
            +         // do something
            +     }
            + }
        ";
        let categories = analyzer.categorize(diff).unwrap();
        assert!(categories.contains(&Category::Api));
        assert!(categories.contains(&Category::Logic));
    }

    #[test]
    fn test_impact_calculation() {
        let analyzer = CodeAnalyzer::new(vec![], false).unwrap();
        let change = Change {
            timestamp: std::time::SystemTime::now().into(),
            author: "test".to_string(),
            commit_id: "test".to_string(),
            message: "test".to_string(),
            diff: "test\ntest\ntest".to_string(),
            categories: vec![Category::Architecture, Category::Security],
            impact_score: 0.0,
        };
        let impact = analyzer.calculate_impact(&change);
        assert!(impact > 0.0);
        assert!(impact <= 10.0);
    }
} 