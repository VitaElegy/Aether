use axum::{
    Json, extract::{State, Path, Query}, http::StatusCode, routing::{get, post, patch}, Router
};
use uuid::Uuid;
use chrono::Utc;
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::models::{VrkbFinding};
use crate::domain::ports::{VrkbRepository};
use std::sync::Arc;
use serde::{Deserialize};

#[derive(Deserialize)]
pub struct CreateFindingRequest {
    title: String,
    severity: String,
    status: Option<String>,
    content: Option<serde_json::Value>,
    is_triage: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateStatusRequest {
    status: String,
}

#[derive(Deserialize)]
pub struct ListFindingsQuery {
    section_id: Option<Uuid>,
    project_id: Option<Uuid>,
}

// --- Handlers ---

async fn create_finding(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(section_id): Path<Uuid>,
    Json(payload): Json<CreateFindingRequest>,
) -> Result<Json<VrkbFinding>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    
    let new_finding = VrkbFinding {
        id: Uuid::new_v4(),
        section_id,
        title: payload.title,
        severity: payload.severity,
        status: payload.status.unwrap_or("Triage".to_string()),
        content: payload.content,
        is_triage: payload.is_triage.unwrap_or(false),
        author_id: Some(user.id),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match repo.create_finding(new_finding.clone()).await {
        Ok(_) => Ok(Json(new_finding)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_findings(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<ListFindingsQuery>,
) -> Result<Json<Vec<VrkbFinding>>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.list_findings(query.section_id, query.project_id).await {
        Ok(findings) => Ok(Json(findings)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_finding(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VrkbFinding>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.get_finding(&id).await {
        Ok(Some(finding)) => Ok(Json(finding)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_finding_status(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<StatusCode, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.update_finding_status(&id, payload.status).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        // Canonical Routes
        // Create finding explicitly under a section
        .route("/api/vrkb/sections/:id/findings", post(create_finding))
        // List/Get findings globally (with query params)
        .route("/api/vrkb/findings", get(list_findings))
        .route("/api/vrkb/findings/:id", get(get_finding)) // missing PUT update_finding implementation! 
        // We will add basic update support or map it to status for now to avoid compilation errors if handler doesn't exist.
        // Actually, let's just stick to what exists for now, but fix prefix.
        .route("/api/vrkb/findings/:id/status", patch(update_finding_status))
}
