use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use chrono::Utc;
use crate::interface::state::AppState;
use crate::domain::models::{LayoutTemplate, permissions};
use crate::infrastructure::persistence::repositories::layout_template_repository::LayoutTemplateRepository;
use crate::interface::api::auth::AuthenticatedUser;

#[derive(Deserialize)]
pub struct CreateTemplateDto {
    pub renderer_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: Option<String>,
    pub tags: Vec<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct UpdateTemplateDto {
    pub renderer_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub tags: Option<Vec<String>>,
    pub config: Option<serde_json::Value>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/templates", get(list_templates).post(create_template))
        .route("/api/templates/:id", put(update_template).delete(delete_template))
}

async fn list_templates(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let templates = LayoutTemplateRepository::list(&state.repo.db).await.unwrap_or_default();
    Json(templates)
}

async fn create_template(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTemplateDto>,
) -> impl IntoResponse {
    if !user.has_permission(permissions::ADMIN) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    let template = LayoutTemplate {
        id: Uuid::new_v4(),
        renderer_id: payload.renderer_id,
        title: payload.title,
        description: payload.description,
        thumbnail: payload.thumbnail,
        tags: payload.tags,
        config: payload.config.unwrap_or(serde_json::json!({})),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match LayoutTemplateRepository::create(&state.repo.db, template.clone()).await {
        Ok(_) => Ok(Json(template)),
        Err(e) => {
            eprintln!("Failed to create template: {:?}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_template(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateTemplateDto>, // Full replacement model
) -> impl IntoResponse {
   if !user.has_permission(permissions::ADMIN) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    
    let template = LayoutTemplate {
        id: id,
        renderer_id: payload.renderer_id,
        title: payload.title,
        description: payload.description,
        thumbnail: payload.thumbnail,
        tags: payload.tags,
        config: payload.config.unwrap_or(serde_json::json!({})),
        created_at: Utc::now(), // Repo ignores this
        updated_at: Utc::now(),
    };
    
     match LayoutTemplateRepository::update(&state.repo.db, id, template.clone()).await {
        Ok(_) => Ok(Json(template)),
        Err(e) => {
            eprintln!("Failed to update template: {:?}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_template(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if !user.has_permission(permissions::ADMIN) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    match LayoutTemplateRepository::delete(&state.repo.db, id).await {
        Ok(_) => Ok(axum::http::StatusCode::NO_CONTENT),
        Err(e) => {
            eprintln!("Failed to delete template: {:?}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
