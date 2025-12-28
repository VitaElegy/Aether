use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{KnowledgeBase, KnowledgeBaseId, UserId, Visibility};
use crate::domain::ports::KnowledgeBaseRepository;
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};

#[derive(serde::Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub cover_image: Option<String>,
    pub visibility: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateKnowledgeBaseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub cover_image: Option<String>,
    pub visibility: Option<String>,
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
    // 1. Check if title exists
    if let Ok(Some(_)) = repo.find_by_title(&UserId(user.id), &payload.title).await {
         return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Knowledge Base with this title already exists" }))).into_response();
    }

    let id = KnowledgeBaseId(Uuid::new_v4());

    let kb = KnowledgeBase {
        id: id.clone(),
        author_id: user.id,
        title: payload.title,
        description: payload.description,
        tags: payload.tags.unwrap_or_default(),
        cover_image: payload.cover_image,
        visibility: match payload.visibility.as_deref() {
            Some("Public") => Visibility::Public,
            Some("Internal") => Visibility::Internal,
            _ => Visibility::Private,
        },
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
        // Check uniqueness if title is changing
        if t != existing.title {
             if let Ok(Some(_)) = repo.find_by_title(&UserId(user.id), &t).await {
                return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Knowledge Base with this title already exists" }))).into_response();
            }
        }
        existing.title = t;
    }
    if let Some(d) = payload.description {
        existing.description = Some(d);
    }
    if let Some(tags) = payload.tags {
        existing.tags = tags;
    }
    if let Some(cover) = payload.cover_image {
        existing.cover_image = Some(cover);
    }
    if let Some(vis) = payload.visibility {
        existing.visibility = match vis.as_str() {
             "Public" => Visibility::Public,
             "Internal" => Visibility::Internal,
             "Private" => Visibility::Private,
             _ => existing.visibility,
        };
    }
    existing.updated_at = Utc::now();

    // 4. Save
    match repo.save(existing).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// TODO: Import/Export handlers (Need generic Service or specific implementation)
