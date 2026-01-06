use axum::{
    Router,
    routing::{get, post, delete},
    extract::{State, Query, Path},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{
    domain::{
        models::{Vocabulary, VocabularyId, UserId},
        ports::VocabularyRepository,
    },
    interface::{api::auth::AuthenticatedUser, state::AppState},
};
use chrono::Utc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateVocabularyRequest {
    pub word: String,
    pub definition: String,
    pub context_sentence: Option<String>,
}

#[derive(Deserialize)]
pub struct ListVocabularyRequest {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vocabulary", post(save_vocabulary).get(list_vocabulary))
        .route("/api/vocabulary/:id", delete(delete_vocabulary))
}

async fn save_vocabulary(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVocabularyRequest>,
) -> impl IntoResponse {
    let user_id = UserId(auth.id);
    // Check if word already exists for this user
    match state.repo.find_by_word(&user_id, &payload.word).await {
        Ok(Some(existing)) => {
            let existing_id = existing.id.clone();
            let updated_vocab = Vocabulary {
                id: existing.id,
                user_id: user_id,
                word: existing.word,
                definition: payload.definition,
                context_sentence: payload.context_sentence,
                status: existing.status,
                created_at: existing.created_at,
                updated_at: Utc::now(),
            };
            match state.repo.save(updated_vocab).await {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": existing_id.0 }))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
            }
        },
        Ok(None) => {
            let vocab = Vocabulary {
                id: VocabularyId(Uuid::new_v4()),
                user_id: user_id,
                word: payload.word,
                definition: payload.definition,
                context_sentence: payload.context_sentence,
                status: "New".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            match state.repo.save(vocab).await {
                 Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id.0 }))),
                 Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
            }
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

async fn list_vocabulary(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<ListVocabularyRequest>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match state.repo.list(&UserId(auth.id), limit, offset).await {
        Ok(list) => (StatusCode::OK, Json(serde_json::to_value(list).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

async fn delete_vocabulary(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let vid = VocabularyId(id);
    match state.repo.delete(&vid).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "deleted" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}
