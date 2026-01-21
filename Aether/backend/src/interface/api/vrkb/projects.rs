use axum::{
    Json, extract::{State, Path}, response::IntoResponse, http::StatusCode, routing::{get, post}, Router
};
use uuid::Uuid;
use chrono::Utc;
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::models::{VrkbProject, VrkbSection};
use crate::domain::ports::{VrkbRepository, RepositoryError};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    name: String,
    repository_url: Option<String>,
    quota_bytes: Option<i64>,
    settings: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct CreateSectionRequest {
    title: String,
    checklist: Option<serde_json::Value>,
}

// --- Handlers ---

async fn list_projects(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<VrkbProject>>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.list_projects().await {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_project(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<VrkbProject>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    
    let new_project = VrkbProject {
        id: Uuid::new_v4(),
        name: payload.name,
        repository_url: payload.repository_url,
        quota_bytes: payload.quota_bytes.unwrap_or(5368709120), // 5GB
        settings: payload.settings,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match repo.create_project(new_project.clone()).await {
        Ok(_) => Ok(Json(new_project)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_project(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VrkbProject>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.get_project(&id).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_sections(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbSection>>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    match repo.list_sections(&project_id).await {
        Ok(sections) => Ok(Json(sections)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_section(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateSectionRequest>,
) -> Result<Json<VrkbSection>, StatusCode> {
    let repo = state.repo.clone() as Arc<dyn VrkbRepository>;
    
    let new_section = VrkbSection {
        id: Uuid::new_v4(),
        project_id,
        title: payload.title,
        checklist: payload.checklist,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match repo.create_section(new_section.clone()).await {
        Ok(_) => Ok(Json(new_section)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vrkb/projects", get(list_projects).post(create_project))
        .route("/vrkb/projects/:id", get(get_project))
        .route("/vrkb/projects/:id/sections", get(list_sections).post(create_section))
}
