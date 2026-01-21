use axum::{
    Json, extract::{State, Path, Query}, response::IntoResponse, http::StatusCode,
};
use crate::domain::{
    ports::{ArticleRepository, RepositoryError, PermissionRepository, UserRepository}, 
    models::{Article, Node, NodeType, PermissionMode, ContentBody, ContentStatus, UserId},
};
// use crate::infrastructure::persistence::entities::draft;
use uuid::Uuid;
use chrono::Utc;
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CollaboratorInfo {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct ContentResponse {
    #[serde(flatten)]
    pub item: crate::domain::models::ContentItem,
    pub user_permission: String, // "author", "editor", "viewer"
    pub collaborators: Vec<CollaboratorInfo>,
}

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
pub struct SaveDraftRequest {
    title: String,
    body: serde_json::Value,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

// --- Permission Helpers ---

async fn check_view_permission(
    repo: &crate::infrastructure::persistence::postgres::PostgresRepository,
    node: &Node,
    user: &Option<AuthenticatedUser>,
) -> bool {
    // 1. Author can always view
    if let Some(u) = user {
        if u.id == node.author_id {
            return true;
        }
    }

    match node.permission_mode {
        PermissionMode::Public => true,
        PermissionMode::Internal => user.is_some(),
        PermissionMode::Private => {
            if let Some(u) = user {
                // Check if explicitly granted "editor" or "viewer" permissions
                 PermissionRepository::has_relation(
                    repo, 
                    node.id, 
                    "node", 
                    "editor", 
                    u.id, 
                    "user"
                ).await.unwrap_or(false)
            } else {
                false
            }
        },
    }
}

async fn check_edit_permission(
    repo: &crate::infrastructure::persistence::postgres::PostgresRepository,
    node: &Node,
    user: &AuthenticatedUser,
) -> bool {
    // 1. Author can always edit
    if user.id == node.author_id {
        return true;
    }

    // 2. Check "editor" relation
    PermissionRepository::has_relation(
        repo, 
        node.id, 
        "node", 
        "editor", 
        user.id, 
        "user"
    ).await.unwrap_or(false)
}

// --- Handlers ---

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

    let status = match payload.status.as_deref().unwrap_or("Draft") {
        "Published" => ContentStatus::Published,
        "Archived" => ContentStatus::Archived,
        _ => ContentStatus::Draft,
    };

    if let Ok(Some(_)) = state.repo.find_by_title(&payload.title).await {
         return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Article with this title already exists" }))).into_response();
    }

    let id = Uuid::new_v4();
    let content_type = payload.content_type.as_deref().unwrap_or("Article");
    
    if content_type == "Folder" || content_type == "Directory" {
        use crate::domain::ports::NodeRepository;

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

        match NodeRepository::save(&*state.repo, node, UserId(user.id)).await {
            Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
            Err(e) => {
                tracing::error!("Failed to create folder: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
            }
        }
    } else {
        let slug = payload.slug.unwrap_or_else(|| {
            format!("{}-{}", payload.title.to_lowercase().replace(" ", "-"), &id.to_string()[..8])
        });
        
        let body_content = payload.body.clone();

        // Prepare derived_data before moving payload fields
        let derived_data_value = {
             let text_to_parse = if payload.category.as_deref() == Some("English Analysis") {
                 serde_json::from_str::<serde_json::Value>(&payload.body)
                     .ok()
                     .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
                     .unwrap_or_else(|| payload.body.clone())
             } else {
                 payload.body.clone()
             };
             
             Some(serde_json::to_value(
                crate::domain::sentence_parser::SentenceParser::parse(&text_to_parse, None) 
            ).unwrap_or(serde_json::Value::Null))
        };

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
            derived_data: derived_data_value,
        };

        match ArticleRepository::save(&*state.repo, article, UserId(user.id), payload._reason).await {
            Ok(id) => {
                // Background Indexing for Graph
                let indexer = state.indexer_service.clone();
                let body = body_content;
                tokio::spawn(async move {
                    if let Err(e) = indexer.index_article(id, &body).await {
                        tracing::error!("Async Indexing failed for {}: {}", id, e);
                    }
                });

                (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response()
            },
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
    let existing_item = match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let existing_node = existing_item.node();

    // Unified Permission Check
    if !check_edit_permission(&state.repo, existing_node, &user).await {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    match existing_item {
        crate::domain::models::ContentItem::Node(mut n) => {
             if payload.content_type.as_deref() == Some("Article") {
                 return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Cannot change type to Article" }))).into_response();
            }
             n.title = payload.title;
             n.updated_at = Utc::now();
             
             use crate::domain::ports::NodeRepository;
             match NodeRepository::save(&*state.repo, n, UserId(user.id)).await {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        },
        crate::domain::models::ContentItem::Article(existing) => {
             let permission_mode = match payload.visibility.as_str() {
                "Private" => PermissionMode::Private,
                "Internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            };
            
            let status = match payload.status.as_deref() {
                Some("Draft") => ContentStatus::Draft,
                Some("Archived") => ContentStatus::Archived,
                Some("Published") => ContentStatus::Published,
                _ => existing.status, // KEEP EXISTING STATUS
            };

            let body_content = payload.body.clone();

            // Prepare derived_data before moving payload
            let derived_data_value = {
                    // Extract old map
                    let old_map = existing.derived_data
                        .and_then(|v| serde_json::from_value::<crate::domain::sentence_parser::SentenceMap>(v).ok());
                    
                    let text_to_parse = if payload.category.as_deref() == Some("English Analysis") {
                         serde_json::from_str::<serde_json::Value>(&payload.body)
                             .ok()
                             .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
                             .unwrap_or_else(|| payload.body.clone())
                    } else {
                        payload.body.clone()
                    };

                    Some(serde_json::to_value(
                        crate::domain::sentence_parser::SentenceParser::parse(&text_to_parse, old_map.as_ref())
                    ).unwrap_or(serde_json::Value::Null))
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
                derived_data: derived_data_value,
            };

            if let Ok(Some(conflict)) = state.repo.find_by_title(&payload.title).await {
                if conflict.node.id != existing.node.id {
                     return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Article with this title already exists" }))).into_response();
                }
            }

            match ArticleRepository::save(&*state.repo, updated_article, UserId(user.id), payload._reason).await {
                Ok(_) => {
                    // Background Indexing
                    let indexer = state.indexer_service.clone();
                    // body_content is owned string here, we move it to async block
                    tokio::spawn(async move {
                         if let Err(e) = indexer.index_article(id, &body_content).await {
                             tracing::error!("Async Indexing failed for {}: {}", id, e);
                         }
                    });
                    (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response()
                },
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
    pub tag: Option<String>,
    pub category: Option<String>,
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
    let tag = params.tag;
    let category = params.category;

    let mut items = match state.repo.list(viewer_id, author_id, knowledge_base_id, tag, category, limit, offset).await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    // Overlay Draft content for Author (Shadow Draft visibility)
    if let Some(uid) = user.0.as_ref() {
        let is_self_view = params.author_id == Some(uid.id);
        if is_self_view {
            // use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
            
            // Collect Article IDs
            let article_ids: Vec<Uuid> = items.iter().filter_map(|i| {
                if let crate::domain::models::ContentItem::Article(a) = i {
                    Some(a.node.id)
                } else {
                    None
                }
            }).collect();

            if !article_ids.is_empty() {
                // Fetch Drafts via Repository (Clean Arch)
                match state.repo.find_drafts_by_article_ids(article_ids).await {
                    Ok(drafts) => {
                         // Map drafts by Article ID
                        let draft_map: std::collections::HashMap<Uuid, (String, serde_json::Value, chrono::DateTime<chrono::Utc>)> = drafts.into_iter()
                            .map(|(id, title, body, updated)| (id, (title, body, updated)))
                            .collect();

                         // Apply Overlay
                        for item in items.iter_mut() {
                            if let crate::domain::models::ContentItem::Article(ref mut a) = item {
                                if let Some((d_title, d_body, d_updated)) = draft_map.get(&a.node.id) {
                                    // Overlay Title & Body (Excerpt)
                                    a.node.title = d_title.clone();
                                    // a.title = d_title.clone(); // Article struct relies on Node title, but let's check definition. 
                                    // If 'title' on Article struct doesn't exist, we skip.
                                    a.node.updated_at = (*d_updated).into();
                                    
                                    // Overlay Body for Excerpt generation
                                    let draft_body_str = d_body.to_string();
                                    a.body = crate::domain::models::ContentBody::Markdown(draft_body_str);
                                }
                            }
                        }
                    },
                    Err(e) => tracing::error!("Failed to fetch shadow drafts: {}", e),
                }
            }
        }
    }

    (StatusCode::OK, Json(items)).into_response()
}

pub async fn get_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(item)) => {
            let node = item.node();
            
            // Draft check
            if let crate::domain::models::ContentItem::Article(ref a) = item {
                let is_author = user.0.as_ref().map(|u| u.id == node.author_id).unwrap_or(false);
                if a.status == ContentStatus::Draft && !is_author {
                     return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
                }
            }

            // Unified Permission Check
            if check_view_permission(&state.repo, node, &user.0).await {
                // Determine Permission Level
                let user_permission = if let Some(u) = &user.0 {
                    if u.id == node.author_id {
                        "author"
                    } else if check_edit_permission(&state.repo, node, u).await {
                        "editor"
                    } else {
                        "viewer"
                    }
                } else {
                    "viewer"
                };

                // Fetch Collaborators (Editors)
                // Note: We use "editor" relation on "node" entity
                let collaborator_ids = PermissionRepository::get_collaborators(&*state.repo, node.id, "node", "editor").await.unwrap_or_default();
                let mut collaborators = Vec::new();
                for uid in collaborator_ids {
                    if let Ok(Some(u)) = UserRepository::find_by_id(&*state.repo, &UserId(uid)).await {
                        collaborators.push(CollaboratorInfo {
                            id: u.id.0,
                            username: u.username,
                            avatar_url: u.avatar_url,
                        });
                    }
                }

                let response = ContentResponse {
                    item,
                    user_permission: user_permission.to_string(),
                    collaborators,
                };

                 (StatusCode::OK, Json(response)).into_response()
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
    let existing_item = match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    
    let node = existing_item.node();

    if !check_edit_permission(&state.repo, node, &user).await {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    match state.repo.delete_recursive(&id).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_content_history_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(item)) => {
            // Unified Permission Check
             if !check_view_permission(&state.repo, item.node(), &user.0).await {
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

pub async fn get_content_version_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path((id, version)): Path<(Uuid, String)>,
) -> impl IntoResponse {
     match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(item)) => {
             // Unified Permission Check
             if !check_view_permission(&state.repo, item.node(), &user.0).await {
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
     match ArticleRepository::find_by_id(&*state.repo, &id).await {
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

    let mut collaborators = Vec::new();
    for uid in collaborator_ids {
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
     let item = match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if item.node().author_id != user.id {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Only owner can add collaborators" }))).into_response();
    }

    // 2. Add Relation: (Node, "editor", User)
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
     let item = match ArticleRepository::find_by_id(&*state.repo, &id).await {
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


// --- Draft Handlers ---

pub async fn save_draft_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SaveDraftRequest>,
) -> impl IntoResponse {
    
    // 1. Verify Article Exists & Permission
    let article = match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Article not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if !check_edit_permission(&state.repo, article.node(), &user).await {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 2. Upsert Draft via Repo
    match state.repo.save_draft(id, payload.title, payload.body).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "draft_saved" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn publish_draft_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Fetch Draft
    let (draft_title, draft_body) = match state.repo.find_draft_by_id(&id).await {
        Ok(Some((t, b))) => (t, b),
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "No draft found to publish" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    // 2. Fetch Article & Check Permission
    let article_item = match ArticleRepository::find_by_id(&*state.repo, &id).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Article not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if !check_edit_permission(&state.repo, article_item.node(), &user).await {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 3. Promote Draft to Article (Update Content & Status)
    if let crate::domain::models::ContentItem::Article(existing_article) = article_item {
        let derived_data_value = {
            let text_to_parse = if existing_article.category.as_deref() == Some("English Analysis") {
                 serde_json::from_value::<serde_json::Value>(draft_body.clone())
                     .ok()
                     .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
                     .unwrap_or("".to_string())
            } else {
                "".to_string() 
            };

            Some(serde_json::to_value(
                crate::domain::sentence_parser::SentenceParser::parse(&text_to_parse, None) // Full re-parse on publish
            ).unwrap_or(serde_json::Value::Null))
        };
        
        // Construct New Article State
        let mut updated_article = existing_article.clone();
        updated_article.node.updated_at = Utc::now();
        updated_article.status = crate::domain::models::ContentStatus::Published;
        updated_article.body = crate::domain::models::ContentBody::Markdown(draft_body.to_string());
        updated_article.derived_data = derived_data_value;
        updated_article.node.title = draft_title; // Update Node Title

        match ArticleRepository::save(&*state.repo, updated_article, UserId(user.id), Some("Published from Draft".to_string())).await {
             Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "published" }))).into_response(),
             Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Target is not an article" }))).into_response()
    }
}

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
        // Drafts
        .route("/api/drafts/:id", post(save_draft_handler)) // Using POST for upsert on ID? Or PUT? POST is fine.
        .route("/api/drafts/:id/publish", post(publish_draft_handler))
}

