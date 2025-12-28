use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{KnowledgeBase, KnowledgeBaseId, UserId};
use crate::domain::ports::KnowledgeBaseRepository;
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};

#[derive(serde::Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateKnowledgeBaseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
}

pub async fn list_knowledge_bases_handler(
    State(repo): State<Arc<dyn KnowledgeBaseRepository>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    // List KBs owned by the logged-in user
    match repo.list(UserId(user.id)).await {
        Ok(kbs) => (StatusCode::OK, Json(kbs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_knowledge_base_handler(
    State(repo): State<Arc<dyn KnowledgeBaseRepository>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateKnowledgeBaseRequest>,
) -> impl IntoResponse {
    let id = KnowledgeBaseId(Uuid::new_v4());

    let kb = KnowledgeBase {
        id: id.clone(),
        author_id: user.id,
        title: payload.title,
        description: payload.description,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match repo.save(kb).await {
        Ok(new_id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": new_id.0 }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_knowledge_base_handler(
    State(repo): State<Arc<dyn KnowledgeBaseRepository>>,
    _user: MaybeAuthenticatedUser, // In future, check for private KBs
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match repo.find_by_id(&KnowledgeBaseId(id)).await {
        Ok(Some(kb)) => (StatusCode::OK, Json(kb)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Knowledge Base not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_knowledge_base_handler(
    State(repo): State<Arc<dyn KnowledgeBaseRepository>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Verify ownership
    let existing = match repo.find_by_id(&KnowledgeBaseId(id)).await {
        Ok(Some(kb)) => kb,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Knowledge Base not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if existing.author_id != user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    match repo.delete(&KnowledgeBaseId(id)).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_knowledge_base_handler(
    State(repo): State<Arc<dyn KnowledgeBaseRepository>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateKnowledgeBaseRequest>,
) -> impl IntoResponse {
    // 1. Find existing
    let mut existing = match repo.find_by_id(&KnowledgeBaseId(id)).await {
        Ok(Some(kb)) => kb,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Knowledge Base not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    // 2. Check ownership
    if existing.author_id != user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 3. Update fields
    if let Some(t) = payload.title {
        existing.title = t;
    }
    if let Some(d) = payload.description {
        existing.description = Some(d);
    }
    existing.updated_at = Utc::now();

    // 4. Save
    match repo.save(existing).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// TODO: Import/Export handlers (Need generic Service or specific implementation)
