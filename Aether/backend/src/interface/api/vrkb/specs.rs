use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::domain::models::VrkbSpec;
use crate::domain::ports::VrkbRepository;
use axum::http::StatusCode;
use chrono::Utc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/specs", get(get_specs).put(save_spec))
}

#[derive(serde::Deserialize)]
struct SaveSpecRequest {
    id: Uuid, // Explicit ID to handle updates
    title: String,
    content: Option<String>,
    version: i32,
}

async fn get_specs(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbSpec>>, (StatusCode, String)> {
    let specs = state.repo.get_specs(&project_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(specs))
}

async fn save_spec(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<SaveSpecRequest>,
) -> Result<Json<Uuid>, (StatusCode, String)> {
    let spec = VrkbSpec {
        id: payload.id,
        project_id,
        title: payload.title,
        content: payload.content,
        version: payload.version,
        updated_at: Utc::now(),
    };
    let id = state.repo.save_spec(spec).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(id))
}
