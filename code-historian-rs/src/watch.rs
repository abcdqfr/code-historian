use std::path::Path;
use std::time::Duration;
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event};
use tokio::sync::mpsc;
use crate::{Config, Result, HistorianError, analyzer::CodeAnalyzer};

pub struct WatchManager {
    config: Config,
    analyzer: CodeAnalyzer,
    debounce_duration: Duration,
}

impl WatchManager {
    pub fn new(config: Config, analyzer: CodeAnalyzer) -> Self {
        Self {
            config,
            analyzer,
            debounce_duration: Duration::from_secs(2),
        }
    }

    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    pub async fn watch(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        
        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            match res {
                Ok(event) => {
                    let _ = tx.blocking_send(event);
                }
                Err(e) => eprintln!("Watch error: {}", e),
            }
        })?;

        // Watch repository path
        watcher.watch(
            self.config.repo_path.as_ref(),
            RecursiveMode::Recursive,
        )?;

        let mut last_analysis = std::time::Instant::now();

        println!("Watching repository for changes...");
        println!("Press Ctrl+C to stop");

        while let Some(event) = rx.recv().await {
            if event.kind.is_modify() && 
               last_analysis.elapsed() >= self.debounce_duration {
                // Perform analysis
                match self.analyzer.analyze(&self.config) {
                    Ok(analysis) => {
                        println!("\nAnalysis update:");
                        println!("- Commits: {}", analysis.metrics.total_commits);
                        println!("- Changes: {}", analysis.metrics.total_changes);
                        println!("- Lines added: {}", analysis.metrics.lines_added);
                        println!("- Lines removed: {}", analysis.metrics.lines_removed);
                        
                        // Update visualizations if enabled
                        if self.config.visualization_enabled {
                            if let Some(output_dir) = self.config.output_dir.as_ref() {
                                let visualizer = crate::visualization::Visualizer::new(output_dir)?;
                                
                                if let Err(e) = visualizer.generate_timeline(&analysis) {
                                    eprintln!("Failed to update timeline: {}", e);
                                }
                                
                                if let Err(e) = visualizer.generate_category_distribution(&analysis) {
                                    eprintln!("Failed to update category distribution: {}", e);
                                }
                                
                                if let Err(e) = visualizer.generate_impact_timeline(&analysis.changes) {
                                    eprintln!("Failed to update impact timeline: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Analysis error: {}", e),
                }
                
                last_analysis = std::time::Instant::now();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_watch_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        let analyzer = CodeAnalyzer::new(vec![], false).unwrap();
        
        let manager = WatchManager::new(config, analyzer)
            .with_debounce(Duration::from_secs(1));
            
        assert_eq!(manager.debounce_duration, Duration::from_secs(1));
    }
} 