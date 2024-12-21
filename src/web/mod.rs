use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use crate::{Analysis, Result, HistorianError, visualization::Visualizer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub active_analyses: Vec<AnalysisStatus>,
    pub team_members: Vec<TeamMember>,
    pub recent_projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatus {
    pub id: String,
    pub repository: String,
    pub progress: f32,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub metrics: Option<crate::Metrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub active_projects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub repository_url: String,
    pub last_analysis: chrono::DateTime<chrono::Utc>,
    pub team_members: Vec<String>,
    pub metrics: Option<crate::Metrics>,
}

pub struct WebServer {
    state: Arc<RwLock<DashboardState>>,
    visualizer: Arc<Visualizer>,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(DashboardState {
                active_analyses: Vec::new(),
                team_members: Vec::new(),
                recent_projects: Vec::new(),
            })),
            visualizer: Arc::new(Visualizer::new(None)),
        }
    }

    pub async fn run(&self, addr: &str) -> Result<()> {
        let app = Router::new()
            .route("/api/dashboard", get(Self::get_dashboard_state))
            .route("/api/analysis/start", post(Self::start_analysis))
            .route("/api/analysis/:id/status", get(Self::get_analysis_status))
            .route("/api/team/members", get(Self::get_team_members))
            .route("/api/team/members", post(Self::add_team_member))
            .route("/api/projects", get(Self::get_projects))
            .route("/api/projects/compare", post(Self::compare_projects))
            .layer(CorsLayer::permissive())
            .with_state(self.state.clone());

        println!("Starting web server on {}", addr);
        axum::Server::bind(&addr.parse()?)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }

    async fn get_dashboard_state(
        State(state): State<Arc<RwLock<DashboardState>>>
    ) -> Json<DashboardState> {
        Json(state.read().await.clone())
    }

    async fn start_analysis(
        State(state): State<Arc<RwLock<DashboardState>>>,
        Json(payload): Json<StartAnalysisRequest>,
    ) -> Result<Json<AnalysisStatus>> {
        let analysis_id = uuid::Uuid::new_v4().to_string();
        let status = AnalysisStatus {
            id: analysis_id.clone(),
            repository: payload.repository,
            progress: 0.0,
            status: "initializing".to_string(),
            started_at: chrono::Utc::now(),
            metrics: None,
        };

        state.write().await.active_analyses.push(status.clone());
        Ok(Json(status))
    }

    async fn get_analysis_status(
        State(state): State<Arc<RwLock<DashboardState>>>,
        axum::extract::Path(id): axum::extract::Path<String>,
    ) -> Result<Json<AnalysisStatus>> {
        let state = state.read().await;
        let status = state.active_analyses.iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(|| HistorianError::NotFound("Analysis not found".into()))?;
        Ok(Json(status))
    }

    async fn get_team_members(
        State(state): State<Arc<RwLock<DashboardState>>>
    ) -> Json<Vec<TeamMember>> {
        Json(state.read().await.team_members.clone())
    }

    async fn add_team_member(
        State(state): State<Arc<RwLock<DashboardState>>>,
        Json(member): Json<TeamMember>,
    ) -> Result<Json<TeamMember>> {
        state.write().await.team_members.push(member.clone());
        Ok(Json(member))
    }

    async fn get_projects(
        State(state): State<Arc<RwLock<DashboardState>>>
    ) -> Json<Vec<ProjectSummary>> {
        Json(state.read().await.recent_projects.clone())
    }

    async fn compare_projects(
        State(state): State<Arc<RwLock<DashboardState>>>,
        Json(payload): Json<CompareProjectsRequest>,
    ) -> Result<Json<ProjectComparison>> {
        let state = state.read().await;
        let projects: Vec<_> = state.recent_projects.iter()
            .filter(|p| payload.project_ids.contains(&p.id))
            .collect();

        Ok(Json(ProjectComparison {
            projects: projects.iter().map(|p| p.clone()).collect(),
            comparison_date: chrono::Utc::now(),
            metrics_comparison: Vec::new(), // TODO: Implement detailed comparison
        }))
    }
}

#[derive(Debug, Deserialize)]
struct StartAnalysisRequest {
    repository: String,
    branch: Option<String>,
    team_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CompareProjectsRequest {
    project_ids: Vec<String>,
    metrics: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ProjectComparison {
    projects: Vec<ProjectSummary>,
    comparison_date: chrono::DateTime<chrono::Utc>,
    metrics_comparison: Vec<MetricComparison>,
}

#[derive(Debug, Serialize)]
struct MetricComparison {
    metric_name: String,
    values: Vec<(String, f64)>,
    trend: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    use hyper::Request;

    #[tokio::test]
    async fn test_dashboard_api() {
        let server = WebServer::new();
        let app = Router::new()
            .route("/api/dashboard", get(WebServer::get_dashboard_state))
            .with_state(server.state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/dashboard")
                    .body(hyper::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_analysis_workflow() {
        let server = WebServer::new();
        let app = Router::new()
            .route("/api/analysis/start", post(WebServer::start_analysis))
            .route("/api/analysis/:id/status", get(WebServer::get_analysis_status))
            .with_state(server.state.clone());

        // Start analysis
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/analysis/start")
                    .header("content-type", "application/json")
                    .body(hyper::Body::from(
                        serde_json::to_string(&StartAnalysisRequest {
                            repository: "test/repo".to_string(),
                            branch: None,
                            team_members: vec![],
                        })
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let status: AnalysisStatus = serde_json::from_slice(&body_bytes).unwrap();

        // Get status
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/analysis/{}/status", status.id))
                    .body(hyper::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
} 