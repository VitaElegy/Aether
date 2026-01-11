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
    q: String,
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
    if params.q.len() < 2 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Query too short" }))).into_response();
    }

    match state.repo.search_users(&params.q).await {
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
