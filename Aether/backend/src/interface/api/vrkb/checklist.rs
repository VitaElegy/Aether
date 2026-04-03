// VRKB-04: Checklist System API
// Section/methodology checklists with completion tracking

use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub is_completed: bool,
    pub is_blocker: bool,
    pub completed_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistSummary {
    pub section_id: Uuid,
    pub total: usize,
    pub completed: usize,
    pub blockers: usize,
    pub completion_percent: f64,
}

#[derive(Deserialize)]
pub struct CreateChecklistItemRequest {
    pub title: String,
    pub description: Option<String>,
    pub is_blocker: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateChecklistItemRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_completed: Option<bool>,
    pub is_blocker: Option<bool>,
    pub sort_order: Option<i32>,
}

// In-memory storage for MVP (would be DB in production)
use std::sync::RwLock;
use std::collections::HashMap;

use std::sync::OnceLock;

fn checklist_store() -> &'static RwLock<HashMap<Uuid, Vec<ChecklistItem>>> {
    static STORE: OnceLock<RwLock<HashMap<Uuid, Vec<ChecklistItem>>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

async fn list_checklist_items(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(section_id): Path<Uuid>,
) -> Result<Json<Vec<ChecklistItem>>, StatusCode> {
    let store = checklist_store().read().unwrap();
    let items = store.get(&section_id).cloned().unwrap_or_default();
    Ok(Json(items))
}

async fn create_checklist_item(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(section_id): Path<Uuid>,
    Json(payload): Json<CreateChecklistItemRequest>,
) -> Result<Json<ChecklistItem>, StatusCode> {
    let now = Utc::now();
    let item = ChecklistItem {
        id: Uuid::new_v4(),
        section_id,
        title: payload.title,
        description: payload.description,
        is_completed: false,
        is_blocker: payload.is_blocker.unwrap_or(false),
        completed_by: None,
        completed_at: None,
        sort_order: payload.sort_order.unwrap_or(0),
        created_at: now,
        updated_at: now,
    };
    let mut store = checklist_store().write().unwrap();
    store.entry(section_id).or_insert_with(Vec::new).push(item.clone());
    Ok(Json(item))
}

async fn update_checklist_item(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Path((section_id, item_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateChecklistItemRequest>,
) -> Result<Json<ChecklistItem>, StatusCode> {
    let mut store = checklist_store().write().unwrap();
    let items = store.entry(section_id).or_insert_with(Vec::new);
    let item = items.iter_mut().find(|i| i.id == item_id).ok_or(StatusCode::NOT_FOUND)?;

    if let Some(title) = payload.title { item.title = title; }
    if let Some(desc) = payload.description { item.description = Some(desc); }
    if let Some(blocker) = payload.is_blocker { item.is_blocker = blocker; }
    if let Some(order) = payload.sort_order { item.sort_order = order; }
    if let Some(completed) = payload.is_completed {
        item.is_completed = completed;
        if completed {
            item.completed_by = Some(user.id);
            item.completed_at = Some(Utc::now());
        } else {
            item.completed_by = None;
            item.completed_at = None;
        }
    }
    item.updated_at = Utc::now();

    Ok(Json(item.clone()))
}

async fn get_checklist_summary(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(section_id): Path<Uuid>,
) -> Result<Json<ChecklistSummary>, StatusCode> {
    let store = checklist_store().read().unwrap();
    let items = store.get(&section_id).cloned().unwrap_or_default();
    let total = items.len();
    let completed = items.iter().filter(|i| i.is_completed).count();
    let blockers = items.iter().filter(|i| i.is_blocker && !i.is_completed).count();
    let completion_percent = if total > 0 { (completed as f64 / total as f64) * 100.0 } else { 0.0 };

    Ok(Json(ChecklistSummary {
        section_id,
        total,
        completed,
        blockers,
        completion_percent,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/sections/:id/checklist", get(list_checklist_items).post(create_checklist_item))
        .route("/api/vrkb/sections/:id/checklist/:item_id", put(update_checklist_item))
        .route("/api/vrkb/sections/:id/checklist/summary", get(get_checklist_summary))
}
