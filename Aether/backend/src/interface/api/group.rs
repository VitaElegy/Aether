use axum::{
    Json, extract::{State, Path}, response::IntoResponse, http::StatusCode,
};
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateGroupRequest {
    name: String,
}

pub async fn create_group_handler(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    match state.permission_service.create_team(payload.name, auth_user.id).await {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id, "message": "Group created" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    user_id: Uuid,
}

pub async fn add_member_handler(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser, // In future, check if auth_user is owner of group
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> impl IntoResponse {
    // TODO: Verify auth_user is owner of group_id using checks
    match state.permission_service.add_team_member(group_id, payload.user_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Member added" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn remove_member_handler(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser, // In future, check if auth_user can remove
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    // TODO: Check permissions
    match state.permission_service.remove_team_member(group_id, user_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Member removed" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::{post, delete};
    axum::Router::new()
        .route("/api/groups", post(create_group_handler))
        .route("/api/groups/:id/members", post(add_member_handler))
        .route("/api/groups/:id/members/:uid", delete(remove_member_handler))
}
