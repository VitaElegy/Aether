// VRKB-05: Evidence Blocks API
// Formal evidence objects attached to findings, docs, or assets

use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Evidence type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum EvidenceType {
    Screenshot,
    RequestResponse,
    LogExtract,
    PocFile,
    ExternalReference,
}

impl std::fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceType::Screenshot => write!(f, "screenshot"),
            EvidenceType::RequestResponse => write!(f, "request_response"),
            EvidenceType::LogExtract => write!(f, "log_extract"),
            EvidenceType::PocFile => write!(f, "poc_file"),
            EvidenceType::ExternalReference => write!(f, "external_reference"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub project_id: Uuid,
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub content: serde_json::Value,   // type-specific content (URL, raw text, asset ref)
    pub attached_to_type: String,     // "finding", "doc", "asset"
    pub attached_to_id: Uuid,
    pub sort_order: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateEvidenceRequest {
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub content: serde_json::Value,
    pub attached_to_type: String,
    pub attached_to_id: Uuid,
    pub sort_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct EvidenceQuery {
    pub attached_to_type: Option<String>,
    pub attached_to_id: Option<Uuid>,
}

// In-memory storage for MVP
use std::sync::RwLock;
use std::collections::HashMap;
use std::sync::OnceLock;

fn evidence_store() -> &'static RwLock<HashMap<Uuid, Vec<Evidence>>> {
    static STORE: OnceLock<RwLock<HashMap<Uuid, Vec<Evidence>>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

async fn list_evidence(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Query(query): Query<EvidenceQuery>,
) -> Result<Json<Vec<Evidence>>, StatusCode> {
    let store = evidence_store().read().unwrap();
    let items = store.get(&project_id).cloned().unwrap_or_default();

    let filtered: Vec<Evidence> = items.into_iter().filter(|e| {
        let type_match = query.attached_to_type.as_ref().is_none_or(|t| &e.attached_to_type == t);
        let id_match = query.attached_to_id.is_none_or(|id| e.attached_to_id == id);
        type_match && id_match
    }).collect();

    Ok(Json(filtered))
}

async fn create_evidence(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateEvidenceRequest>,
) -> Result<Json<Evidence>, StatusCode> {
    let now = Utc::now();
    let evidence = Evidence {
        id: Uuid::new_v4(),
        project_id,
        evidence_type: payload.evidence_type,
        title: payload.title,
        description: payload.description,
        content: payload.content,
        attached_to_type: payload.attached_to_type,
        attached_to_id: payload.attached_to_id,
        sort_order: payload.sort_order.unwrap_or(0),
        created_by: user.id,
        created_at: now,
        updated_at: now,
    };

    let mut store = evidence_store().write().unwrap();
    store.entry(project_id).or_default().push(evidence.clone());
    Ok(Json(evidence))
}

async fn delete_evidence(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path((project_id, evidence_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let mut store = evidence_store().write().unwrap();
    if let Some(items) = store.get_mut(&project_id) {
        items.retain(|e| e.id != evidence_id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_evidence(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path((project_id, evidence_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Evidence>, StatusCode> {
    let store = evidence_store().read().unwrap();
    let items = store.get(&project_id).ok_or(StatusCode::NOT_FOUND)?;
    let evidence = items.iter().find(|e| e.id == evidence_id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(evidence.clone()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/evidence", get(list_evidence).post(create_evidence))
        .route("/api/vrkb/projects/:id/evidence/:eid", get(get_evidence).delete(delete_evidence))
}
