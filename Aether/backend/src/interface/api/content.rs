use axum::{
    Json, extract::{State, Path, Query}, response::IntoResponse, http::StatusCode,
};
use crate::domain::{
    ports::ArticleRepository, 
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

    match state.repo.save(article, UserId(user.id)).await {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create content: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        },
    }
}

pub async fn update_content_handler(
    State(state): State<crate::interface::state::AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateContentRequest>, 
) -> impl IntoResponse {
    let existing = match state.repo.find_by_id(&id).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if existing.node.author_id != user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

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

    // No-Op Check
    if existing.node.title == updated_article.node.title 
       && existing.body == updated_article.body
       && existing.category == updated_article.category
       && existing.tags == updated_article.tags 
       && existing.node.permission_mode == updated_article.node.permission_mode
       && existing.status == updated_article.status
    {
         // Everything identical, no-op
         return (StatusCode::OK, Json(serde_json::json!({ "id": id, "status": "no-op" }))).into_response();
    }

    match state.repo.save(updated_article, UserId(user.id)).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

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
        Ok(Some(article)) => {
            let is_author = user.as_ref().map(|u| u.id == article.node.author_id).unwrap_or(false);
             if article.status == ContentStatus::Draft && !is_author {
                 return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
            }
            let can_view = match article.node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.is_some(),
                PermissionMode::Private => is_author,
            };

            if can_view {
                (StatusCode::OK, Json(article)).into_response()
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
    let existing = match state.repo.find_by_id(&id).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if existing.node.author_id != user.id && !user.has_permission(0x0100) {
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
    // Optional: Check permissions (read access required)
    // For now, let's assume public/internal logic similar to get_content or just allow if they can view the article.
    // Ideally we should check if they can view the article first.
    
    // Quick check: fetch article to verify permissions
    match state.repo.find_by_id(&id).await {
        Ok(Some(article)) => {
             let user = user.0;
             let is_author = user.as_ref().map(|u| u.id == article.node.author_id).unwrap_or(false);
             let can_view = match article.node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.is_some(),
                PermissionMode::Private => is_author,
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

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/content", post(create_content_handler).get(list_content_handler))
        .route("/api/content/:id", get(get_content_handler).put(update_content_handler).delete(delete_content_handler))
        .route("/api/content/:id/history", get(get_content_history_handler))
        .route("/api/content/:id/history/:version", get(get_content_version_handler))
        .route("/api/search", get(search_content_handler))
}

pub async fn get_content_version_handler(
    State(state): State<crate::interface::state::AppState>,
    user: MaybeAuthenticatedUser,
    Path((id, version)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    // Permission check similar to get_content_handler
     match state.repo.find_by_id(&id).await {
        Ok(Some(article)) => {
             let user = user.0;
             let is_author = user.as_ref().map(|u| u.id == article.node.author_id).unwrap_or(false);
             let can_view = match article.node.permission_mode {
                PermissionMode::Public => true,
                PermissionMode::Internal => user.is_some(),
                PermissionMode::Private => is_author,
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
