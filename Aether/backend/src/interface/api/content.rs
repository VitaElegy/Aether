use axum::{
    Json, extract::{State, Path, Query}, response::IntoResponse, http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::{
    ports::ContentRepository,
    models::{ContentAggregate, ContentId, ContentStatus, Visibility, ContentBody, UserId},
    diff_service::DiffService,
};
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};

#[derive(Deserialize)]
pub struct CreateContentRequest {
    title: String,
    body: String, // Assumed markdown for now
    tags: Vec<String>,
    category: Option<String>,
    visibility: String, // "Public", "Private", "Internal"
    status: Option<String>, // Added status field
    reason: Option<String>, // Git-like commit message
    snapshot: Option<bool>, // Control version snapshot creation
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}


fn parse_version(version_str: &str) -> Result<i32, String> {
    if version_str.starts_with("0.0.") {
        version_str.split('.').nth(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| "Invalid version format".to_string())
    } else {
        version_str.parse::<i32>()
            .map_err(|_| "Invalid version format".to_string())
    }
}

pub async fn search_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let user = user.0;

    let contents = match repo.search(&params.q).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let filtered: Vec<_> = contents.into_iter().filter(|c| {
        // Filter out Drafts if not the author
        if c.status == ContentStatus::Draft {
             if let Some(ref u) = user {
                 if u.id != c.author_id { return false; }
             } else {
                 return false;
             }
        }

        match c.visibility {
            Visibility::Public => true,
            Visibility::Internal => user.is_some(),
            Visibility::Private => user.as_ref().map(|u| u.id == c.author_id).unwrap_or(false),
        }
    }).collect();

    (StatusCode::OK, Json(filtered)).into_response()
}

pub async fn create_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateContentRequest>,
) -> impl IntoResponse {
    let visibility = match payload.visibility.as_str() {
        "Private" => Visibility::Private,
        "Internal" => Visibility::Internal,
        _ => Visibility::Public,
    };

    let status = match payload.status.as_deref().unwrap_or("Published") {
        "Draft" => ContentStatus::Draft,
        "Archived" => ContentStatus::Archived,
        _ => ContentStatus::Published,
    };

    // Generate deterministic ID based on User + Title to prevent duplicates
    let name = format!("{}:{}", user.id, payload.title);
    let id = ContentId(Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()));

    // Check for existing content to prevent duplicates/overwrites on create
    if let Ok(Some(_)) = repo.find_by_id(&id).await {
         return (StatusCode::CONFLICT, Json(serde_json::json!({
             "error": "Content already exists",
             "id": id.0
         }))).into_response();
    }

    let content = ContentAggregate {
        id,
        author_id: user.id,
        author_name: None, // Filled by Repo on read, not needed on write
        title: payload.title.clone(),
        slug: payload.title.to_lowercase().replace(" ", "-") + "-" + &Uuid::new_v4().to_string()[..8],
        status,
        visibility,
        category: payload.category,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        body: ContentBody::Markdown(payload.body),
        tags: payload.tags,
        version_message: payload.reason,
    };

    // Default snapshot policy: true if Published, false if Draft (unless overridden)
    let should_snapshot = payload.snapshot.unwrap_or(content.status == ContentStatus::Published);

    match repo.save(content, crate::domain::models::UserId(user.id), should_snapshot).await {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id.0 }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create content: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        },
    }
}

pub async fn update_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateContentRequest>, // Reusing CreateContentRequest for update
) -> impl IntoResponse {
    // 1. Fetch existing content to verify ownership
    let existing = match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    if existing.author_id != user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 2. Update fields
    let visibility = match payload.visibility.as_str() {
        "Private" => Visibility::Private,
        "Internal" => Visibility::Internal,
        _ => Visibility::Public,
    };

    let status = match payload.status.as_deref().unwrap_or("Published") {
        "Draft" => ContentStatus::Draft,
        "Archived" => ContentStatus::Archived,
        _ => ContentStatus::Published,
    };

    let updated_content = ContentAggregate {
        id: ContentId(id),
        author_id: user.id,
        author_name: None, // Filled by Repo on read, not needed on write
        title: payload.title,
        slug: existing.slug, // Keep slug or regenerate? Keeping for now.
        status,
        visibility,
        category: payload.category,
        created_at: existing.created_at,
        updated_at: Utc::now(),
        body: ContentBody::Markdown(payload.body),
        tags: payload.tags,
        version_message: payload.reason,
    };

    // Default snapshot policy: true if Published, false if Draft (unless overridden)
    let should_snapshot = payload.snapshot.unwrap_or(updated_content.status == ContentStatus::Published);

    // 3. Save
    match repo.save(updated_content, crate::domain::models::UserId(user.id), should_snapshot).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update content: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        },
    }
}

#[derive(serde::Deserialize)]
pub struct ListParams {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub author_id: Option<Uuid>,
}

pub async fn list_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let viewer_id = user.0.map(|u| UserId(u.id));
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let author_id = params.author_id.map(UserId);

    tracing::info!(
        "List content request: viewer_id={:?}, author_id={:?}, limit={}, offset={}",
        viewer_id.as_ref().map(|id| id.0),
        author_id.as_ref().map(|id| id.0),
        limit,
        offset
    );

    match repo.list(viewer_id, author_id, limit, offset).await {
        Ok(contents) => {
            tracing::info!("Found {} contents", contents.len());
            (StatusCode::OK, Json(contents)).into_response()
        },
        Err(e) => {
            tracing::error!("Failed to list contents: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        },
    }
}

pub async fn get_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user = user.0;

    match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(content)) => {
            let is_author = user.as_ref().map(|u| u.id == content.author_id).unwrap_or(false);

            if content.status == ContentStatus::Draft && !is_author {
                 return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
            }

            let can_view = match content.visibility {
                Visibility::Public => true,
                Visibility::Internal => user.is_some(),
                Visibility::Private => is_author,
            };

            if can_view {
                (StatusCode::OK, Json(content)).into_response()
            } else {
                (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response()
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Fetch to verify ownership
    let existing = match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    // Author or Admin (assuming Admin bit is 0x0100 from models.rs comment)
    if existing.author_id != user.id && !user.has_permission(0x0100) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    match repo.delete(&ContentId(id)).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_content_diff_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    _user: AuthenticatedUser,
    Path((id, v1_str, v2_str)): Path<(Uuid, String, String)>,
) -> impl IntoResponse {
    let v1 = match parse_version(&v1_str) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    };
    let v2 = match parse_version(&v2_str) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    };

    // 1. Check access (simplified: author or public)
    let _content = match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(c)) => c,
        _ => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
    };

    // 2. Fetch both versions
    let v1_data = repo.get_version(&ContentId(id), v1).await.unwrap_or(None);
    let v2_data = repo.get_version(&ContentId(id), v2).await.unwrap_or(None);

    if v1_data.is_none() || v2_data.is_none() {
         return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Version not found" }))).into_response();
    }

    let (title1, body1) = v1_data.unwrap();
    let (title2, body2) = v2_data.unwrap();

    // Helper to extract text from JSON body (e.g. Markdown data)
    let extract_text = |title: &str, json_body: &str| -> String {
        let body_text = if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_body) {
             // Expecting { type: "Markdown", data: "..." }
             if let Some(data) = val.get("data").and_then(|v| v.as_str()) {
                 data.to_string()
             } else {
                 json_body.to_string()
             }
        } else {
            json_body.to_string()
        };
        format!("Title: {}\n\n{}", title, body_text)
    };

    let t1 = extract_text(&title1, &body1);
    let t2 = extract_text(&title2, &body2);

    let diff = DiffService::compute_diff(&t1, &t2);

    (StatusCode::OK, Json(diff)).into_response()
}

pub async fn get_history_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Check visibility permission
    let content = match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let user = user.0;
    let is_author = user.as_ref().map(|u| u.id == content.author_id).unwrap_or(false);
    let can_view = match content.visibility {
        Visibility::Public => true,
        Visibility::Internal => user.is_some(),
        Visibility::Private => is_author,
    };

    if !can_view {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 2. Fetch History
    match repo.get_history(&ContentId(id)).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_version_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
    Path((id, version_str)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    let version = match parse_version(&version_str) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    };

    // 1. Check permission
     let content = match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Content not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let user = user.0;
    let is_author = user.as_ref().map(|u| u.id == content.author_id).unwrap_or(false);
    let can_view = match content.visibility {
        Visibility::Public => true,
        Visibility::Internal => user.is_some(),
        Visibility::Private => is_author,
    };

    if !can_view {
         return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Access denied" }))).into_response();
    }

    // 2. Fetch Version
    match repo.get_version(&ContentId(id), version).await {
        Ok(Some((title, body_str))) => {
            // Unpack JSON body to embed in response object
            let body_json: serde_json::Value = serde_json::from_str(&body_str).unwrap_or(serde_json::Value::String(body_str));

            let response = serde_json::json!({
                "title": title,
                "body": body_json
            });

            (StatusCode::OK, Json(response)).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Version not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
