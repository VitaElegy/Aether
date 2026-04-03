// VRKB-03: Triage Queue API
// Unified triage review endpoints for findings

use crate::domain::models::VrkbFinding;
use crate::domain::ports::VrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TriageStats {
    pub unreviewed: usize,
    pub stale: usize,
    pub missing_evidence: usize,
    pub total: usize,
}

#[derive(Deserialize)]
pub struct TriageQuery {
    pub filter: Option<String>, // "unreviewed", "stale", "missing_evidence", "all"
}

/// GET /api/vrkb/projects/:id/triage — returns findings that need triage attention
async fn get_triage_queue(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Query(query): Query<TriageQuery>,
) -> Result<Json<Vec<VrkbFinding>>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    let findings = repo.list_findings(None, Some(project_id)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let filter = query.filter.unwrap_or_else(|| "unreviewed".to_string());

    let filtered: Vec<VrkbFinding> = match filter.as_str() {
        "unreviewed" => findings.into_iter().filter(|f| f.status == "triage" || f.is_triage).collect(),
        "stale" => {
            let stale_threshold = chrono::Utc::now() - chrono::Duration::days(7);
            findings.into_iter().filter(|f| f.updated_at < stale_threshold && f.status != "closed" && f.status != "risk_accepted").collect()
        },
        "missing_evidence" => {
            // Findings with no content/evidence (content is None or empty)
            findings.into_iter().filter(|f| {
                f.content.as_ref().map_or(true, |c| c.is_null() || (c.is_object() && c.as_object().unwrap().is_empty()))
            }).collect()
        },
        _ => findings,
    };

    Ok(Json(filtered))
}

/// GET /api/vrkb/projects/:id/triage/stats — triage queue statistics
async fn get_triage_stats(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<TriageStats>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    let findings = repo.list_findings(None, Some(project_id)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stale_threshold = chrono::Utc::now() - chrono::Duration::days(7);

    let unreviewed = findings.iter().filter(|f| f.status == "triage" || f.is_triage).count();
    let stale = findings.iter().filter(|f| f.updated_at < stale_threshold && f.status != "closed" && f.status != "risk_accepted").count();
    let missing_evidence = findings.iter().filter(|f| {
        f.content.as_ref().map_or(true, |c| c.is_null())
    }).count();

    Ok(Json(TriageStats {
        unreviewed,
        stale,
        missing_evidence,
        total: findings.len(),
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/triage", get(get_triage_queue))
        .route("/api/vrkb/projects/:id/triage/stats", get(get_triage_stats))
}
