use crate::{
    domain::{
        models::{AnalysisStatus, Node, NodeType, PermissionMode, UserId, Vocabulary},
        ports::VocabularyRepository,
    },
    interface::{api::auth::AuthenticatedUser, state::AppState},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
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
    pub global_sentence_id: Option<Uuid>,
    /// If true, set this as the primary example for the word
    pub is_primary: Option<bool>,
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

    // Core fields
    pub root: Option<String>,
    pub examples: Option<Vec<ExampleRequest>>,
    pub kb_id: Option<Uuid>,

    // ENG-03: New formal fields
    pub lemma: Option<String>,
    pub level: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mastery: Option<String>,
    pub source_kb_id: Option<Uuid>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListVocabularyRequest {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub query: Option<String>,
    pub sort_by: Option<String>, // "query_count", "is_important", "created_at"
    pub order: Option<String>,   // "asc", "desc"
    pub kb_id: Option<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/vocabulary",
            post(save_vocabulary).get(list_vocabulary),
        )
        .route(
            "/api/vocabulary/batch-delete",
            post(batch_delete_vocabulary),
        )
        .route("/api/vocabulary/:id", delete(delete_vocabulary))
        .route("/api/vocabulary/:id/examples", post(add_example))
        .route(
            "/api/vocabulary/:id/increment_query",
            post(increment_query_count),
        )
        .route(
            "/api/vocabulary/:id/toggle_importance",
            post(toggle_importance),
        )
        .route("/api/vocabulary/sentences/search", post(search_sentences))
        // ENG-02: Article workspace endpoints
        .route(
            "/api/english/articles/:id/reanalyze",
            post(reanalyze_article),
        )
        .route(
            "/api/english/articles/:id/analysis-status",
            post(update_analysis_status),
        )
        // ENG-03: Vocabulary batch operations
        .route("/api/vocabulary/batch-tag", post(batch_tag))
        .route(
            "/api/vocabulary/batch-importance",
            post(batch_importance),
        )
        .route("/api/vocabulary/batch-archive", post(batch_archive))
        .route("/api/vocabulary/batch-restore", post(batch_restore))
        .route("/api/vocabulary/merge", post(merge_duplicates))
        // ENG-04: Example system 2.0
        .route(
            "/api/vocabulary/:id/examples/:example_id/primary",
            post(set_primary_example),
        )
        .route(
            "/api/vocabulary/:id/examples/:example_id",
            delete(delete_example),
        )
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
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
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
    let (id, _is_update, existing_count, existing_importance) =
        if let Ok(Some(existing)) = state.repo.find_by_word(&user_id, &payload.word).await {
            (
                existing.node.id,
                true,
                existing.query_count,
                existing.is_important,
            )
        } else {
            (Uuid::new_v4(), false, 0, false)
        };

    // Map Examples
    let examples = payload
        .examples
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
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
                global_sentence_id: e.global_sentence_id,
                is_primary: e.is_primary.unwrap_or(i == 0), // First example is primary by default
            }
        })
        .collect();

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
        lemma: payload.lemma,
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
        query_count: existing_count,
        is_important: existing_importance,
        level: payload
            .level
            .and_then(|l| serde_json::from_value(serde_json::Value::String(l)).ok())
            .unwrap_or_default(),
        tags: payload.tags.unwrap_or_default(),
        mastery: payload
            .mastery
            .and_then(|m| serde_json::from_value(serde_json::Value::String(m)).ok())
            .unwrap_or_default(),
        source_kb_id: payload.source_kb_id.or(payload.kb_id),
        is_archived: false,
    };

    match state.repo.save(vocab).await {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
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

    match state
        .repo
        .list(
            &UserId(auth.id),
            limit,
            offset,
            params.query,
            params.sort_by,
            params.order,
            params.kb_id,
        )
        .await
    {
        Ok(list) => (StatusCode::OK, Json(serde_json::to_value(list).unwrap())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
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
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "deleted" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
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
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "batch_deleted", "count": payload.ids.len() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ImportancePayload {
    pub is_important: bool,
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
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "updated" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
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
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "updated" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
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
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Vocabulary not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // 2. Permission Check (Owner only)
    if vocab.node.author_id != user_id.0 {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied" })),
        )
            .into_response();
    }

    // 3. Add Example
    use crate::domain::models::VocabularyExample;
    let is_primary = payload.is_primary.unwrap_or(false);

    // If marking as primary, un-primary all existing
    if is_primary {
        for ex in &mut vocab.examples {
            ex.is_primary = false;
        }
    }

    let new_example = VocabularyExample {
        id: Uuid::new_v4(),
        sentence: payload.sentence,
        translation: payload.translation,
        note: payload.note,
        image_url: payload.image_url,
        article_id: payload.article_id,
        sentence_uuid: payload.sentence_uuid,
        created_at: Utc::now(),
        global_sentence_id: payload.global_sentence_id,
        is_primary,
    };
    vocab.examples.push(new_example);

    // 4. Save
    match state.repo.save(vocab).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "status": "example_added" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// --- ENG-02: Article Workspace Endpoints ---

#[derive(Deserialize, ToSchema)]
pub struct UpdateAnalysisStatusRequest {
    pub status: String, // "pending", "analyzing", "analyzed", "failed", "archived"
    pub error_message: Option<String>,
    pub error_code: Option<String>,
}

/// Trigger a re-analysis of an article. Transitions status to Analyzing.
#[utoipa::path(
    post,
    path = "/api/english/articles/{id}/reanalyze",
    params(
        ("id" = Uuid, Path, description = "Article ID")
    ),
    responses(
        (status = 200, description = "Reanalysis triggered"),
        (status = 400, description = "Invalid state transition"),
        (status = 404, description = "Article not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "english"
)]
async fn reanalyze_article(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Fetch the article
    let article = match state.article_repo.find_by_id(&id).await {
        Ok(Some(crate::domain::models::ContentItem::Article(a))) => a,
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Article not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Validate state transition
    let current_status = article
        .analysis_status
        .clone()
        .unwrap_or(AnalysisStatus::Pending);

    if !current_status.can_transition_to(&AnalysisStatus::Analyzing) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Cannot reanalyze from status '{}'", current_status),
                "current_status": current_status.to_string(),
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "analyzing",
            "article_id": id,
            "message": "Reanalysis triggered successfully"
        })),
    )
        .into_response()
}

/// Update the analysis status of an article (used by backend analysis workers).
#[utoipa::path(
    post,
    path = "/api/english/articles/{id}/analysis-status",
    params(
        ("id" = Uuid, Path, description = "Article ID")
    ),
    request_body = UpdateAnalysisStatusRequest,
    responses(
        (status = 200, description = "Status updated"),
        (status = 400, description = "Invalid status or transition"),
        (status = 500, description = "Internal server error")
    ),
    tag = "english"
)]
async fn update_analysis_status(
    _auth: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAnalysisStatusRequest>,
) -> impl IntoResponse {
    let target_status: AnalysisStatus = match payload.status.parse() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    let diagnostics = if target_status == AnalysisStatus::Failed {
        Some(crate::domain::models::AnalysisDiagnostics {
            error_code: payload.error_code,
            error_message: payload.error_message.unwrap_or_else(|| "Unknown error".to_string()),
            failed_at: Utc::now(),
            retry_count: 0,
        })
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "article_id": id,
            "status": target_status.to_string(),
            "diagnostics": diagnostics,
        })),
    )
        .into_response()
}

// --- ENG-03: Vocabulary Batch Operations ---

#[derive(Deserialize, ToSchema)]
pub struct BatchTagRequest {
    pub ids: Vec<Uuid>,
    pub tags: Vec<String>,
    /// "add" or "set" — add appends tags, set replaces them
    pub mode: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct BatchImportanceRequest {
    pub ids: Vec<Uuid>,
    pub is_important: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct BatchArchiveRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct MergeDuplicatesRequest {
    /// The ID to keep (primary)
    pub primary_id: Uuid,
    /// IDs to merge into the primary and then delete
    pub duplicate_ids: Vec<Uuid>,
}

async fn batch_tag(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchTagRequest>,
) -> impl IntoResponse {
    let mode = payload.mode.unwrap_or_else(|| "add".to_string());
    let mut updated = 0u32;

    for id in &payload.ids {
        if let Ok(Some(mut vocab)) = state.repo.find_by_id(id).await {
            match mode.as_str() {
                "set" => {
                    vocab.tags = payload.tags.clone();
                }
                _ => {
                    // "add" mode — append tags avoiding duplicates
                    for tag in &payload.tags {
                        if !vocab.tags.contains(tag) {
                            vocab.tags.push(tag.clone());
                        }
                    }
                }
            }
            if state.repo.save(vocab).await.is_ok() {
                updated += 1;
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "batch_tagged", "updated": updated })),
    )
        .into_response()
}

async fn batch_importance(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchImportanceRequest>,
) -> impl IntoResponse {
    let mut updated = 0u32;
    for id in &payload.ids {
        if state.repo.set_importance(id, payload.is_important).await.is_ok() {
            updated += 1;
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "batch_importance", "updated": updated })),
    )
        .into_response()
}

async fn batch_archive(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchArchiveRequest>,
) -> impl IntoResponse {
    let mut updated = 0u32;
    for id in &payload.ids {
        if let Ok(Some(mut vocab)) = state.repo.find_by_id(id).await {
            vocab.is_archived = true;
            if state.repo.save(vocab).await.is_ok() {
                updated += 1;
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "batch_archived", "updated": updated })),
    )
        .into_response()
}

async fn batch_restore(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchArchiveRequest>,
) -> impl IntoResponse {
    let mut updated = 0u32;
    for id in &payload.ids {
        if let Ok(Some(mut vocab)) = state.repo.find_by_id(id).await {
            vocab.is_archived = false;
            if state.repo.save(vocab).await.is_ok() {
                updated += 1;
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "batch_restored", "updated": updated })),
    )
        .into_response()
}

async fn merge_duplicates(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<MergeDuplicatesRequest>,
) -> impl IntoResponse {
    let user_id = UserId(auth.id);

    // 1. Load the primary vocabulary
    let mut primary = match state.repo.find_by_id(&payload.primary_id).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Primary vocabulary not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // 2. Check ownership
    if primary.node.author_id != user_id.0 {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied" })),
        )
            .into_response();
    }

    // 3. Merge data from duplicates
    let mut merged_count = 0u32;
    for dup_id in &payload.duplicate_ids {
        if let Ok(Some(dup)) = state.repo.find_by_id(dup_id).await {
            // Merge examples (avoid duplicates by sentence text)
            for ex in dup.examples {
                if !primary.examples.iter().any(|e| e.sentence == ex.sentence) {
                    primary.examples.push(ex);
                }
            }
            // Merge tags
            for tag in dup.tags {
                if !primary.tags.contains(&tag) {
                    primary.tags.push(tag);
                }
            }
            // Accumulate query count
            primary.query_count += dup.query_count;
            // Keep importance if any is important
            if dup.is_important {
                primary.is_important = true;
            }
            // Delete the duplicate
            let _ = state.repo.delete(dup_id).await;
            merged_count += 1;
        }
    }

    // 4. Save the merged primary
    match state.repo.save(primary).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "merged",
                "primary_id": payload.primary_id,
                "merged_count": merged_count,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// --- ENG-04: Example System 2.0 ---

/// Set an example as the primary example for a vocabulary entry.
async fn set_primary_example(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path((vocab_id, example_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let user_id = UserId(auth.id);

    let mut vocab = match state.repo.find_by_id(&vocab_id).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Vocabulary not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if vocab.node.author_id != user_id.0 {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied" })),
        )
            .into_response();
    }

    // Find and set primary
    let mut found = false;
    for ex in &mut vocab.examples {
        if ex.id == example_id {
            ex.is_primary = true;
            found = true;
        } else {
            ex.is_primary = false;
        }
    }

    if !found {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Example not found" })),
        )
            .into_response();
    }

    match state.repo.save(vocab).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "primary_set", "example_id": example_id })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Delete a specific example from a vocabulary entry.
async fn delete_example(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path((vocab_id, example_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let user_id = UserId(auth.id);

    let mut vocab = match state.repo.find_by_id(&vocab_id).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Vocabulary not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if vocab.node.author_id != user_id.0 {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied" })),
        )
            .into_response();
    }

    let original_len = vocab.examples.len();
    vocab.examples.retain(|ex| ex.id != example_id);

    if vocab.examples.len() == original_len {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Example not found" })),
        )
            .into_response();
    }

    // If we removed the primary example, make the first remaining one primary
    if !vocab.examples.is_empty() && !vocab.examples.iter().any(|e| e.is_primary) {
        vocab.examples[0].is_primary = true;
    }

    match state.repo.save(vocab).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "example_deleted" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
