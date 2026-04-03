use crate::domain::models::{LinkedEntity, Memo, Node, NodeType, PermissionMode, SavedView, SavedViewFilters, UserId};
use crate::domain::ports::{MemoBulkUpdate, MemoRepository};
use crate::interface::api::auth::{AuthenticatedUser, MaybeAuthenticatedUser};
use crate::interface::state::AppState;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use uuid::Uuid;

// ──────────────────────────────────────────────
// Request / Response DTOs
// ──────────────────────────────────────────────

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
    // MEMO-01
    pub channel: Option<String>,
    // MEMO-05
    pub linked_entities: Option<Vec<LinkedEntity>>,
    // MEMO-06
    pub scheduled_at: Option<DateTime<Utc>>,
    pub snoozed_until: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListMemosRequest {
    pub author_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    // MEMO-06 review queue
    pub queue: Option<String>, // "due_today", "overdue", "stale"
    pub stale_days: Option<i64>,
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
    // MEMO-01
    pub channel: Option<String>,
    // MEMO-05
    pub linked_entities: Option<Vec<LinkedEntity>>,
    // MEMO-06
    pub scheduled_at: Option<DateTime<Utc>>,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

// MEMO-04: Bulk operations
#[derive(serde::Deserialize, Debug)]
pub struct BulkUpdateRequest {
    pub ids: Vec<Uuid>,
    pub update: MemoBulkUpdate,
}

#[derive(serde::Deserialize, Debug)]
pub struct BulkDeleteRequest {
    pub ids: Vec<Uuid>,
}

#[derive(serde::Deserialize, Debug)]
pub struct MergeMemoRequest {
    pub source_ids: Vec<Uuid>,   // Memos to merge
    pub target_id: Option<Uuid>, // Merge into existing; if None, create new
    pub title: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct SplitMemoRequest {
    pub split_at: usize, // Character offset to split content
    pub new_title: Option<String>,
}

// MEMO-03: Saved Views
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SavedViewRequest {
    pub name: String,
    pub icon: Option<String>,
    pub filters: SavedViewFilters,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub view_mode: Option<String>,
    pub pinned: Option<bool>,
    pub position: Option<i32>,
}

// MEMO-07: Import/Export
#[derive(serde::Deserialize, Debug)]
pub struct ExportRequest {
    pub format: String, // "markdown", "json", "daily_archive"
    pub ids: Option<Vec<Uuid>>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ImportRequest {
    pub memos: Vec<ImportMemoItem>,
    pub merge_tags: Option<bool>,
    pub merge_channels: Option<bool>,
    pub detect_duplicates: Option<bool>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ImportMemoItem {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

// ──────────────────────────────────────────────
// Handlers — CRUD (existing, enhanced)
// ──────────────────────────────────────────────

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
            knowledge_base_id: None,
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
        channel: payload.channel,
        excerpt: None,
        linked_entities: payload.linked_entities.unwrap_or_default(),
        scheduled_at: payload.scheduled_at,
        snoozed_until: payload.snoozed_until,
        reviewed_at: None,
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
    _auth: AuthenticatedUser,
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

    let aid = target_author_id.unwrap();

    // MEMO-06: Review queue routing
    if let Some(ref queue) = params.queue {
        let result = match queue.as_str() {
            "due_today" => state.repo.find_due_today(aid).await,
            "overdue" => state.repo.find_overdue(aid).await,
            "stale" => {
                let days = params.stale_days.unwrap_or(7);
                state.repo.find_stale(aid, days).await
            }
            _ => state.repo.list(viewer_id, Some(aid)).await,
        };
        return match result {
            Ok(memos) => Json::<Vec<Memo>>(memos).into_response(),
            Err(e) => {
                tracing::error!("Failed to list memos (queue={}): {:?}", queue, e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list memos").into_response()
            }
        };
    }

    match (params.start_date, params.end_date) {
        (Some(start), Some(end)) => {
            match state.repo.find_by_date_range(aid, start, end).await {
                Ok(memos) => Json::<Vec<Memo>>(memos).into_response(),
                Err(e) => {
                    tracing::error!("Failed to list memos by date: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list memos").into_response()
                }
            }
        }
        _ => match state.repo.list(viewer_id, Some(aid)).await {
            Ok(memos) => Json::<Vec<Memo>>(memos).into_response(),
            Err(e) => {
                tracing::error!("Failed to list memos: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list memos").into_response()
            }
        },
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
                return (StatusCode::FORBIDDEN, "Not authorized to delete this memo")
                    .into_response();
            }
        }
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

pub async fn update_memo_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMemoRequest>,
) -> impl IntoResponse {
    let existing_memo = match state.repo.find_by_id(&id).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if existing_memo.node.author_id != user.id {
        return (StatusCode::FORBIDDEN, "Not authorized to update this memo").into_response();
    }

    let mut updated_memo = existing_memo;
    if let Some(t) = payload.title {
        updated_memo.node.title = t;
    }
    if let Some(c) = payload.content {
        updated_memo.content = c;
    }
    if let Some(tags) = payload.tags {
        updated_memo.tags = tags;
    }
    if let Some(vis) = payload.visibility {
        updated_memo.node.permission_mode = match vis.to_lowercase().as_str() {
            "private" => PermissionMode::Private,
            "internal" => PermissionMode::Internal,
            _ => PermissionMode::Public,
        };
    }
    if let Some(p) = payload.priority {
        updated_memo.priority = p;
    }
    if let Some(s) = payload.status {
        updated_memo.status = s;
    }
    if let Some(c) = payload.color {
        updated_memo.color = c;
    }
    if let Some(pin) = payload.is_pinned {
        updated_memo.is_pinned = pin;
    }
    if let Some(d) = payload.due_at {
        updated_memo.due_at = Some(d);
    }
    if let Some(r) = payload.reminder_at {
        updated_memo.reminder_at = Some(r);
    }
    // MEMO-01
    if let Some(ch) = payload.channel {
        updated_memo.channel = Some(ch);
    }
    // MEMO-05
    if let Some(le) = payload.linked_entities {
        updated_memo.linked_entities = le;
    }
    // MEMO-06
    if let Some(sa) = payload.scheduled_at {
        updated_memo.scheduled_at = Some(sa);
    }
    if let Some(su) = payload.snoozed_until {
        updated_memo.snoozed_until = Some(su);
    }
    if let Some(ra) = payload.reviewed_at {
        updated_memo.reviewed_at = Some(ra);
    }

    updated_memo.node.updated_at = Utc::now();

    match state.repo.save(updated_memo).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to update memo: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update memo").into_response()
        }
    }
}

// ──────────────────────────────────────────────
// MEMO-01: Quick Actions (archive, pin, snooze, convert)
// ──────────────────────────────────────────────

#[derive(serde::Deserialize, Debug)]
pub struct QuickActionRequest {
    pub action: String, // "archive", "pin", "unpin", "snooze", "convert_task", "convert_note"
    pub snooze_until: Option<DateTime<Utc>>,
}

pub async fn quick_action_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<QuickActionRequest>,
) -> impl IntoResponse {
    let existing = match state.repo.find_by_id(&id).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if existing.node.author_id != user.id {
        return (StatusCode::FORBIDDEN, "Not authorized").into_response();
    }

    let mut memo = existing;
    match payload.action.as_str() {
        "archive" => memo.status = "Archived".to_string(),
        "pin" => memo.is_pinned = true,
        "unpin" => memo.is_pinned = false,
        "snooze" => {
            memo.snoozed_until = payload
                .snooze_until
                .or(Some(Utc::now() + chrono::Duration::hours(4)));
        }
        "convert_task" => {
            memo.status = "Todo".to_string();
            if memo.priority == "P2" || memo.priority.is_empty() {
                memo.priority = "P1".to_string();
            }
        }
        "convert_note" => {
            memo.status = "Done".to_string();
            memo.priority = "P3".to_string();
        }
        _ => return (StatusCode::BAD_REQUEST, "Unknown action").into_response(),
    }
    memo.node.updated_at = Utc::now();

    match state.repo.save(memo).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Quick action failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response()
        }
    }
}

// ──────────────────────────────────────────────
// MEMO-04: Bulk Operations
// ──────────────────────────────────────────────

pub async fn bulk_update_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<BulkUpdateRequest>,
) -> impl IntoResponse {
    match state.repo.bulk_update(payload.ids, payload.update).await {
        Ok(count) => Json(serde_json::json!({ "updated": count })).into_response(),
        Err(e) => {
            tracing::error!("Bulk update failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Bulk update failed").into_response()
        }
    }
}

pub async fn bulk_delete_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<BulkDeleteRequest>,
) -> impl IntoResponse {
    match state.repo.bulk_delete(payload.ids).await {
        Ok(count) => Json(serde_json::json!({ "deleted": count })).into_response(),
        Err(e) => {
            tracing::error!("Bulk delete failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Bulk delete failed").into_response()
        }
    }
}

pub async fn merge_memos_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<MergeMemoRequest>,
) -> impl IntoResponse {
    if payload.source_ids.len() < 2 {
        return (StatusCode::BAD_REQUEST, "Need at least 2 memos to merge").into_response();
    }

    // Fetch all source memos
    let mut source_memos = Vec::new();
    for sid in &payload.source_ids {
        match state.repo.find_by_id(sid).await {
            Ok(Some(m)) => {
                if m.node.author_id != user.id {
                    return (StatusCode::FORBIDDEN, "Not authorized").into_response();
                }
                source_memos.push(m);
            }
            _ => return (StatusCode::NOT_FOUND, "Source memo not found").into_response(),
        }
    }

    // Merge content
    let merged_content = source_memos
        .iter()
        .map(|m| {
            format!(
                "## {}\n\n{}",
                if m.node.title.is_empty() {
                    "Untitled"
                } else {
                    &m.node.title
                },
                &m.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    // Merge tags
    let mut merged_tags: Vec<String> = Vec::new();
    for m in &source_memos {
        for t in &m.tags {
            if !merged_tags.contains(t) {
                merged_tags.push(t.clone());
            }
        }
    }

    let title = payload
        .title
        .unwrap_or_else(|| source_memos[0].node.title.clone());

    let new_id = Uuid::new_v4();
    let merged = Memo {
        node: Node {
            id: new_id,
            parent_id: None,
            author_id: user.id,
            knowledge_base_id: None,
            r#type: NodeType::Memo,
            title,
            permission_mode: PermissionMode::Private,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        content: merged_content,
        priority: source_memos[0].priority.clone(),
        status: "Todo".to_string(),
        color: source_memos[0].color.clone(),
        is_pinned: false,
        due_at: source_memos.iter().filter_map(|m| m.due_at).min(),
        reminder_at: None,
        tags: merged_tags,
        channel: source_memos[0].channel.clone(),
        excerpt: None,
        linked_entities: vec![],
        scheduled_at: None,
        snoozed_until: None,
        reviewed_at: None,
    };

    match state.repo.save(merged).await {
        Ok(id) => {
            // Archive source memos
            for sid in &payload.source_ids {
                let _ = state
                    .repo
                    .bulk_update(
                        vec![*sid],
                        MemoBulkUpdate {
                            status: Some("Archived".to_string()),
                            ..Default::default()
                        },
                    )
                    .await;
            }
            (StatusCode::CREATED, Json(id)).into_response()
        }
        Err(e) => {
            tracing::error!("Merge failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Merge failed").into_response()
        }
    }
}

pub async fn split_memo_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SplitMemoRequest>,
) -> impl IntoResponse {
    let existing = match state.repo.find_by_id(&id).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Memo not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if existing.node.author_id != user.id {
        return (StatusCode::FORBIDDEN, "Not authorized").into_response();
    }

    let content = &existing.content;
    if payload.split_at >= content.len() {
        return (StatusCode::BAD_REQUEST, "Split position out of range").into_response();
    }

    let (first_part, second_part) = content.split_at(payload.split_at);

    // Update original with first part
    let mut original = existing.clone();
    original.content = first_part.to_string();
    original.node.updated_at = Utc::now();

    // Create new memo with second part
    let new_id = Uuid::new_v4();
    let new_memo = Memo {
        node: Node {
            id: new_id,
            parent_id: None,
            author_id: user.id,
            knowledge_base_id: existing.node.knowledge_base_id,
            r#type: NodeType::Memo,
            title: payload
                .new_title
                .unwrap_or_else(|| format!("{} (split)", existing.node.title)),
            permission_mode: existing.node.permission_mode.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        content: second_part.to_string(),
        priority: existing.priority.clone(),
        status: existing.status.clone(),
        color: existing.color.clone(),
        is_pinned: false,
        due_at: None,
        reminder_at: None,
        tags: existing.tags.clone(),
        channel: existing.channel.clone(),
        excerpt: None,
        linked_entities: vec![],
        scheduled_at: None,
        snoozed_until: None,
        reviewed_at: None,
    };

    let _ = state.repo.save(original).await;
    match state.repo.save(new_memo).await {
        Ok(nid) => Json(serde_json::json!({ "original_id": id, "new_id": nid })).into_response(),
        Err(e) => {
            tracing::error!("Split failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Split failed").into_response()
        }
    }
}

// ──────────────────────────────────────────────
// MEMO-05: Backlinks
// ──────────────────────────────────────────────

pub async fn get_backlinks_handler(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.find_backlinks(&id).await {
        Ok(memos) => Json(memos).into_response(),
        Err(e) => {
            tracing::error!("Backlinks query failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response()
        }
    }
}

// ──────────────────────────────────────────────
// MEMO-03: Saved Views (stored in user.experience.memo_saved_views)
// ──────────────────────────────────────────────

pub async fn list_saved_views_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let user_model =
        match crate::infrastructure::persistence::entities::user::Entity::find_by_id(user.id)
            .one(&state.repo.db)
            .await
        {
            Ok(Some(u)) => u,
            _ => return Json::<Vec<SavedView>>(vec![]).into_response(),
        };

    if let Some(exp) = user_model.experience {
        if let Some(views) = exp.get("memo_saved_views") {
            if let Ok(v) = serde_json::from_value::<Vec<SavedView>>(views.clone()) {
                return Json(v).into_response();
            }
        }
    }
    Json::<Vec<SavedView>>(vec![]).into_response()
}

pub async fn save_view_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<SavedViewRequest>,
) -> impl IntoResponse {
    use crate::infrastructure::persistence::entities::user;
    use sea_orm::*;

    let user_model = match user::Entity::find_by_id(user.id).one(&state.repo.db).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response(),
    };

    let mut exp = user_model
        .experience
        .clone()
        .unwrap_or(serde_json::json!({}));
    if !exp.is_object() {
        exp = serde_json::json!({});
    }

    let mut views: Vec<SavedView> = exp
        .get("memo_saved_views")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let new_view = SavedView {
        id: Uuid::new_v4(),
        name: payload.name,
        icon: payload.icon,
        filters: payload.filters,
        sort_by: payload.sort_by,
        sort_dir: payload.sort_dir,
        view_mode: payload.view_mode,
        pinned: payload.pinned.unwrap_or(false),
        position: payload.position.unwrap_or(views.len() as i32),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let view_id = new_view.id;
    views.push(new_view);

    exp["memo_saved_views"] = serde_json::to_value(&views).unwrap();

    let mut user_active: user::ActiveModel = user_model.into();
    user_active.experience = Set(Some(exp));

    match user_active.update(&state.repo.db).await {
        Ok(_) => (StatusCode::CREATED, Json(view_id)).into_response(),
        Err(e) => {
            tracing::error!("Failed to save view: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response()
        }
    }
}

pub async fn delete_saved_view_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(view_id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::infrastructure::persistence::entities::user;
    use sea_orm::*;

    let user_model = match user::Entity::find_by_id(user.id).one(&state.repo.db).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response(),
    };

    let mut exp = user_model
        .experience
        .clone()
        .unwrap_or(serde_json::json!({}));

    let mut views: Vec<SavedView> = exp
        .get("memo_saved_views")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    views.retain(|v| v.id != view_id);
    exp["memo_saved_views"] = serde_json::to_value(&views).unwrap();

    let mut user_active: user::ActiveModel = user_model.into();
    user_active.experience = Set(Some(exp));

    match user_active.update(&state.repo.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete view: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response()
        }
    }
}

// ──────────────────────────────────────────────
// MEMO-07: Export / Import
// ──────────────────────────────────────────────

#[derive(serde::Serialize)]
struct ExportBundle {
    version: String,
    exported_at: DateTime<Utc>,
    memos: Vec<Memo>,
}

pub async fn export_memos_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ExportRequest>,
) -> impl IntoResponse {
    let aid = UserId(user.id);

    // Fetch memos
    let all_memos = if let Some(ref ids) = payload.ids {
        let mut result = Vec::new();
        for id in ids {
            if let Ok(Some(m)) = state.repo.find_by_id(id).await {
                if m.node.author_id == user.id {
                    result.push(m);
                }
            }
        }
        result
    } else if let (Some(from), Some(to)) = (payload.date_from, payload.date_to) {
        state
            .repo
            .find_by_date_range(aid.clone(), from, to)
            .await
            .unwrap_or_default()
    } else {
        state
            .repo
            .list(Some(aid.clone()), Some(aid))
            .await
            .unwrap_or_default()
    };

    match payload.format.as_str() {
        "json" => {
            let bundle = ExportBundle {
                version: "1.0".to_string(),
                exported_at: Utc::now(),
                memos: all_memos,
            };
            Json(bundle).into_response()
        }
        "markdown" => {
            let mut md = String::from("# Memos Export\n\n");
            for m in &all_memos {
                md.push_str(&format!("## {}\n\n", m.node.title));
                md.push_str(&format!(
                    "- **Status**: {} | **Priority**: {} | **Tags**: {}\n",
                    m.status,
                    m.priority,
                    m.tags.join(", ")
                ));
                if let Some(ref ch) = m.channel {
                    md.push_str(&format!("- **Channel**: {}\n", ch));
                }
                md.push_str(&format!("- **Created**: {}\n\n", m.node.created_at));
                md.push_str(&m.content);
                md.push_str("\n\n---\n\n");
            }
            (
                StatusCode::OK,
                [(
                    "Content-Type",
                    "text/markdown; charset=utf-8",
                )],
                md,
            )
                .into_response()
        }
        "daily_archive" => {
            // Group by date
            let mut by_date: std::collections::BTreeMap<String, Vec<&Memo>> =
                std::collections::BTreeMap::new();
            for m in &all_memos {
                let date_key = m.node.created_at.format("%Y-%m-%d").to_string();
                by_date.entry(date_key).or_default().push(m);
            }

            let mut md = String::from("# Daily Archive\n\n");
            for (date, memos) in &by_date {
                md.push_str(&format!("## {}\n\n", date));
                for m in memos {
                    md.push_str(&format!("### {}\n\n{}\n\n", m.node.title, m.content));
                }
                md.push_str("---\n\n");
            }
            (
                StatusCode::OK,
                [("Content-Type", "text/markdown; charset=utf-8")],
                md,
            )
                .into_response()
        }
        _ => (StatusCode::BAD_REQUEST, "Unknown format").into_response(),
    }
}

pub async fn import_memos_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ImportRequest>,
) -> impl IntoResponse {
    let detect_dupes = payload.detect_duplicates.unwrap_or(true);

    // If duplicate detection enabled, fetch existing titles for comparison
    let existing_titles: std::collections::HashSet<String> = if detect_dupes {
        let existing = state
            .repo
            .list(Some(UserId(user.id)), Some(UserId(user.id)))
            .await
            .unwrap_or_default();
        existing.iter().map(|m| m.node.title.clone()).collect()
    } else {
        std::collections::HashSet::new()
    };

    let mut imported = 0usize;
    let mut skipped = 0usize;

    for item in &payload.memos {
        if detect_dupes && existing_titles.contains(&item.title) {
            skipped += 1;
            continue;
        }

        let id = Uuid::new_v4();
        let memo = Memo {
            node: Node {
                id,
                parent_id: None,
                author_id: user.id,
                knowledge_base_id: None,
                r#type: NodeType::Memo,
                title: item.title.clone(),
                permission_mode: PermissionMode::Private,
                created_at: item.created_at.unwrap_or(Utc::now()),
                updated_at: Utc::now(),
            },
            content: item.content.clone(),
            priority: item.priority.clone().unwrap_or("P2".to_string()),
            status: item.status.clone().unwrap_or("Todo".to_string()),
            color: "Yellow".to_string(),
            is_pinned: false,
            due_at: item.due_at,
            reminder_at: None,
            tags: item.tags.clone(),
            channel: item.channel.clone(),
            excerpt: None,
            linked_entities: vec![],
            scheduled_at: None,
            snoozed_until: None,
            reviewed_at: None,
        };

        if state.repo.save(memo).await.is_ok() {
            imported += 1;
        }
    }

    Json(serde_json::json!({
        "imported": imported,
        "skipped": skipped,
        "total": payload.memos.len()
    }))
    .into_response()
}

// ──────────────────────────────────────────────
// Workflow (existing)
// ──────────────────────────────────────────────

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WorkflowConfig {
    pub columns: Vec<String>,
}

pub async fn get_workflow_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let user_model =
        match crate::infrastructure::persistence::entities::user::Entity::find_by_id(user.id)
            .one(&state.repo.db)
            .await
        {
            Ok(Some(u)) => u,
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response(),
        };

    let default_columns = vec!["Todo".to_string(), "Doing".to_string(), "Done".to_string()];

    if let Some(exp) = user_model.experience {
        if let Some(workflow) = exp.get("memo_workflow") {
            if let Ok(cols) = serde_json::from_value::<Vec<String>>(workflow.clone()) {
                return Json(WorkflowConfig { columns: cols }).into_response();
            }
        }
    }

    Json(WorkflowConfig {
        columns: default_columns,
    })
    .into_response()
}

pub async fn update_workflow_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<WorkflowConfig>,
) -> impl IntoResponse {
    use crate::infrastructure::persistence::entities::user;
    use sea_orm::*;

    let user_model = match user::Entity::find_by_id(user.id).one(&state.repo.db).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response(),
    };

    let mut user_active: user::ActiveModel = user_model.into();

    let mut exp = user_active
        .experience
        .clone()
        .unwrap()
        .unwrap_or(serde_json::json!({}));
    if !exp.is_object() {
        exp = serde_json::json!({});
    }

    exp["memo_workflow"] = serde_json::to_value(payload.columns).unwrap();
    user_active.experience = Set(Some(exp));

    match user_active.update(&state.repo.db).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to save workflow: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save workflow").into_response()
        }
    }
}

// ──────────────────────────────────────────────
// Router
// ──────────────────────────────────────────────

pub fn router() -> axum::Router<AppState> {
    use axum::routing::{delete, get, post, put};
    axum::Router::new()
        .route(
            "/api/memos",
            post(create_memo_handler).get(list_memos_handler),
        )
        .route(
            "/api/memos/workflow",
            get(get_workflow_handler).put(update_workflow_handler),
        )
        // MEMO-04: Bulk operations
        .route("/api/memos/bulk-update", post(bulk_update_handler))
        .route("/api/memos/bulk-delete", post(bulk_delete_handler))
        .route("/api/memos/merge", post(merge_memos_handler))
        // MEMO-07: Import/Export
        .route("/api/memos/export", post(export_memos_handler))
        .route("/api/memos/import", post(import_memos_handler))
        // MEMO-03: Saved Views
        .route(
            "/api/memos/views",
            get(list_saved_views_handler).post(save_view_handler),
        )
        .route(
            "/api/memos/views/:view_id",
            delete(delete_saved_view_handler),
        )
        // Single memo CRUD + actions
        .route(
            "/api/memos/:id",
            get(get_memo_handler)
                .delete(delete_memo_handler)
                .put(update_memo_handler),
        )
        // MEMO-01: Quick actions
        .route("/api/memos/:id/action", post(quick_action_handler))
        // MEMO-04: Split
        .route("/api/memos/:id/split", post(split_memo_handler))
        // MEMO-05: Backlinks
        .route("/api/memos/:id/backlinks", get(get_backlinks_handler))
}
