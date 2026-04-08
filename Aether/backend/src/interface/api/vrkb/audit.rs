use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Path, Query},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- VRKB-09: Audit Log Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub event_type: String,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct AuditQueryParams {
    limit: Option<i64>,
    offset: Option<i64>,
    event_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateAuditLogRequest {
    event_type: String,
    actor_id: Option<Uuid>,
    actor_name: Option<String>,
    target_type: String,
    target_id: Option<Uuid>,
    details: Option<serde_json::Value>,
}

// --- VRKB-09: Notification Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbNotification {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String, // "assignment" | "due_soon" | "reopened"
    pub title: String,
    pub message: String,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct AuditLogListResponse {
    items: Vec<AuditLogEntry>,
    total: usize,
}

// --- In-memory audit store (MVP) ---
// In production, this should use the database via VrkbRepository
use std::sync::RwLock;
static AUDIT_STORE: std::sync::LazyLock<RwLock<Vec<AuditLogEntry>>> =
    std::sync::LazyLock::new(|| RwLock::new(Vec::new()));
static NOTIFICATION_STORE: std::sync::LazyLock<RwLock<Vec<VrkbNotification>>> =
    std::sync::LazyLock::new(|| RwLock::new(Vec::new()));

// --- Event Types ---
// finding_created, finding_status_changed, evidence_added,
// doc_updated, member_added, member_removed, report_generated,
// asset_linked, asset_unlinked

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/vrkb/projects/:id/audit",
            get(list_audit_logs).post(create_audit_log),
        )
        .route(
            "/api/vrkb/projects/:id/notifications",
            get(list_notifications),
        )
        .route(
            "/api/vrkb/notifications/:id/read",
            post(mark_notification_read),
        )
}

async fn list_audit_logs(
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Query(params): Query<AuditQueryParams>,
) -> Result<Json<AuditLogListResponse>, (StatusCode, String)> {
    let store = AUDIT_STORE.read().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let mut filtered: Vec<AuditLogEntry> = store
        .iter()
        .filter(|e| e.project_id == project_id)
        .filter(|e| {
            if let Some(ref et) = params.event_type {
                e.event_type == *et
            } else {
                true
            }
        })
        .cloned()
        .collect();

    // Sort by most recent first
    filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = filtered.len();
    let offset = params.offset.unwrap_or(0) as usize;
    let limit = params.limit.unwrap_or(50) as usize;

    let items: Vec<AuditLogEntry> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok(Json(AuditLogListResponse { items, total }))
}

async fn create_audit_log(
    user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateAuditLogRequest>,
) -> Result<Json<AuditLogEntry>, (StatusCode, String)> {
    let entry = AuditLogEntry {
        id: Uuid::new_v4(),
        project_id,
        event_type: payload.event_type.clone(),
        actor_id: Some(payload.actor_id.unwrap_or(user.id)),
        actor_name: payload.actor_name,
        target_type: payload.target_type,
        target_id: payload.target_id,
        details: payload.details.unwrap_or(serde_json::json!({})),
        created_at: Utc::now(),
    };

    let mut store = AUDIT_STORE.write().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;
    store.push(entry.clone());

    // Auto-generate notifications for certain event types
    if matches!(
        payload.event_type.as_str(),
        "finding_created" | "member_added" | "finding_status_changed"
    ) {
        let notification = VrkbNotification {
            id: Uuid::new_v4(),
            project_id,
            user_id: payload.actor_id.unwrap_or(Uuid::nil()),
            notification_type: match payload.event_type.as_str() {
                "finding_created" => "assignment".to_string(),
                "finding_status_changed" => "reopened".to_string(),
                _ => "assignment".to_string(),
            },
            title: format!("New Event: {}", payload.event_type),
            message: format!("A {} event occurred in the project.", payload.event_type),
            read: false,
            created_at: Utc::now(),
        };

        if let Ok(mut notif_store) = NOTIFICATION_STORE.write() {
            notif_store.push(notification);
        }
    }

    Ok(Json(entry))
}

async fn list_notifications(
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbNotification>>, (StatusCode, String)> {
    let store = NOTIFICATION_STORE.read().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let notifications: Vec<VrkbNotification> = store
        .iter()
        .filter(|n| n.project_id == project_id)
        .cloned()
        .collect();

    Ok(Json(notifications))
}

async fn mark_notification_read(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut store = NOTIFICATION_STORE.write().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    if let Some(notif) = store.iter_mut().find(|n| n.id == id) {
        notif.read = true;
        Ok(StatusCode::OK)
    } else {
        Err((StatusCode::NOT_FOUND, "Notification not found".to_string()))
    }
}
