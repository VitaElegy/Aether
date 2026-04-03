use crate::domain::models::VrkbDoc;
use crate::domain::ports::VrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/vrkb/projects/:id/docs",
            get(list_docs).post(create_doc),
        )
        .route("/api/vrkb/projects/:id/trash", get(list_trash)) // New
        .route(
            "/api/vrkb/docs/:id",
            get(get_doc).put(update_doc).delete(delete_doc),
        )
        .route(
            "/api/vrkb/docs/:id/restore",
            axum::routing::post(restore_doc),
        ) // New
        .route(
            "/api/vrkb/docs/:id/permanent",
            axum::routing::delete(permanent_delete_doc),
        ) // New
        // --- VRKB-07: Move doc, templates, report generation ---
        .route(
            "/api/vrkb/docs/:id/move",
            axum::routing::post(move_doc),
        )
        .route(
            "/api/vrkb/docs/templates",
            get(list_templates),
        )
        .route(
            "/api/vrkb/projects/:id/docs/from-template",
            axum::routing::post(create_from_template),
        )
        .route(
            "/api/vrkb/projects/:id/report",
            axum::routing::post(generate_report),
        )
}

#[derive(serde::Deserialize)]
struct CreateDocRequest {
    title: String,
    content: Option<serde_json::Value>,
    parent_id: Option<Uuid>,
}

#[derive(serde::Deserialize)]
struct UpdateDocRequest {
    title: String,
    content: Option<serde_json::Value>,
    parent_id: Option<Uuid>,
}

// --- VRKB-07: Move doc request ---
#[derive(Deserialize)]
struct MoveDocRequest {
    parent_id: Option<Uuid>,
}

// --- VRKB-07: Doc template model ---
#[derive(Serialize)]
struct DocTemplate {
    id: String,
    name: String,
    description: String,
    content: serde_json::Value,
}

// --- VRKB-07: Create from template request ---
#[derive(Deserialize)]
struct CreateFromTemplateRequest {
    template_id: String,
    title: String,
}

// --- VRKB-07: Report generation request ---
#[derive(Deserialize)]
struct GenerateReportRequest {
    include_findings: Option<bool>,
    include_appendix: Option<bool>,
}

// --- VRKB-07: Report generation response ---
#[derive(Serialize)]
struct ReportResponse {
    doc_id: Uuid,
    title: String,
    message: String,
}

async fn list_docs(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbDoc>>, (StatusCode, String)> {
    let docs = state
        .repo
        .list_docs(&project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(docs))
}

async fn list_trash(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbDoc>>, (StatusCode, String)> {
    let docs = state
        .repo
        .list_trash(&project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(docs))
}

async fn create_doc(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateDocRequest>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let doc = VrkbDoc {
        id: Uuid::new_v4(),
        project_id,
        title: payload.title,
        content: payload.content,
        parent_id: payload.parent_id,
        author_id: Some(user.id),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };
    state
        .repo
        .create_doc(doc.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(doc))
}

async fn get_doc(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let doc = state
        .repo
        .get_doc(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match doc {
        Some(d) => Ok(Json(d)),
        None => Err((StatusCode::NOT_FOUND, "Doc not found".to_string())),
    }
}

async fn update_doc(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDocRequest>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let existing = state
        .repo
        .get_doc(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(mut doc) = existing {
        doc.title = payload.title;
        doc.content = payload.content;
        doc.parent_id = payload.parent_id;
        doc.updated_at = Utc::now();

        // Clone doc because repo.update_doc consumes it
        state
            .repo
            .update_doc(doc.clone())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(doc))
    } else {
        Err((StatusCode::NOT_FOUND, "Doc not found".to_string()))
    }
}

async fn delete_doc(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .repo
        .delete_doc(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn restore_doc(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .repo
        .restore_doc(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn permanent_delete_doc(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .repo
        .permanent_delete_doc(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

// --- VRKB-07: Move doc to a new parent ---
async fn move_doc(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<MoveDocRequest>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let existing = state
        .repo
        .get_doc(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(mut doc) = existing {
        doc.parent_id = payload.parent_id;
        doc.updated_at = Utc::now();
        state
            .repo
            .update_doc(doc.clone())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(doc))
    } else {
        Err((StatusCode::NOT_FOUND, "Doc not found".to_string()))
    }
}

// --- VRKB-07: List built-in doc templates ---
async fn list_templates() -> Json<Vec<DocTemplate>> {
    let templates = vec![
        DocTemplate {
            id: "pentest-report".to_string(),
            name: "Pentest Report".to_string(),
            description: "Standard penetration testing report with executive summary, methodology, findings, and recommendations".to_string(),
            content: serde_json::json!({
                "sections": [
                    {"title": "Executive Summary", "content": ""},
                    {"title": "Scope & Methodology", "content": ""},
                    {"title": "Findings Summary", "content": ""},
                    {"title": "Detailed Findings", "content": ""},
                    {"title": "Recommendations", "content": ""},
                    {"title": "Appendix", "content": ""}
                ]
            }),
        },
        DocTemplate {
            id: "vuln-assessment".to_string(),
            name: "Vulnerability Assessment".to_string(),
            description: "Vulnerability assessment document with risk ratings and remediation timeline".to_string(),
            content: serde_json::json!({
                "sections": [
                    {"title": "Overview", "content": ""},
                    {"title": "Risk Summary", "content": ""},
                    {"title": "Vulnerability Details", "content": ""},
                    {"title": "Remediation Plan", "content": ""},
                    {"title": "Timeline", "content": ""}
                ]
            }),
        },
        DocTemplate {
            id: "meeting-notes".to_string(),
            name: "Meeting Notes".to_string(),
            description: "Project meeting notes template with action items".to_string(),
            content: serde_json::json!({
                "sections": [
                    {"title": "Attendees", "content": ""},
                    {"title": "Agenda", "content": ""},
                    {"title": "Discussion", "content": ""},
                    {"title": "Action Items", "content": ""},
                    {"title": "Next Meeting", "content": ""}
                ]
            }),
        },
        DocTemplate {
            id: "blank".to_string(),
            name: "Blank Document".to_string(),
            description: "Empty document".to_string(),
            content: serde_json::json!({}),
        },
    ];
    Json(templates)
}

// --- VRKB-07: Create doc from template ---
async fn create_from_template(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateFromTemplateRequest>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    // Get template content
    let templates = list_templates().await;
    let template = templates
        .0
        .iter()
        .find(|t| t.id == payload.template_id)
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Template '{}' not found", payload.template_id),
        ))?;

    let doc = VrkbDoc {
        id: Uuid::new_v4(),
        project_id,
        title: payload.title,
        content: Some(template.content.clone()),
        parent_id: None,
        author_id: Some(user.id),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };
    state
        .repo
        .create_doc(doc.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(doc))
}

// --- VRKB-07: Generate report (compiles findings + docs into a report doc) ---
async fn generate_report(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<GenerateReportRequest>,
) -> Result<Json<ReportResponse>, (StatusCode, String)> {
    let include_findings = payload.include_findings.unwrap_or(true);
    let include_appendix = payload.include_appendix.unwrap_or(true);

    // Gather project data
    let mut report_content = serde_json::json!({
        "type": "generated_report",
        "generated_at": Utc::now().to_rfc3339(),
        "sections": []
    });

    let sections = report_content["sections"].as_array_mut().unwrap();

    // Executive Summary
    sections.push(serde_json::json!({
        "title": "Executive Summary",
        "content": "Auto-generated report for the VRKB project."
    }));

    // Findings section
    if include_findings {
        let findings = state
            .repo
            .list_findings(None, Some(project_id))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let findings_data: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "title": f.title,
                    "severity": f.severity,
                    "status": f.status,
                    "confidence": f.confidence,
                })
            })
            .collect();

        sections.push(serde_json::json!({
            "title": "Findings",
            "count": findings.len(),
            "items": findings_data
        }));
    }

    // Appendix section
    if include_appendix {
        let docs = state
            .repo
            .list_docs(&project_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let doc_refs: Vec<serde_json::Value> = docs
            .iter()
            .map(|d| {
                serde_json::json!({
                    "title": d.title,
                    "id": d.id.to_string()
                })
            })
            .collect();

        sections.push(serde_json::json!({
            "title": "Appendix - Document References",
            "items": doc_refs
        }));
    }

    // Create the report as a new doc
    let report_title = format!("Report - {}", Utc::now().format("%Y-%m-%d %H:%M"));
    let doc = VrkbDoc {
        id: Uuid::new_v4(),
        project_id,
        title: report_title.clone(),
        content: Some(report_content),
        parent_id: None,
        author_id: Some(user.id),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };
    state
        .repo
        .create_doc(doc.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ReportResponse {
        doc_id: doc.id,
        title: report_title,
        message: "Report generated successfully".to_string(),
    }))
}
