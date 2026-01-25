use axum::{
    extract::{Path, State},
    routing::{get, delete},
    Json, Router,
};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::domain::models::VrkbMember;
use crate::domain::ports::VrkbRepository;
use axum::http::StatusCode;
use chrono::Utc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/members", get(list_members).post(add_member))
        .route("/api/vrkb/projects/:id/members/:uid", delete(remove_member).put(update_member))
}

#[derive(serde::Deserialize)]
struct AddMemberRequest {
    user_id: Uuid,
    role: String,
}

#[derive(serde::Deserialize)]
struct UpdateMemberRequest {
    role: String,
}

async fn list_members(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbMember>>, (StatusCode, String)> {
    let members = state.repo.list_members(&project_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(members))
}

async fn add_member(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let member = VrkbMember {
        project_id,
        user_id: payload.user_id,
        role: payload.role,
        joined_at: Utc::now(),
        user: None,
    };
    state.repo.add_member(member).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::CREATED)
}

async fn remove_member(
    State(state): State<AppState>,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.remove_member(&project_id, &user_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn update_member(
    State(state): State<AppState>,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMemberRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.update_member_role(&project_id, &user_id, payload.role).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}
