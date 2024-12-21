use std::sync::Arc;
use tokio::sync::mpsc;
use serde::Serialize;
use crate::{Analysis, Change, Result, HistorianError};

const CHUNK_SIZE: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct DataChunk {
    pub changes: Vec<Change>,
    pub is_last: bool,
}

pub struct StreamingVisualizer {
    chunk_size: usize,
}

impl StreamingVisualizer {
    pub fn new(chunk_size: Option<usize>) -> Self {
        Self {
            chunk_size: chunk_size.unwrap_or(CHUNK_SIZE),
        }
    }

    pub async fn stream_changes(&self, analysis: Arc<Analysis>) -> Result<mpsc::Receiver<DataChunk>> {
        let (tx, rx) = mpsc::channel(4);
        let chunk_size = self.chunk_size;

        tokio::spawn(async move {
            let chunks = analysis.changes.chunks(chunk_size);
            let total_chunks = chunks.len();

            for (i, chunk) in chunks.enumerate() {
                let is_last = i == total_chunks - 1;
                let chunk_data = DataChunk {
                    changes: chunk.to_vec(),
                    is_last,
                };

                if tx.send(chunk_data).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::Category;

    #[tokio::test]
    async fn test_streaming() {
        let changes = (0..250).map(|i| Change {
            timestamp: Utc::now(),
            author: format!("author{}", i),
            commit_id: format!("commit{}", i),
            message: format!("message{}", i),
            diff: format!("diff{}", i),
            categories: vec![Category::Feature],
            impact_score: (i as f64) / 25.0,
        }).collect();

        let analysis = Arc::new(Analysis {
            changes,
            metrics: crate::Metrics {
                total_commits: 250,
                total_changes: 250,
                lines_added: 1000,
                lines_removed: 500,
                category_distribution: Default::default(),
            },
            patterns: vec![],
        });

        let visualizer = StreamingVisualizer::new(Some(50));
        let mut rx = visualizer.stream_changes(analysis).await.unwrap();

        let mut total_changes = 0;
        let mut chunks_received = 0;
        let mut last_chunk_seen = false;

        while let Some(chunk) = rx.recv().await {
            total_changes += chunk.changes.len();
            chunks_received += 1;
            if chunk.is_last {
                last_chunk_seen = true;
            }
        }

        assert_eq!(total_changes, 250);
        assert_eq!(chunks_received, 5);
        assert!(last_chunk_seen);
    }
} 