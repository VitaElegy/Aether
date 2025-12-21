use axum::{
    Json, extract::{State, Path}, response::IntoResponse, http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::{
    ports::ContentRepository,
    models::{ContentAggregate, ContentId, ContentStatus, Visibility, ContentBody},
};
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};

#[derive(Deserialize)]
pub struct CreateContentRequest {
    title: String,
    body: String, // Assumed markdown for now
    tags: Vec<String>,
    category: Option<String>,
    visibility: String, // "Public", "Private", "Internal"
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

    let content = ContentAggregate {
        id: ContentId(Uuid::new_v4()),
        author_id: user.id,
        title: payload.title.clone(),
        slug: payload.title.to_lowercase().replace(" ", "-") + "-" + &Uuid::new_v4().to_string()[..8],
        status: ContentStatus::Published, // Default to Published for simplicity
        visibility,
        category: payload.category,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        body: ContentBody::Markdown(payload.body),
        tags: payload.tags,
    };

    match repo.save(content).await {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id.0 }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn list_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
) -> impl IntoResponse {
    let user = user.0;

    let contents = match repo.list(100, 0).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let filtered: Vec<_> = contents.into_iter().filter(|c| {
        match c.visibility {
            Visibility::Public => true,
            Visibility::Internal => user.is_some(),
            Visibility::Private => user.as_ref().map(|u| u.id == c.author_id).unwrap_or(false),
        }
    }).collect();

    (StatusCode::OK, Json(filtered)).into_response()
}

pub async fn get_content_handler(
    State(repo): State<Arc<dyn ContentRepository>>,
    user: MaybeAuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user = user.0;

    match repo.find_by_id(&ContentId(id)).await {
        Ok(Some(content)) => {
            let can_view = match content.visibility {
                Visibility::Public => true,
                Visibility::Internal => user.is_some(),
                Visibility::Private => user.as_ref().map(|u| u.id == content.author_id).unwrap_or(false),
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

