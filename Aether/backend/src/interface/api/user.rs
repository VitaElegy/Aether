use axum::{
    Router,
    routing::get,
    extract::{State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::permission_service::SYSTEM_ROOT_ID;
use crate::domain::ports::UserRepository;
use serde::{Deserialize, Serialize};

use axum::extract::Path;


#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
}

#[derive(Serialize)]
pub struct UserSummary {
    id: uuid::Uuid,
    username: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
}

pub async fn search_users_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    tracing::info!("Searching users: q='{}', limit={}, offset={}", query, limit, offset);

    match state.repo.search_users(&query, limit, offset).await {
        Ok(users) => {
            let dtos: Vec<UserSummary> = users.into_iter().map(|u| UserSummary {
                id: u.id.0,
                username: u.username,
                display_name: u.display_name,
                avatar_url: u.avatar_url,
            }).collect();
            (StatusCode::OK, Json(dtos)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(target_user_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    // 1. Check System Permission: Can this user manage users?
    // Check (SYSTEM_ROOT_ID, "user_manager", requester_id)
    match state.permission_service.check_permission(auth_user.id, SYSTEM_ROOT_ID, "manage_users").await {
        Ok(allowed) => {
            if !allowed {
                 // Temporary: Allows "break-glass" if admin has it? 
                 // Actually break glass grants "owner" on specific entity.
                 // So if Admin B did break-glass on SYSTEM_ROOT_ID with 'owner', they have 'manage_users' (if mapped).
                 return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Insufficient system permissions" }))).into_response();
            }
        },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }

    // 2. Perform Delete
    match state.repo.delete(&crate::domain::models::UserId(target_user_id)).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "User deleted" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}


pub fn router() -> Router<AppState> {
    use axum::routing::delete;
    Router::new()
        .route("/api/users/search", get(search_users_handler))
        .route("/api/users/:id", delete(delete_user_handler))
}
