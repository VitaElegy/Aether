use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use crate::domain::ports::TagRepository;
use crate::interface::api::auth::AuthenticatedUser;

pub async fn list_tags_handler(
    State(repo): State<Arc<dyn TagRepository>>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match repo.get_all_tags().await {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::get;
    axum::Router::new()
         .route("/api/tags", get(list_tags_handler))
}
