use axum::{
    extract::{Path, State, Query},
    Json, Router, routing::get,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::domain::ports::{UserRepository, PermissionRepository}; // Assuming UserRepo is available in ports

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
}

async fn search_users(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    // ... implementation ...
    let users = state.repo.search_users(&params.q).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = users.into_iter().map(|u| UserResponse {
        id: u.id.0,
        username: u.username,
        display_name: u.display_name,
        avatar_url: u.avatar_url,
    }).collect();

    Ok(Json(response))
}
