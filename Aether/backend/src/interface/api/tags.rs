use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::interface::{api::auth::AuthenticatedUser, state::AppState};
use crate::domain::ports::TagRepository; // Added Import

pub async fn list_tags_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.get_all_tags().await {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::get;
    axum::Router::new()
         .route("/api/tags", get(list_tags_handler))
}
