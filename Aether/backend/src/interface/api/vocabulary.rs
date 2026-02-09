use axum::{
    Router,
    routing::{post, delete},
    extract::{State, Query, Path},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::Deserialize;
use utoipa::{ToSchema, IntoParams};
use crate::{
    domain::{
        models::{Vocabulary, Node, NodeType, PermissionMode, UserId},
        ports::VocabularyRepository,
    },
    interface::{api::auth::AuthenticatedUser, state::AppState},
};
use chrono::Utc;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct ExampleRequest {
    #[schema(example = "This is an example sentence.")]
    pub sentence: String,
    #[schema(example = "这是一个例句。")]
    pub translation: Option<String>,
    pub note: Option<String>,
    pub image_url: Option<String>,
    pub article_id: Option<Uuid>,
    pub sentence_uuid: Option<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateVocabularyRequest {
    #[schema(example = "apple")]
    pub word: String,
    #[schema(example = "A round fruit with red or green skin.")]
    pub definition: String,
    #[schema(example = "苹果")]
    pub translation: Option<String>,
    #[schema(example = "/ˈæp.əl/")]
    pub phonetic: Option<String>,
    
    // Deprecated but kept optional
    pub context_sentence: Option<String>,
    pub image_url: Option<String>,
    
    #[schema(example = "en")]
    pub language: Option<String>,
    
    // New
    pub root: Option<String>,
    pub examples: Option<Vec<ExampleRequest>>,
    pub kb_id: Option<Uuid>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListVocabularyRequest {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub query: Option<String>,
    pub sort_by: Option<String>, // "query_count", "is_important", "created_at"
    pub order: Option<String>, // "asc", "desc"
    pub kb_id: Option<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vocabulary", post(save_vocabulary).get(list_vocabulary))
        .route("/api/vocabulary/batch-delete", post(batch_delete_vocabulary))
        .route("/api/vocabulary/:id", delete(delete_vocabulary))
        .route("/api/vocabulary/:id/examples", post(add_example))
        .route("/api/vocabulary/:id/increment_query", post(increment_query_count))
        .route("/api/vocabulary/:id/toggle_importance", post(toggle_importance))
        .route("/api/vocabulary/sentences/search", post(search_sentences))
}

#[derive(Deserialize, ToSchema)]
pub struct SearchSentencesRequest {
    #[schema(example = "apple")]
    pub query: String,
}

#[utoipa::path(
    post,
    path = "/api/vocabulary/sentences/search",
    request_body = SearchSentencesRequest,
    responses(
        (status = 200, description = "Search results found", body = Vec<serde_json::Value>),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn search_sentences(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<SearchSentencesRequest>,
) -> impl IntoResponse {
    match state.repo.search_global_sentences(&payload.query).await {
        Ok(results) => {
            let mapped: Vec<serde_json::Value> = results.into_iter().map(|(id, text, translation)| {
                serde_json::json!({ "id": id, "text": text, "translation": translation })
            }).collect();
            (StatusCode::OK, Json(mapped)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/vocabulary",
    request_body = CreateVocabularyRequest,
    responses(
        (status = 201, description = "Vocabulary created successfully", body = serde_json::Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn save_vocabulary(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVocabularyRequest>,
) -> impl IntoResponse {
    let user_id = UserId(auth.id);
    
    // Check for existing word to Determine Upsert vs Create
    let (id, _is_update, existing_count, existing_importance) = if let Ok(Some(existing)) = state.repo.find_by_word(&user_id, &payload.word).await {
         (existing.node.id, true, existing.query_count, existing.is_important)
    } else {
         (Uuid::new_v4(), false, 0, false)
    };
    
    // Map Examples
    let examples = payload.examples.unwrap_or_default().into_iter().map(|e| {
        use crate::domain::models::VocabularyExample;
        VocabularyExample {
            id: Uuid::new_v4(),
            sentence: e.sentence,
            translation: e.translation,
            note: e.note,
            image_url: e.image_url,
            article_id: e.article_id,
            sentence_uuid: e.sentence_uuid,
            created_at: Utc::now(),
            global_sentence_id: None,
        }
    }).collect();

    let vocab = Vocabulary {
        node: Node {
            id,
            parent_id: None,
            author_id: user_id.0,
            knowledge_base_id: payload.kb_id,
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
        root: payload.root,
        examples,
        query_count: existing_count, // Preserve or 0
        is_important: existing_importance, // Preserve or false
    };

    match state.repo.save(vocab).await {
            Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/vocabulary",
    params(
        ListVocabularyRequest
    ),
    responses(
        (status = 200, description = "List of vocabulary", body = serde_json::Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn list_vocabulary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListVocabularyRequest>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match state.repo.list(&UserId(auth.id), limit, offset, params.query, params.sort_by, params.order, params.kb_id).await {
        Ok(list) => (StatusCode::OK, Json(serde_json::to_value(list).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/vocabulary/{id}",
    params(
        ("id" = Uuid, Path, description = "Vocabulary ID")
    ),
    responses(
        (status = 200, description = "Vocabulary deleted"),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
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

#[utoipa::path(
    post,
    path = "/api/vocabulary/batch-delete",
    request_body = BatchDeleteRequest,
    responses(
        (status = 200, description = "Batch delete successful"),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn batch_delete_vocabulary(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchDeleteRequest>,
) -> impl IntoResponse {
    match state.repo.delete_many(&payload.ids).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "batch_deleted", "count": payload.ids.len() }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ImportancePayload {
    pub is_important: bool
}

#[utoipa::path(
    post,
    path = "/api/vocabulary/{id}/increment_query",
    params(
        ("id" = Uuid, Path, description = "Vocabulary ID")
    ),
    responses(
        (status = 200, description = "Query count incremented"),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn increment_query_count(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.increment_query_count(&id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "updated" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/vocabulary/{id}/toggle_importance",
    params(
        ("id" = Uuid, Path, description = "Vocabulary ID")
    ),
    request_body = ImportancePayload,
    responses(
        (status = 200, description = "Importance toggled"),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn toggle_importance(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ImportancePayload>,
) -> impl IntoResponse {
    match state.repo.set_importance(&id, payload.is_important).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "updated" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/vocabulary/{id}/examples",
    params(
        ("id" = Uuid, Path, description = "Vocabulary ID")
    ),
    request_body = ExampleRequest,
    responses(
        (status = 201, description = "Example added"),
        (status = 404, description = "Vocabulary not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    tag = "vocabulary"
)]
async fn add_example(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ExampleRequest>,
) -> impl IntoResponse {
    let user_id = UserId(auth.id);
    
    // 1. Fetch Existing
    let mut vocab = match state.repo.find_by_id(&id).await {
        Ok(Some(v)) => v,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Vocabulary not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    // 2. Permission Check (Owner only)
    if vocab.node.author_id != user_id.0 {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 3. Add Example
    use crate::domain::models::VocabularyExample;
    let new_example = VocabularyExample {
        id: Uuid::new_v4(),
        sentence: payload.sentence,
        translation: payload.translation,
        note: payload.note,
        image_url: payload.image_url,
        article_id: payload.article_id,
        sentence_uuid: payload.sentence_uuid,
        created_at: Utc::now(),
        global_sentence_id: None,
    };
    vocab.examples.push(new_example);

    // 4. Save
    match state.repo.save(vocab).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "status": "example_added" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
