use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Serialize;
use crate::{Analysis, Change, Result, HistorianError};

#[derive(Debug, Clone, Serialize)]
pub struct LazyLoadOptions {
    pub initial_load: usize,
    pub batch_size: usize,
    pub preload_threshold: f32,
}

impl Default for LazyLoadOptions {
    fn default() -> Self {
        Self {
            initial_load: 50,
            batch_size: 25,
            preload_threshold: 0.8,
        }
    }
}

pub struct LazyVisualizer {
    options: LazyLoadOptions,
    data: Arc<RwLock<Vec<Change>>>,
    loaded_count: Arc<RwLock<usize>>,
}

impl LazyVisualizer {
    pub fn new(options: Option<LazyLoadOptions>) -> Self {
        Self {
            options: options.unwrap_or_default(),
            data: Arc::new(RwLock::new(Vec::new())),
            loaded_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn initialize(&self, analysis: Arc<Analysis>) -> Result<()> {
        let mut data = self.data.write().await;
        let initial_load = self.options.initial_load.min(analysis.changes.len());
        *data = analysis.changes[..initial_load].to_vec();
        
        let mut loaded_count = self.loaded_count.write().await;
        *loaded_count = initial_load;

        Ok(())
    }

    pub async fn load_more(&self, analysis: Arc<Analysis>) -> Result<bool> {
        let current_count = *self.loaded_count.read().await;
        if current_count >= analysis.changes.len() {
            return Ok(false);
        }

        let mut data = self.data.write().await;
        let mut loaded_count = self.loaded_count.write().await;
        
        let start = current_count;
        let end = (start + self.options.batch_size).min(analysis.changes.len());
        
        data.extend_from_slice(&analysis.changes[start..end]);
        *loaded_count = end;

        Ok(true)
    }

    pub async fn get_loaded_data(&self) -> Result<Vec<Change>> {
        Ok(self.data.read().await.clone())
    }

    pub async fn get_loading_progress(&self, analysis: Arc<Analysis>) -> Result<f32> {
        let loaded = *self.loaded_count.read().await;
        let total = analysis.changes.len();
        
        Ok(loaded as f32 / total as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::Category;

    #[tokio::test]
    async fn test_lazy_loading() {
        let changes = (0..100).map(|i| Change {
            timestamp: Utc::now(),
            author: format!("author{}", i),
            commit_id: format!("commit{}", i),
            message: format!("message{}", i),
            diff: format!("diff{}", i),
            categories: vec![Category::Feature],
            impact_score: (i as f64) / 10.0,
        }).collect();

        let analysis = Arc::new(Analysis {
            changes,
            metrics: crate::Metrics {
                total_commits: 100,
                total_changes: 100,
                lines_added: 500,
                lines_removed: 200,
                category_distribution: Default::default(),
            },
            patterns: vec![],
        });

        let options = LazyLoadOptions {
            initial_load: 30,
            batch_size: 20,
            preload_threshold: 0.8,
        };

        let visualizer = LazyVisualizer::new(Some(options));
        
        // Test initialization
        visualizer.initialize(analysis.clone()).await.unwrap();
        let initial_data = visualizer.get_loaded_data().await.unwrap();
        assert_eq!(initial_data.len(), 30);

        // Test loading more data
        let more_loaded = visualizer.load_more(analysis.clone()).await.unwrap();
        assert!(more_loaded);
        
        let updated_data = visualizer.get_loaded_data().await.unwrap();
        assert_eq!(updated_data.len(), 50);

        // Test progress calculation
        let progress = visualizer.get_loading_progress(analysis.clone()).await.unwrap();
        assert!((progress - 0.5).abs() < f32::EPSILON);
    }
} 