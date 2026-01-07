use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Comment, CommentId, CommentableId, CommentableType};
use crate::domain::ports::CommentRepository;
use crate::interface::api::auth::AuthenticatedUser;

#[derive(serde::Deserialize, Debug)]
pub struct CreateCommentRequest {
    pub text: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
}

fn parse_target(target_type: &str, target_id: Uuid) -> Option<CommentableId> {
    let c_type = match target_type.to_lowercase().as_str() {
        "content" => CommentableType::Content,
        "memo" => CommentableType::Memo,
        _ => return None,
    };
    Some(CommentableId {
        target_type: c_type,
        target_id,
    })
}

pub async fn create_comment_handler(
    State(repo): State<Arc<dyn CommentRepository>>,
    user: AuthenticatedUser,
    Path((target_type, target_id)): Path<(String, Uuid)>,
    Json(payload): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    let target = match parse_target(&target_type, target_id) {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, "Invalid target type").into_response(),
    };

    let comment = Comment {
        id: CommentId(Uuid::new_v4()),
        target,
        user_id: crate::domain::models::UserId(user.id),
        user_name: None, // Will be filled by repo join (or should be provided?)
        // Actually repo join fills it on read. On write we just save IDs.
        // But for response we might want it? The repo returns CommentId currently.
        // Let's assume repo won't return full object immediately or we don't need it.
        user_avatar: None,
        parent_id: payload.parent_id.map(CommentId),
        text: payload.text,
        created_at: Utc::now(),
        replies: vec![],
    };

    match repo.add_comment(comment).await {
        Ok(id) => (StatusCode::CREATED, Json(id)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create comment: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn get_comments_handler(
    State(repo): State<Arc<dyn CommentRepository>>,
    Path((target_type, target_id)): Path<(String, Uuid)>,
) -> impl IntoResponse {
    let target = match parse_target(&target_type, target_id) {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, "Invalid target type").into_response(),
    };

    match repo.get_comments(&target).await {
        Ok(comments) => Json(comments).into_response(),
        Err(e) => {
             tracing::error!("Failed to fetch comments: {:?}", e);
             (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch comments").into_response()
        }
    }
}

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::post;
    axum::Router::new()
        .route("/api/comments/:type/:id", post(create_comment_handler).get(get_comments_handler))
}


