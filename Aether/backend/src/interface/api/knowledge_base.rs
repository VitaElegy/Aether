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
use crate::domain::ports::{PermissionRepository, UserRepository};
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub cover_image: Option<String>,
    pub cover_offset_y: Option<i32>,
    pub renderer_id: Option<String>,
    pub visibility: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateKnowledgeBaseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub cover_image: Option<String>,
    pub cover_offset_y: Option<i32>,
    pub renderer_id: Option<String>,
    pub visibility: Option<String>,
}

use axum::extract::Query;

#[derive(serde::Deserialize)]
pub struct ListKnowledgeBasesRequest {
    pub author_id: Option<Uuid>,
}

pub async fn list_knowledge_bases_handler(
    State(repo): State<Arc<dyn KnowledgeBaseRepository>>,
    user: MaybeAuthenticatedUser,
    Query(params): Query<ListKnowledgeBasesRequest>,
) -> impl IntoResponse {
    let viewer_id = user.0.map(|u| UserId(u.id));
    let author_id = params.author_id.map(UserId);

    // If no author_id specified, assume listing own KBs (backward compatibility behavior)
    // But for guests, listing "own" KBs is impossible.
    // So:
    // If author_id is Some -> List that author's KBs (filtered).
    // If author_id is None ->
    //    If Logged In -> List My KBs (author_id = Me).
    //    If Guest -> Return Empty or Public Feed?
    //    Let's default to "List My KBs" if logged in.
    //    If Guest and no author_id, return Empty List (or BadRequest?)

    let target_author_id = if author_id.is_some() {
        author_id
    } else {
        viewer_id.clone()
    };

    if target_author_id.is_none() {
        // Guest querying nothing?
        return (StatusCode::OK, Json(Vec::<KnowledgeBase>::new())).into_response();
    }

    match repo.list(viewer_id, target_author_id).await {
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
        cover_offset_y: payload.cover_offset_y.unwrap_or(50),
        renderer_id: payload.renderer_id,
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
    if let Some(offset) = payload.cover_offset_y {
        existing.cover_offset_y = offset.clamp(0, 100);
    }
    if let Some(renderer) = payload.renderer_id {
        existing.renderer_id = Some(renderer);
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

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::{get, post, delete};
    axum::Router::new()
        .route("/api/knowledge-bases", post(create_knowledge_base_handler).get(list_knowledge_bases_handler))
        .route("/api/knowledge-bases/:id", get(get_knowledge_base_handler).put(update_knowledge_base_handler).delete(delete_knowledge_base_handler))
        .route("/api/knowledge-bases/:id/collaborators", get(list_collaborators_handler).post(add_collaborator_handler))
        .route("/api/knowledge-bases/:id/collaborators/:uid", delete(remove_collaborator_handler))
}

// --- Collaborators ---
#[derive(Deserialize)]
pub struct AddCollaboratorRequest {
    pub user_id: Uuid,
    pub role: String, // "viewer", "editor", "owner"
}

pub async fn add_collaborator_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddCollaboratorRequest>,
) -> impl IntoResponse {
    // 1. Check if requester is Owner (or has delete permission on KB)
    // Note: 'delete' action typically implies full control/ownership
    let is_owner = match state.permission_service.check_permission(user.id, id, "delete").await {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Permission check failed").into_response(),
    };
    if !is_owner {
        return (StatusCode::FORBIDDEN, "Only owners can manage collaborators").into_response();
    }

    // 2. Validate Role
    let role = payload.role.to_lowercase();
    if !["viewer", "editor", "owner"].contains(&role.as_str()) {
        return (StatusCode::BAD_REQUEST, "Invalid role").into_response();
    }

    // 3. Add Tuple (KB, role, User)
    match state.repo.add_relation(id, "knowledge_base", &role, payload.user_id, "user").await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "added"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn remove_collaborator_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path((id, target_user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    // 1. Check if requester is Owner
    let is_owner = match state.permission_service.check_permission(user.id, id, "delete").await {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Permission check failed").into_response(),
    };
    if !is_owner {
        return (StatusCode::FORBIDDEN, "Only owners can manage collaborators").into_response();
    }

    // 2. Remove all roles (iterate simplistically)
    let roles = vec!["viewer", "editor", "owner"];
    for role in roles {
        let _ = state.repo.remove_relation(id, "knowledge_base", role, target_user_id, "user").await;
    }
    
    (StatusCode::OK, Json(serde_json::json!({"status": "removed"}))).into_response()
}

#[derive(Serialize)]
pub struct CollaboratorDto {
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub role: String,
}

pub async fn list_collaborators_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
   // 1. Check if requester can view KB
   let can_view = match state.permission_service.check_permission(user.id, id, "read").await {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Permission check failed").into_response(),
   };
   if !can_view {
       return (StatusCode::NOT_FOUND, "KB not found or access denied").into_response();
   }

   // 2. Fetch collaborators for each role
   let mut collaborators = Vec::new();
   let roles = vec!["owner", "editor", "viewer"];
   
   for role in roles {
       if let Ok(user_ids) = state.repo.get_collaborators(id, "knowledge_base", role).await {
           for uid in user_ids {
               if let Ok(Some(u)) = UserRepository::find_by_id(state.repo.as_ref(), &crate::domain::models::UserId(uid)).await {
                   collaborators.push(CollaboratorDto {
                       user_id: u.id.0,
                       username: u.username,
                       avatar_url: u.avatar_url,
                       role: role.to_string(),
                   });
               }
           }
       }
   }
   
   (StatusCode::OK, Json(collaborators)).into_response()
}
