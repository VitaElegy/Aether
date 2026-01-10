use axum::{
    Json, extract::{State, Query}, response::IntoResponse, http::StatusCode,
};
use crate::interface::state::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::models::UserId;
use axum::extract::FromRef;

#[derive(Deserialize)]
pub struct CheckPermissionParams {
    user_id: Uuid,
    entity_id: Uuid,
    action: String,
}

pub async fn check_permission_handler(
    State(state): State<AppState>,
    Query(params): Query<CheckPermissionParams>,
) -> impl IntoResponse {
    // Note: This endpoint is OPEN (no auth) to allow internal/debug checks easily.
    // In production, you might want to protect it or restrict it.
    
    // We use service directly.
    // Note: PermissionService is generic over R, but AppState has specific type.
    // Ideally we should use the field directly.
    let service = &state.permission_service;
    
    match service.check_permission(params.user_id, params.entity_id, &params.action).await {
        Ok(allowed) => (StatusCode::OK, Json(serde_json::json!({ "allowed": allowed }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct TupleRequest {
    entity_type: String,
    entity_id: Uuid,
    relation: String,
    subject_type: String,
    subject_id: Uuid,
}

pub async fn add_tuple_handler(
    State(state): State<AppState>,
    Json(payload): Json<TupleRequest>,
) -> impl IntoResponse {
    use crate::domain::ports::PermissionRepository; // Import trait to use add_relation
    
    let repo = &state.repo; // PostgresRepository implements PermissionRepository
    
    match repo.add_relation(
        payload.entity_id, &payload.entity_type, &payload.relation, 
        payload.subject_id, &payload.subject_type
    ).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Tuple added" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/permissions/check", get(check_permission_handler))
        .route("/api/permissions/tuple", post(add_tuple_handler))
}
