use std::sync::Arc;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};
use crate::{Analysis, Result, visualization::Visualizer};

#[derive(Debug, Clone, Serialize)]
pub struct DashboardUpdate {
    pub analysis_updates: Vec<AnalysisUpdate>,
    pub team_updates: Vec<TeamUpdate>,
    pub project_updates: Vec<ProjectUpdate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisUpdate {
    pub id: String,
    pub progress: f32,
    pub status: String,
    pub metrics: Option<crate::Metrics>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamUpdate {
    pub member_id: String,
    pub action: String,
    pub project_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectUpdate {
    pub id: String,
    pub action: String,
    pub metrics: Option<crate::Metrics>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardMetrics {
    pub active_analyses: usize,
    pub total_team_members: usize,
    pub total_projects: usize,
    pub recent_activity: Vec<ActivityEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub details: String,
}

pub struct Dashboard {
    update_tx: broadcast::Sender<DashboardUpdate>,
    visualizer: Arc<Visualizer>,
}

impl Dashboard {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            update_tx: tx,
            visualizer: Arc::new(Visualizer::new(None)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DashboardUpdate> {
        self.update_tx.subscribe()
    }

    pub async fn update_analysis(&self, update: AnalysisUpdate) -> Result<()> {
        let dashboard_update = DashboardUpdate {
            analysis_updates: vec![update],
            team_updates: Vec::new(),
            project_updates: Vec::new(),
        };
        
        let _ = self.update_tx.send(dashboard_update);
        Ok(())
    }

    pub async fn update_team(&self, update: TeamUpdate) -> Result<()> {
        let dashboard_update = DashboardUpdate {
            analysis_updates: Vec::new(),
            team_updates: vec![update],
            project_updates: Vec::new(),
        };
        
        let _ = self.update_tx.send(dashboard_update);
        Ok(())
    }

    pub async fn update_project(&self, update: ProjectUpdate) -> Result<()> {
        let dashboard_update = DashboardUpdate {
            analysis_updates: Vec::new(),
            team_updates: Vec::new(),
            project_updates: vec![update],
        };
        
        let _ = self.update_tx.send(dashboard_update);
        Ok(())
    }

    pub async fn create_visualization(&self, analysis: Arc<Analysis>) -> Result<String> {
        // Initialize streaming visualization
        let mut rx = self.visualizer.stream_changes(analysis.clone()).await?
            .ok_or_else(|| crate::HistorianError::Analysis("Streaming not enabled".into()))?;

        let mut html = String::new();
        html.push_str(include_str!("../../templates/dashboard_chart.html"));

        // Process data chunks
        while let Some(chunk) = rx.recv().await {
            // Update visualization data
            let data_update = serde_json::to_string(&chunk.changes)?;
            html.push_str(&format!(
                "<script>updateVisualization({});</script>",
                data_update
            ));

            if chunk.is_last {
                break;
            }
        }

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_dashboard_updates() {
        let dashboard = Dashboard::new(10);
        let mut rx1 = dashboard.subscribe();
        let mut rx2 = dashboard.subscribe();

        // Send analysis update
        let analysis_update = AnalysisUpdate {
            id: "test".into(),
            progress: 0.5,
            status: "running".into(),
            metrics: None,
            timestamp: chrono::Utc::now(),
        };
        dashboard.update_analysis(analysis_update.clone()).await.unwrap();

        // Both subscribers should receive the update
        let update1 = rx1.recv().await.unwrap();
        let update2 = rx2.recv().await.unwrap();

        assert_eq!(update1.analysis_updates.len(), 1);
        assert_eq!(update2.analysis_updates.len(), 1);
        assert_eq!(update1.analysis_updates[0].id, "test");
        assert_eq!(update2.analysis_updates[0].id, "test");
    }

    #[tokio::test]
    async fn test_visualization_streaming() {
        let dashboard = Dashboard::new(10);
        let changes = (0..100).map(|i| crate::Change {
            timestamp: chrono::Utc::now(),
            author: format!("author{}", i),
            commit_id: format!("commit{}", i),
            message: format!("message{}", i),
            diff: format!("diff{}", i),
            categories: vec![crate::Category::Feature],
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

        let html = dashboard.create_visualization(analysis).await.unwrap();
        assert!(html.contains("updateVisualization"));
    }
} 