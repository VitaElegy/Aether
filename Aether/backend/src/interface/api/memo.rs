use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Memo, Node, NodeType, PermissionMode, UserId};
use crate::domain::ports::MemoRepository; // Import Trait
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};
use crate::interface::state::AppState;

#[derive(serde::Deserialize, Debug)]
pub struct CreateMemoRequest {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub visibility: String,
    pub priority: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListMemosRequest {
    pub author_id: Option<Uuid>,
}

pub async fn create_memo_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateMemoRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let memo = Memo {
        node: Node {
            id,
            parent_id: None, 
            author_id: user.id,
            r#type: NodeType::Memo,
            title: payload.title,
            permission_mode: match payload.visibility.to_lowercase().as_str() {
                "private" => PermissionMode::Private,
                "internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        content: payload.content,
        priority: Some(payload.priority.unwrap_or("Medium".to_string())),
        tags: payload.tags,
    };

    match state.repo.save(memo).await {
        Ok(id) => (StatusCode::CREATED, Json::<Uuid>(id)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create memo").into_response()
        }
    }
}

pub async fn get_memo_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.find_by_id(&id).await {
        Ok(Some(memo)) => Json::<Memo>(memo).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch memo").into_response()
        }
    }
}

pub async fn list_memos_handler(
    State(state): State<AppState>,
    user: MaybeAuthenticatedUser,
    Query(params): Query<ListMemosRequest>,
) -> impl IntoResponse {
    let viewer_id = user.0.map(|u| UserId(u.id));
    let author_id = params.author_id.map(UserId);

    let target_author_id = author_id.or(viewer_id.clone());

    if target_author_id.is_none() {
         return Json::<Vec<Memo>>(Vec::new()).into_response();
    }

    match state.repo.list(viewer_id, target_author_id).await {
         Ok(memos) => Json::<Vec<Memo>>(memos).into_response(),
         Err(e) => {
             tracing::error!("Failed to list memos: {:?}", e);
             (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list memos").into_response()
         }
    }
}

pub async fn delete_memo_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.find_by_id(&id).await {
        Ok(Some(memo)) => {
            if memo.node.author_id != user.id {
                return (StatusCode::FORBIDDEN, "Not authorized to delete this memo").into_response();
            }
        },
        Ok(None) => return (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }

    match state.repo.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete memo").into_response()
        }
    }
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/memos", post(create_memo_handler).get(list_memos_handler))
        .route("/api/memos/:id", get(get_memo_handler).delete(delete_memo_handler))
}
