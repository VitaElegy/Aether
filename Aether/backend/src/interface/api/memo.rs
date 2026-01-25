use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;
use chrono::{Utc, DateTime};
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
    pub status: Option<String>,
    pub color: Option<String>,
    pub is_pinned: Option<bool>,
    pub due_at: Option<DateTime<Utc>>,
    pub reminder_at: Option<DateTime<Utc>>,
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
            knowledge_base_id: None, // TODO: Support Project ID if passed
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
        priority: payload.priority.unwrap_or("P2".to_string()),
        status: payload.status.unwrap_or("Todo".to_string()),
        color: payload.color.unwrap_or("Yellow".to_string()),
        is_pinned: payload.is_pinned.unwrap_or(false),
        due_at: payload.due_at,
        reminder_at: payload.reminder_at,
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

#[derive(serde::Deserialize, Debug)]
pub struct UpdateMemoRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub color: Option<String>,
    pub is_pinned: Option<bool>,
    pub due_at: Option<DateTime<Utc>>,
    pub reminder_at: Option<DateTime<Utc>>,
}

pub async fn update_memo_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMemoRequest>,
) -> impl IntoResponse {
    // 1. Fetch Existing
    let existing_memo = match state.repo.find_by_id(&id).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 2. Check Permission
    if existing_memo.node.author_id != user.id {
        return (StatusCode::FORBIDDEN, "Not authorized to update this memo").into_response();
    }

    // 3. Update Fields
    let mut updated_memo = existing_memo;
    if let Some(t) = payload.title { updated_memo.node.title = t; }
    if let Some(c) = payload.content { updated_memo.content = c; }
    if let Some(tags) = payload.tags { updated_memo.tags = tags; }
    if let Some(vis) = payload.visibility {
         updated_memo.node.permission_mode = match vis.to_lowercase().as_str() {
            "private" => PermissionMode::Private,
            "internal" => PermissionMode::Internal,
            _ => PermissionMode::Public,
        };
    }
    if let Some(p) = payload.priority { updated_memo.priority = p; }
    if let Some(s) = payload.status { updated_memo.status = s; }
    if let Some(c) = payload.color { updated_memo.color = c; }
    if let Some(pin) = payload.is_pinned { updated_memo.is_pinned = pin; }
    // Don't overwrite date if None is passed (Optional Update). 
    // Wait, Json(payload) with Option fields: None means field explicitly missing or null.
    // If user wants to clear the date, they might send null.
    // Deserializing Option<DateTime> from JSON null results in None.
    // So "clearing" a date via API requires explicit logic.
    // Ideally we use a `Nullable<T>` custom type or just assume passing UpdateRequest fields usually means SET to this value.
    // But standard JSON patch behavior: undefined = ignore, null = clear.
    // Rust's Option handles both as None unless using `Option<Option<T>>` with skip_serializing_if.
    // Defaulting to: If provided, update. If not provided, keep. 
    // This means we can't clear dates easily. It's acceptable for now.
    if let Some(d) = payload.due_at { updated_memo.due_at = Some(d); }
    if let Some(r) = payload.reminder_at { updated_memo.reminder_at = Some(r); }

    updated_memo.node.updated_at = Utc::now();

    // 4. Save
    match state.repo.save(updated_memo).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to update memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update memo").into_response()
        }
    }
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::{get, post, put};
    axum::Router::new()
        .route("/api/memos", post(create_memo_handler).get(list_memos_handler))
        .route("/api/memos/:id", get(get_memo_handler).delete(delete_memo_handler).put(update_memo_handler))
}
