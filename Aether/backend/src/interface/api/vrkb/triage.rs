// VRKB-03: Triage Queue API
// Unified triage review endpoints for findings with categorized queue and triage actions

use crate::domain::models::VrkbFinding;
use crate::domain::ports::VrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TriageStats {
    pub unreviewed: usize,
    pub duplicate_suspects: usize,
    pub stale: usize,
    pub missing_evidence: usize,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorizedTriageQueue {
    pub unreviewed: Vec<VrkbFinding>,
    pub duplicate_suspects: Vec<VrkbFinding>,
    pub stale: Vec<VrkbFinding>,
    pub missing_evidence: Vec<VrkbFinding>,
}

#[derive(Deserialize)]
pub struct TriageQuery {
    pub filter: Option<String>, // "unreviewed", "duplicates", "stale", "missing_evidence", "all"
}

#[derive(Deserialize)]
pub struct MergeRequest {
    pub canonical_id: Uuid,
}

/// GET /api/vrkb/projects/:id/triage — returns categorized triage review queue
async fn get_triage_queue(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Query(query): Query<TriageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    let filter = query.filter.unwrap_or_else(|| "all".to_string());

    match filter.as_str() {
        "unreviewed" => {
            let findings = repo.list_triage_unreviewed(&project_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(serde_json::to_value(findings).unwrap()))
        }
        "duplicates" => {
            let findings = repo.list_triage_duplicate_suspects(&project_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(serde_json::to_value(findings).unwrap()))
        }
        "stale" => {
            let findings = repo.list_triage_stale(&project_id, 7).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(serde_json::to_value(findings).unwrap()))
        }
        "missing_evidence" => {
            let findings = repo.list_triage_missing_evidence(&project_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(serde_json::to_value(findings).unwrap()))
        }
        _ => {
            // "all" — return categorized queue
            let (unreviewed, duplicates, stale, missing) = tokio::try_join!(
                repo.list_triage_unreviewed(&project_id),
                repo.list_triage_duplicate_suspects(&project_id),
                repo.list_triage_stale(&project_id, 7),
                repo.list_triage_missing_evidence(&project_id),
            ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let queue = CategorizedTriageQueue {
                unreviewed,
                duplicate_suspects: duplicates,
                stale,
                missing_evidence: missing,
            };
            Ok(Json(serde_json::to_value(queue).unwrap()))
        }
    }
}

/// GET /api/vrkb/projects/:id/triage/stats — triage queue statistics
async fn get_triage_stats(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<TriageStats>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    let (unreviewed, duplicates, stale, missing) = tokio::try_join!(
        repo.list_triage_unreviewed(&project_id),
        repo.list_triage_duplicate_suspects(&project_id),
        repo.list_triage_stale(&project_id, 7),
        repo.list_triage_missing_evidence(&project_id),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = unreviewed.len() + duplicates.len() + stale.len() + missing.len();

    Ok(Json(TriageStats {
        unreviewed: unreviewed.len(),
        duplicate_suspects: duplicates.len(),
        stale: stale.len(),
        missing_evidence: missing.len(),
        total,
    }))
}

/// POST /api/vrkb/projects/:id/triage/:finding_id/accept — accept finding (transition to confirmed)
async fn accept_finding(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((_project_id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VrkbFinding>, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    repo.transition_finding_status(&finding_id, "confirmed".to_string())
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let finding = repo.get_finding(&finding_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Finding not found".to_string()))?;

    Ok(Json(finding))
}

/// POST /api/vrkb/projects/:id/triage/:finding_id/reject — reject finding (transition to closed)
async fn reject_finding(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((_project_id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VrkbFinding>, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    // Reject = close with a verification note
    repo.update_finding(
        &finding_id,
        None, None,
        Some("closed".to_string()),
        None,
        Some(false), // no longer in triage
        None, None, None, None, None, None,
        Some(Some("Rejected during triage review".to_string())),
    ).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let finding = repo.get_finding(&finding_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Finding not found".to_string()))?;

    Ok(Json(finding))
}

/// POST /api/vrkb/projects/:id/triage/:finding_id/merge — merge as duplicate
async fn merge_finding(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((_project_id, finding_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<MergeRequest>,
) -> Result<Json<VrkbFinding>, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    repo.merge_finding_duplicate(&finding_id, &payload.canonical_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let finding = repo.get_finding(&finding_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Finding not found".to_string()))?;

    Ok(Json(finding))
}

/// POST /api/vrkb/projects/:id/triage/:finding_id/request-evidence — request more evidence
async fn request_evidence(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((_project_id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VrkbFinding>, (StatusCode, String)> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;

    // Mark finding as needing evidence by updating verification_note
    repo.update_finding(
        &finding_id,
        None, None, None, None, None,
        None, None, None, None, None, None,
        Some(Some("Evidence requested during triage review".to_string())),
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let finding = repo.get_finding(&finding_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Finding not found".to_string()))?;

    Ok(Json(finding))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/triage", get(get_triage_queue))
        .route("/api/vrkb/projects/:id/triage/stats", get(get_triage_stats))
        .route("/api/vrkb/projects/:id/triage/:finding_id/accept", post(accept_finding))
        .route("/api/vrkb/projects/:id/triage/:finding_id/reject", post(reject_finding))
        .route("/api/vrkb/projects/:id/triage/:finding_id/merge", post(merge_finding))
        .route("/api/vrkb/projects/:id/triage/:finding_id/request-evidence", post(request_evidence))
}
