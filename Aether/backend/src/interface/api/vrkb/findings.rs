use crate::domain::models::VrkbFinding;
use crate::domain::ports::VrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateFindingRequest {
    title: String,
    severity: String,
    status: Option<String>,
    content: Option<serde_json::Value>,
    is_triage: Option<bool>,
    // VRKB-02 extended fields
    confidence: Option<String>,
    owner_id: Option<Uuid>,
    due_date: Option<String>, // ISO8601 string
    affected_assets: Option<serde_json::Value>,
    repro_steps: Option<String>,
    remediation: Option<String>,
    verification_note: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateFindingRequest {
    title: Option<String>,
    severity: Option<String>,
    status: Option<String>,
    content: Option<serde_json::Value>,
    is_triage: Option<bool>,
    // VRKB-02 extended fields
    confidence: Option<String>,
    owner_id: Option<Uuid>,
    due_date: Option<String>, // ISO8601 string, null to clear
    affected_assets: Option<serde_json::Value>,
    repro_steps: Option<String>,
    remediation: Option<String>,
    verification_note: Option<String>,
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
) -> Result<Json<VrkbFinding>, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    // Validate severity
    let severity_lower = payload.severity.to_lowercase();
    if !VALID_SEVERITIES.contains(&severity_lower.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid severity '{}'. Valid: {:?}", payload.severity, VALID_SEVERITIES),
        ));
    }

    // Validate status if provided
    let status = payload.status.as_deref().unwrap_or("triage").to_lowercase();
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid status '{}'. Valid: {:?}", status, VALID_STATUSES),
        ));
    }

    // Validate confidence if provided
    if let Some(ref conf) = payload.confidence {
        let conf_lower = conf.to_lowercase();
        if !VALID_CONFIDENCES.contains(&conf_lower.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid confidence '{}'. Valid: {:?}", conf, VALID_CONFIDENCES),
            ));
        }
    }

    let due_date = payload.due_date.and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc))
    });

    let new_finding = VrkbFinding {
        id: Uuid::new_v4(),
        section_id,
        title: payload.title,
        severity: severity_lower,
        status,
        content: payload.content,
        is_triage: payload.is_triage.unwrap_or(true),
        author_id: Some(user.id),
        confidence: payload.confidence,
        owner_id: payload.owner_id,
        due_date,
        affected_assets: payload.affected_assets,
        repro_steps: payload.repro_steps,
        remediation: payload.remediation,
        verification_note: payload.verification_note,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match repo.create_finding(new_finding.clone()).await {
        Ok(_) => Ok(Json(new_finding)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
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

async fn update_finding(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFindingRequest>,
) -> Result<Json<VrkbFinding>, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    // Block status changes through the general update endpoint
    // Status must be changed via PATCH /findings/:id/status to enforce state machine
    if payload.status.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Status cannot be changed via PUT. Use PATCH /api/vrkb/findings/:id/status instead.".to_string(),
        ));
    }

    // Validate severity if provided
    if let Some(ref severity) = payload.severity {
        let severity_lower = severity.to_lowercase();
        if !VALID_SEVERITIES.contains(&severity_lower.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid severity '{}'. Valid: {:?}", severity, VALID_SEVERITIES),
            ));
        }
    }

    // Validate confidence if provided
    if let Some(ref confidence) = payload.confidence {
        let confidence_lower = confidence.to_lowercase();
        if !VALID_CONFIDENCES.contains(&confidence_lower.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid confidence '{}'. Valid: {:?}", confidence, VALID_CONFIDENCES),
            ));
        }
    }

    let due_date_parsed = payload.due_date.map(|d| {
        chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc))
    });

    match repo
        .update_finding(
            &id,
            payload.title,
            payload.severity,
            None, // status is always None — blocked above
            payload.content.map(Some),
            payload.is_triage,
            payload.confidence.map(Some),
            payload.owner_id.map(Some),
            due_date_parsed,
            payload.affected_assets.map(Some),
            payload.repro_steps.map(Some),
            payload.remediation.map(Some),
            payload.verification_note.map(Some),
        )
        .await
    {
        Ok(_) => {
            // Return the updated finding
            match repo.get_finding(&id).await {
                Ok(Some(finding)) => Ok(Json(finding)),
                _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve updated finding".to_string())),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn delete_finding(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.delete_finding(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Valid states for the 7-state lifecycle
const VALID_STATUSES: &[&str] = &["triage", "confirmed", "exploiting", "fixing", "verifying", "closed", "risk_accepted"];
const VALID_SEVERITIES: &[&str] = &["low", "medium", "high", "critical", "info"];
const VALID_CONFIDENCES: &[&str] = &["certain", "firm", "tentative"];

async fn update_finding_status(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    // Validate status value
    let status = payload.status.to_lowercase();
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid status '{}'. Valid: {:?}", status, VALID_STATUSES),
        ));
    }

    // Use transition_finding_status to enforce the state machine
    repo.transition_finding_status(&id, status).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;
    Ok(StatusCode::OK)
}

pub fn router() -> Router<AppState> {
    Router::new()
        // Create finding explicitly under a section
        .route("/api/vrkb/sections/:id/findings", post(create_finding))
        // List findings globally (with query params)
        .route("/api/vrkb/findings", get(list_findings))
        // Get / Update / Delete a single finding
        .route(
            "/api/vrkb/findings/:id",
            get(get_finding)
                .put(update_finding)
                .delete(delete_finding),
        )
        // Dedicated status update endpoint
        .route(
            "/api/vrkb/findings/:id/status",
            patch(update_finding_status),
        )
}
