use axum::{
    Router,
    routing::{post, delete},
    extract::{State, Query, Path},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::Deserialize;
use crate::{
    domain::{
        models::{Vocabulary, Node, NodeType, PermissionMode, UserId},
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
    pub translation: Option<String>,
    pub phonetic: Option<String>,
    pub context_sentence: Option<String>,
    pub image_url: Option<String>,
    pub language: Option<String>,
}

#[derive(Deserialize)]
pub struct ListVocabularyRequest {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub query: Option<String>,
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
    
    if let Ok(Some(existing)) = state.repo.find_by_word(&user_id, &payload.word).await {
         return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Word already exists", "id": existing.node.id }))).into_response();
    }

    let id = Uuid::new_v4();
    let vocab = Vocabulary {
        node: Node {
            id,
            parent_id: None,
            author_id: user_id.0,
            knowledge_base_id: None,
            r#type: NodeType::Vocabulary,
            title: payload.word.clone(), 
            permission_mode: PermissionMode::Private, 
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        word: payload.word,
        definition: payload.definition,
        translation: payload.translation,
        phonetic: payload.phonetic,
        context_sentence: payload.context_sentence,
        image_url: payload.image_url,
        language: payload.language.unwrap_or("en".to_string()),
        status: "New".to_string(),
    };

    match state.repo.save(vocab).await {
            Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn list_vocabulary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListVocabularyRequest>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match state.repo.list(&UserId(auth.id), limit, offset, params.query).await {
        Ok(list) => (StatusCode::OK, Json(serde_json::to_value(list).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn delete_vocabulary(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.delete(&id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "deleted" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
