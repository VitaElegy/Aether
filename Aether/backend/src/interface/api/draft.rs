use axum::{
    routing::{get, put, delete},
    Router,
    extract::{State, Json},
    response::{IntoResponse},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::models::{UserDraft, UserId};
use crate::domain::ports::DraftRepository;

#[derive(Deserialize)]
pub struct DraftUpdateDTO {
    pub target_article_id: Option<Uuid>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub knowledge_base_id: Option<Uuid>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/draft", get(get_draft_handler).put(save_draft_handler).delete(delete_draft_handler))
}

async fn get_draft_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.get_draft(&UserId(user.id)).await {
        Ok(Some(draft)) => (StatusCode::OK, Json(draft)).into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn save_draft_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<DraftUpdateDTO>,
) -> impl IntoResponse {
    let draft = UserDraft {
        user_id: UserId(user.id),
        target_article_id: payload.target_article_id,
        title: payload.title,
        body: payload.body,
        tags: payload.tags,
        category: payload.category,
        knowledge_base_id: payload.knowledge_base_id,
        updated_at: chrono::Utc::now(), // Placeholder, repo sets DB time
    };

    match state.repo.save_draft(draft).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn delete_draft_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.delete_draft(&UserId(user.id)).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
