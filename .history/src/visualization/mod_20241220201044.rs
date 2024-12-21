mod streaming;
mod lazy;

pub use streaming::{StreamingVisualizer, DataChunk};
pub use lazy::{LazyVisualizer, LazyLoadOptions};

use std::sync::Arc;
use tokio::sync::mpsc;
use serde::Serialize;
use crate::{Analysis, Change, Result};

#[derive(Debug, Clone, Serialize)]
pub struct VisualizationOptions {
    pub width: u32,
    pub height: u32,
    pub show_legend: bool,
    pub interactive: bool,
    pub streaming: Option<StreamingOptions>,
    pub lazy_loading: Option<LazyLoadOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamingOptions {
    pub chunk_size: usize,
    pub buffer_size: usize,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            show_legend: true,
            interactive: true,
            streaming: Some(StreamingOptions {
                chunk_size: 100,
                buffer_size: 4,
            }),
            lazy_loading: Some(LazyLoadOptions::default()),
        }
    }
}

pub struct Visualizer {
    options: VisualizationOptions,
    streaming: Option<StreamingVisualizer>,
    lazy: Option<LazyVisualizer>,
}

impl Visualizer {
    pub fn new(options: Option<VisualizationOptions>) -> Self {
        let options = options.unwrap_or_default();
        let streaming = options.streaming.as_ref().map(|opts| {
            StreamingVisualizer::new(Some(opts.chunk_size))
        });
        let lazy = options.lazy_loading.as_ref().map(|opts| {
            LazyVisualizer::new(Some(opts.clone()))
        });

        Self {
            options,
            streaming,
            lazy,
        }
    }

    pub async fn stream_changes(&self, analysis: Arc<Analysis>) -> Result<Option<mpsc::Receiver<DataChunk>>> {
        if let Some(streaming) = &self.streaming {
            Ok(Some(streaming.stream_changes(analysis).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn initialize_lazy(&self, analysis: Arc<Analysis>) -> Result<()> {
        if let Some(lazy) = &self.lazy {
            lazy.initialize(analysis).await?;
        }
        Ok(())
    }

    pub async fn load_more_lazy(&self, analysis: Arc<Analysis>) -> Result<bool> {
        if let Some(lazy) = &self.lazy {
            Ok(lazy.load_more(analysis).await?)
        } else {
            Ok(false)
        }
    }

    pub async fn get_lazy_data(&self) -> Result<Option<Vec<Change>>> {
        if let Some(lazy) = &self.lazy {
            Ok(Some(lazy.get_loaded_data().await?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_loading_progress(&self, analysis: Arc<Analysis>) -> Result<Option<f32>> {
        if let Some(lazy) = &self.lazy {
            Ok(Some(lazy.get_loading_progress(analysis).await?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::Category;

    #[tokio::test]
    async fn test_visualization_modes() {
        let changes = (0..200).map(|i| Change {
            timestamp: Utc::now(),
            author: format!("author{}", i),
            commit_id: format!("commit{}", i),
            message: format!("message{}", i),
            diff: format!("diff{}", i),
            categories: vec![Category::Feature],
            impact_score: (i as f64) / 20.0,
        }).collect();

        let analysis = Arc::new(Analysis {
            changes,
            metrics: crate::Metrics {
                total_commits: 200,
                total_changes: 200,
                lines_added: 800,
                lines_removed: 400,
                category_distribution: Default::default(),
            },
            patterns: vec![],
        });

        // Test streaming mode
        let options = VisualizationOptions {
            streaming: Some(StreamingOptions {
                chunk_size: 50,
                buffer_size: 2,
            }),
            ..Default::default()
        };

        let visualizer = Visualizer::new(Some(options));
        
        if let Some(mut rx) = visualizer.stream_changes(analysis.clone()).await.unwrap() {
            let mut total_changes = 0;
            while let Some(chunk) = rx.recv().await {
                total_changes += chunk.changes.len();
            }
            assert_eq!(total_changes, 200);
        }

        // Test lazy loading mode
        let options = VisualizationOptions {
            lazy_loading: Some(LazyLoadOptions {
                initial_load: 40,
                batch_size: 30,
                preload_threshold: 0.8,
            }),
            streaming: None,
            ..Default::default()
        };

        let visualizer = Visualizer::new(Some(options));
        visualizer.initialize_lazy(analysis.clone()).await.unwrap();
        
        let initial_data = visualizer.get_lazy_data().await.unwrap().unwrap();
        assert_eq!(initial_data.len(), 40);

        visualizer.load_more_lazy(analysis.clone()).await.unwrap();
        let updated_data = visualizer.get_lazy_data().await.unwrap().unwrap();
        assert_eq!(updated_data.len(), 70);
    }
} 