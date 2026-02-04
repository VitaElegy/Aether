use axum::{
    Json, extract::{State, Query, Path}, response::IntoResponse, http::StatusCode,
};
use crate::interface::state::AppState;
use serde::Deserialize;
use uuid::Uuid;
use crate::interface::api::auth::AuthenticatedUser;


#[derive(Deserialize)]
pub struct CheckPermissionParams {
    user_id: Uuid,
    entity_id: Uuid,
    action: String,
}

// Open endpoint for checking (Internal use mainly)
pub async fn check_permission_handler(
    State(state): State<AppState>,
    Query(params): Query<CheckPermissionParams>,
) -> impl IntoResponse {
    let service = &state.permission_service;
    match service.check_permission(params.user_id, params.entity_id, &params.action).await {
        Ok(allowed) => (StatusCode::OK, Json(serde_json::json!({ "allowed": allowed }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// --- Management API ---

#[derive(Deserialize)]
pub struct GrantRequest {
    user_id: Uuid,
    entity_id: Uuid,
    relation: String,
}

pub async fn list_user_permissions_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    // Auth guard needed? For now open to Authenticated users (via router layer usually)
    // but here we just implement logic
) -> impl IntoResponse {
    match state.permission_service.get_user_explicit_permissions(user_id).await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn grant_permission_handler(
    State(state): State<AppState>,
    Json(payload): Json<GrantRequest>,
) -> impl IntoResponse {
    match state.permission_service.grant_permission(payload.user_id, payload.entity_id, &payload.relation).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Permission granted" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn revoke_permission_handler(
    State(state): State<AppState>,
    Json(payload): Json<GrantRequest>,
) -> impl IntoResponse {
    match state.permission_service.revoke_permission(payload.user_id, payload.entity_id, &payload.relation).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Permission revoked" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct BreakGlassRequest {
    entity_id: Uuid,
    relation: String,
}

pub async fn break_glass_handler(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser, // Correct extractor
    Json(payload): Json<BreakGlassRequest>,
) -> impl IntoResponse {
    match state.permission_service.break_glass_access(auth_user.id, payload.entity_id, &payload.relation).await {

        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Break-Glass access granted. Audit log recorded." }))).into_response(),
        Err(e) => {
            if e.to_string().contains("Unauthorized") {
                (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
            }
        }
    }
}

// Deprecated direct tuple handler (wrapped by grant/revoke now)
// kept if needed but not exposing in router unless necessary

pub fn router() -> axum::Router<AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/permissions/check", get(check_permission_handler))
        .route("/api/permissions/user/:id", get(list_user_permissions_handler))
        .route("/api/permissions/grant", post(grant_permission_handler))
        .route("/api/permissions/revoke", post(revoke_permission_handler))
        .route("/api/permissions/break-glass", post(break_glass_handler))
}
