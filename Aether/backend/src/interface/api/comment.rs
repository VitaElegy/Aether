use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Comment, CommentId, UserId};
use crate::domain::ports::CommentRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;

#[derive(serde::Deserialize, Debug)]
pub struct CreateCommentRequest {
    pub text: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
}

pub async fn create_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((_target_type, target_id)): Path<(String, Uuid)>, 
    Json(payload): Json<CreateCommentRequest>,
) -> impl IntoResponse {
   
    let comment = Comment {
        id: CommentId(Uuid::new_v4()),
        target_id,
        user_id: UserId(user.id),
        user_name: None, 
        user_avatar: None,
        parent_id: payload.parent_id.map(CommentId),
        text: payload.text,
        created_at: Utc::now(),
        replies: vec![],
    };

    match state.repo.add_comment(comment).await {
        Ok(id) => (StatusCode::CREATED, Json::<CommentId>(id)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create comment: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn get_comments_handler(
    State(state): State<AppState>,
    Path((_target_type, target_id)): Path<(String, Uuid)>, 
) -> impl IntoResponse {
    match state.repo.get_comments(&target_id).await {
        Ok(comments) => Json::<Vec<Comment>>(comments).into_response(),
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
