use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Memo, MemoId, Visibility, UserId};
use crate::domain::ports::MemoRepository;
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};

#[derive(serde::Deserialize, Debug)]
pub struct CreateMemoRequest {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub visibility: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListMemosRequest {
    pub author_id: Option<Uuid>,
}

pub async fn create_memo_handler(
    State(repo): State<Arc<dyn MemoRepository>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateMemoRequest>,
) -> impl IntoResponse {
    let memo = Memo {
        id: MemoId(Uuid::new_v4()),
        author_id: user.id,
        title: payload.title,
        content: payload.content,
        tags: payload.tags,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        visibility: match payload.visibility.to_lowercase().as_str() {
            "private" => Visibility::Private,
            "internal" => Visibility::Internal,
            _ => Visibility::Public,
        },
    };

    match repo.save(memo).await {
        Ok(id) => (StatusCode::CREATED, Json(id)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create memo").into_response()
        }
    }
}

pub async fn get_memo_handler(
    State(repo): State<Arc<dyn MemoRepository>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match repo.find_by_id(&MemoId(id)).await {
        Ok(Some(memo)) => Json(memo).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch memo").into_response()
        }
    }
}

pub async fn list_memos_handler(
    State(repo): State<Arc<dyn MemoRepository>>,
    user: MaybeAuthenticatedUser,
    Query(params): Query<ListMemosRequest>,
) -> impl IntoResponse {
    let viewer_id = user.0.map(|u| UserId(u.id));
    let author_id = params.author_id.map(UserId);

    // Similar logic: if no author_id, default to viewer's own memos.
    // If guest and no author_id, return empty.
    let target_author_id = author_id.or(viewer_id.clone());

    if target_author_id.is_none() {
         return Json(Vec::<Memo>::new()).into_response();
    }

    match repo.list(viewer_id, target_author_id).await {
         Ok(memos) => Json(memos).into_response(),
         Err(e) => {
             tracing::error!("Failed to list memos: {:?}", e);
             (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list memos").into_response()
         }
    }
}

pub async fn delete_memo_handler(
    State(repo): State<Arc<dyn MemoRepository>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Check ownership or admin
    match repo.find_by_id(&MemoId(id)).await {
        Ok(Some(memo)) => {
            // Simplified permission check: only author can delete for now
            if memo.author_id != user.id {
                return (StatusCode::FORBIDDEN, "Not authorized to delete this memo").into_response();
            }
        },
        Ok(None) => return (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }

    match repo.delete(&MemoId(id)).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete memo").into_response()
        }
    }
}

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/memos", post(create_memo_handler).get(list_memos_handler))
        .route("/api/memos/:id", get(get_memo_handler).delete(delete_memo_handler))
}
