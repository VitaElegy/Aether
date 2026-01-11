use axum::{
    Json, extract::{State, Path, Query}, response::IntoResponse, http::StatusCode,
};
use crate::domain::{
    ports::{ArticleRepository, RepositoryError}, 
    models::{Article, Node, NodeType, PermissionMode, ContentBody, ContentStatus, UserId},
};
use uuid::Uuid; // Added
use chrono::Utc;
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateContentRequest {
    title: String,
    body: String, 
    tags: Vec<String>,
    category: Option<String>,
    visibility: String, 
    status: Option<String>, 
    #[serde(rename = "reason")]
    _reason: Option<String>, 
    #[serde(rename = "snapshot")]
    _snapshot: Option<bool>, 
    parent_id: Option<Uuid>,
    knowledge_base_id: Option<Uuid>,
    slug: Option<String>,
    #[serde(rename = "type")]
    content_type: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search_content_handler(
    State(state): State<crate::interface::state::AppState>,
    _user: MaybeAuthenticatedUser,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    match state.repo.search(&params.q).await {
        Ok(articles) => (StatusCode::OK, Json(articles)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[axum::debug_handler]
pub async fn create_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateContentRequest>,
) -> impl IntoResponse {
    let permission_mode = match payload.visibility.as_str() {
        "Private" => PermissionMode::Private,
        "Internal" => PermissionMode::Internal,
        _ => PermissionMode::Public,
    };

    let status = match payload.status.as_deref().unwrap_or("Published") {
        "Draft" => ContentStatus::Draft,
        "Archived" => ContentStatus::Archived,
        _ => ContentStatus::Published,
    };

    // Duplicate Check
    if let Ok(Some(_)) = state.repo.find_by_title(&payload.title).await {
         return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Article with this title already exists" }))).into_response();
    }

    let id = Uuid::new_v4();
    
    // Check Content Type
    let content_type = payload.content_type.as_deref().unwrap_or("Article");
    
    if content_type == "Folder" || content_type == "Directory" {
        use crate::domain::ports::NodeRepository; // Import trait locally

        let node = Node {
            id,
            parent_id: payload.parent_id,
            author_id: user.id,
            knowledge_base_id: payload.knowledge_base_id,
            r#type: NodeType::Folder,
            title: payload.title,
            permission_mode,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        match NodeRepository::save(&*state.repo, node, UserId(user.id)).await { // Explicit NodeRepository call
            Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
            Err(e) => {
                tracing::error!("Failed to create folder: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
            }
        }
    } else {
        // Article Creation Logic
        let slug = payload.slug.unwrap_or_else(|| {
            format!("{}-{}", payload.title.to_lowercase().replace(" ", "-"), &id.to_string()[..8])
        });

        let article = Article {
            node: Node {
                id,
                parent_id: payload.parent_id,
                author_id: user.id,
                knowledge_base_id: payload.knowledge_base_id,
                r#type: NodeType::Article,
                title: payload.title,
                permission_mode,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            slug,
            status,
            category: payload.category,
            body: ContentBody::Markdown(payload.body),
            tags: payload.tags,
            author_name: None,
            author_avatar: None,
        };

        match ArticleRepository::save(&*state.repo, article, UserId(user.id), payload._reason).await { // Explicit ArticleRepository call
            Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
            Err(RepositoryError::DuplicateTitle(msg)) => (StatusCode::CONFLICT, Json(serde_json::json!({ "error": msg }))).into_response(),
            Err(RepositoryError::ValidationError(msg)) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg }))).into_response(),
            Err(e) => {
                tracing::error!("Failed to create content: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
            },
        }
    }
}

// Helper to get node from ContentItem
impl crate::domain::models::ContentItem {
    fn node(&self) -> &Node {
        match self {
            crate::domain::models::ContentItem::Article(a) => &a.node,
            crate::domain::models::ContentItem::Node(n) => n,
        }
    }
}

pub async fn update_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateContentRequest>, 
) -> impl IntoResponse {
    let existing_item = match state.repo.find_by_id(&id).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let existing_node = existing_item.node();

    // Check Author or Editor permission
    let is_author = existing_node.author_id == user.id;
    let is_editor = if is_author {
        true
    } else {
        use crate::domain::ports::PermissionRepository;
        PermissionRepository::has_relation(&*state.repo, id, "node", "editor", user.id, "user").await.unwrap_or(false)
    };

    if !is_editor {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // If it's a Node (Folder), only support partial updates or reject for now.
    // For MVP, support renaming Folders.
    match existing_item {
        crate::domain::models::ContentItem::Node(mut n) => {
            if payload.content_type.as_deref() == Some("Article") {
                 return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Cannot change type to Article" }))).into_response();
            }
             // Update Node fields
             n.title = payload.title;
             n.updated_at = Utc::now();
             // n.parent_id = ... logic?
             
             use crate::domain::ports::NodeRepository;
             match NodeRepository::save(&*state.repo, n, UserId(user.id)).await {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        },
        crate::domain::models::ContentItem::Article(existing) => {
             // Article Update Logic (Same as before)
             let permission_mode = match payload.visibility.as_str() {
                "Private" => PermissionMode::Private,
                "Internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            };
            
            let status = match payload.status.as_deref().unwrap_or("Published") {
                "Draft" => ContentStatus::Draft,
                "Archived" => ContentStatus::Archived,
                _ => ContentStatus::Published,
            };

            let updated_article = Article {
                node: Node {
                    id,
                    parent_id: payload.parent_id.or(existing.node.parent_id),
                    author_id: user.id,
                    knowledge_base_id: payload.knowledge_base_id.or(existing.node.knowledge_base_id),
                    r#type: NodeType::Article,
                    title: payload.title.clone(),
                    permission_mode,
                    created_at: existing.node.created_at,
                    updated_at: Utc::now(),
                },
                slug: existing.slug.clone(),
                status,
                category: payload.category,
                body: ContentBody::Markdown(payload.body),
                tags: payload.tags,
                author_name: None,
                author_avatar: None,
            };

            // Duplicate Check (Title exists and ID is different)
            if let Ok(Some(conflict)) = state.repo.find_by_title(&payload.title).await {
                if conflict.node.id != existing.node.id {
                     return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Article with this title already exists" }))).into_response();
                }
            }

            match ArticleRepository::save(&*state.repo, updated_article, UserId(user.id), payload._reason).await {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
                Err(RepositoryError::DuplicateTitle(msg)) => (StatusCode::CONFLICT, Json(serde_json::json!({ "error": msg }))).into_response(),
                Err(RepositoryError::ValidationError(msg)) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
    }
}

// ... ListContentHandler
#[derive(serde::Deserialize)]
pub struct ListParams {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub author_id: Option<Uuid>,
    pub knowledge_base_id: Option<Uuid>,
}

pub async fn list_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let viewer_id = user.0.map(|u| UserId(u.id));
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let author_id = params.author_id.map(UserId);
    let knowledge_base_id = params.knowledge_base_id;

    match state.repo.list(viewer_id, author_id, knowledge_base_id, limit, offset).await {
        Ok(articles) => (StatusCode::OK, Json(articles)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user = user.0;

    match state.repo.find_by_id(&id).await {
        Ok(Some(item)) => {
            let node = item.node();
            let is_author = user.as_ref().map(|u| u.id == node.author_id).unwrap_or(false);
            
            // Draft check only applies to Articles really, but safe to check if item is article
            if let crate::domain::models::ContentItem::Article(ref a) = item {
                if a.status == ContentStatus::Draft && !is_author {
                     return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
                }
            }

            let can_view = match node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.is_some(),
                PermissionMode::Private => {
                    if is_author { true }
                    else if let Some(u) = &user {
                        use crate::domain::ports::PermissionRepository;
                         PermissionRepository::has_relation(
                            &*state.repo, 
                            node.id, 
                            "node", 
                            "editor", 
                            u.id, 
                            "user"
                        ).await.unwrap_or(false)
                    } else { false }
                },
            };

            if can_view {
                (StatusCode::OK, Json(item)).into_response()
            } else {
                (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response()
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let existing_item = match state.repo.find_by_id(&id).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    
    let node = existing_item.node();

    if node.author_id != user.id && !user.has_permission(0x0100) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    match state.repo.delete(&id).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_content_history_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.find_by_id(&id).await {
        Ok(Some(item)) => {
             let node = item.node();
             let user = user.0;
             let is_author = user.as_ref().map(|u| u.id == node.author_id).unwrap_or(false);
             let can_view = match node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.is_some(),
                PermissionMode::Private => {
                    if is_author { true }
                    else if let Some(u) = &user {
                        use crate::domain::ports::PermissionRepository;
                         PermissionRepository::has_relation(
                            &*state.repo, 
                            node.id, 
                            "node", 
                            "editor", 
                            u.id, 
                            "user"
                        ).await.unwrap_or(false)
                    } else { false }
                },
            };
            if !can_view {
                 return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
            }
        },
        _ => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
    }

    match state.repo.get_history(&id).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ... get_content_version_handler similar update ...

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/content", post(create_content_handler).get(list_content_handler))
        .route("/api/content/:id", get(get_content_handler).put(update_content_handler).delete(delete_content_handler))
        .route("/api/content/:id/history", get(get_content_history_handler))
        .route("/api/content/:id/history/:version", get(get_content_version_handler))
        .route("/api/content/:id/diff/:v1/:v2", get(get_content_diff_handler))
        .route("/api/content/:id/collaborators", get(list_collaborators_handler).post(add_collaborator_handler))
        .route("/api/content/:id/collaborators/:uid", axum::routing::delete(remove_collaborator_handler))
        .route("/api/search", get(search_content_handler))
}

pub async fn get_content_version_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path((id, version)): Path<(Uuid, String)>,
) -> impl IntoResponse {
     match state.repo.find_by_id(&id).await {
        Ok(Some(item)) => {
             let node = item.node();
             let user = user.0;
             let is_author = user.as_ref().map(|u| u.id == node.author_id).unwrap_or(false);
             let can_view = match node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.is_some(),
                PermissionMode::Private => {
                    if is_author { true }
                    else if let Some(u) = &user {
                        use crate::domain::ports::PermissionRepository;
                         PermissionRepository::has_relation(
                            &*state.repo, 
                            node.id, 
                            "node", 
                            "editor", 
                            u.id, 
                            "user"
                        ).await.unwrap_or(false)
                    } else { false }
                },
            };
            if !can_view {
                 return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
            }
        },
        _ => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
    }

    match state.repo.get_version(&id, &version).await {
        Ok(Some(v)) => (StatusCode::OK, Json(v)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Version not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_content_diff_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path((id, v1, v2)): Path<(Uuid, String, String)>,
) -> impl IntoResponse {
    // Check permissions (reusing logic from get_version would be better, but copying for simplicity now)
     match state.repo.find_by_id(&id).await {
        Ok(Some(item)) => {
             let node = item.node();
             let user_id = user.0.as_ref().map(|u| u.id);
             let is_author = user_id.map(|uid| uid == node.author_id).unwrap_or(false);
             let can_view = match node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.0.is_some(),
                PermissionMode::Private => {
                    if is_author { true }
                    else if let Some(u) = &user.0 {
                        use crate::domain::ports::PermissionRepository;
                         PermissionRepository::has_relation(
                            &*state.repo, 
                            node.id, 
                            "node", 
                            "editor", 
                            u.id, 
                            "user"
                        ).await.unwrap_or(false)
                    } else { false }
                },
            };
            if !can_view {
                 return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
            }
        },
        _ => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
    }

    match state.repo.get_diff(&id, &v1, &v2).await {
        Ok(diff) => (StatusCode::OK, Json(diff)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// Collaborator Management
#[derive(serde::Deserialize)]
pub struct AddCollaboratorRequest {
    pub user_id: Uuid,
}

#[derive(serde::Serialize)]
pub struct CollaboratorResponse {
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}

pub async fn list_collaborators_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Verify Requestor Permission (Owner only)
    use crate::domain::ports::ArticleRepository;
    let item = match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    
    if item.node().author_id != user.id {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Only owner can list collaborators" }))).into_response();
    }

    // 2. Query Relations
    use crate::domain::ports::PermissionRepository;
    let collaborator_ids = match PermissionRepository::get_collaborators(&*state.repo, id, "node", "editor").await {
        Ok(ids) => ids,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    use crate::domain::ports::UserRepository;
    let mut collaborators = Vec::new();
    for uid in collaborator_ids {
         // Explicitly call UserRepository to avoid ambiguity with ArticleRepository/NodeRepository
         if let Ok(Some(u)) = UserRepository::find_by_id(&*state.repo, &crate::domain::models::UserId(uid)).await {
             collaborators.push(CollaboratorResponse {
                 user_id: u.id.0,
                 username: u.username,
                 avatar_url: u.avatar_url,
             });
         }
    }

    (StatusCode::OK, Json(collaborators)).into_response()
}

pub async fn add_collaborator_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddCollaboratorRequest>,
) -> impl IntoResponse {
    // 1. Verify Permission (Owner)
     let item = match state.repo.find_by_id(&id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if item.node().author_id != user.id {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Only owner can add collaborators" }))).into_response();
    }

    // 2. Add Relation: (Node, "editor", User)
    // Note: We need a PermissionService/Repository access. `state.repo` implements PermissionRepository via PostgresRepository.
    use crate::domain::ports::PermissionRepository;
    match PermissionRepository::add_relation(&*state.repo, id, "node", "editor", payload.user_id, "user").await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "added" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn remove_collaborator_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path((id, target_user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    // 1. Verify Permission (Owner)
     let item = match state.repo.find_by_id(&id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if item.node().author_id != user.id {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Only owner can remove collaborators" }))).into_response();
    }

    // 2. Remove Relation
    use crate::domain::ports::PermissionRepository;
    match PermissionRepository::remove_relation(&*state.repo, id, "node", "editor", target_user_id, "user").await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

