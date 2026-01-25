use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::domain::models::VrkbStats;
use crate::domain::ports::VrkbRepository;
use axum::http::StatusCode;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/stats", get(get_project_stats))
}

async fn get_project_stats(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<VrkbStats>, (StatusCode, String)> {
    let stats = state.repo.get_project_stats(&project_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(stats))
}
