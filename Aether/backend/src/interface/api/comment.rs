use axum::{
    extract::{Path, State, Json},
    response::IntoResponse,
    http::StatusCode,
    Extension,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Comment, CommentId, ContentId, UserId, AuthClaims};
use crate::domain::ports::CommentRepository;

#[derive(serde::Deserialize)]
pub struct CreateCommentRequest {
    pub text: String,
    pub parent_id: Option<String>,
}

pub async fn create_comment_handler(
    State(repo): State<Arc<dyn CommentRepository>>,
    Extension(claims): Extension<AuthClaims>,
    Path(content_id): Path<Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    // Validate
    if payload.text.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Comment cannot be empty").into_response();
    }

    let comment_id = Uuid::new_v4();
    // Safety check for parent_id validity could be added here, but DB will catch FK error usually.
    let parent_uuid = if let Some(pid) = payload.parent_id {
        match uuid::Uuid::parse_str(&pid) {
            Ok(u) => Some(CommentId(u)),
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid parent ID").into_response(),
        }
    } else {
        None
    };

    let user_uuid = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(u) => u,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    let comment = Comment {
        id: CommentId(comment_id),
        content_id: ContentId(content_id),
        user_id: UserId(user_uuid),
        user_name: None, // Will be filled on read
        user_avatar: None,
        parent_id: parent_uuid,
        text: payload.text,
        created_at: Utc::now(),
        replies: Vec::new(),
    };

    match repo.add_comment(comment).await {
        Ok(id) => (StatusCode::CREATED, Json(id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_comments_handler(
    State(repo): State<Arc<dyn CommentRepository>>,
    Path(content_id): Path<Uuid>,
) -> impl IntoResponse {
    match repo.get_comments(&ContentId(content_id)).await {
        Ok(comments) => Json(comments).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
