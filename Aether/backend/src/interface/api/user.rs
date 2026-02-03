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
use crate::domain::ports::UserRepository;
use serde::{Deserialize, Serialize};

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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/users/search", get(search_users_handler))
}
