use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::domain::models::VrkbDoc;
use crate::domain::ports::VrkbRepository;
use axum::http::StatusCode;
use chrono::Utc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/projects/:id/docs", get(list_docs).post(create_doc))
        .route("/api/vrkb/projects/:id/trash", get(list_trash)) // New
        .route("/api/vrkb/docs/:id", get(get_doc).put(update_doc).delete(delete_doc))
        .route("/api/vrkb/docs/:id/restore", axum::routing::post(restore_doc)) // New
        .route("/api/vrkb/docs/:id/permanent", axum::routing::delete(permanent_delete_doc)) // New
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

async fn list_docs(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbDoc>>, (StatusCode, String)> {
    let docs = state.repo.list_docs(&project_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(docs))
}

async fn list_trash(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbDoc>>, (StatusCode, String)> {
    let docs = state.repo.list_trash(&project_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(docs))
}

async fn create_doc(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateDocRequest>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let doc = VrkbDoc {
        id: Uuid::new_v4(),
        project_id,
        title: payload.title,
        content: payload.content,
        parent_id: payload.parent_id,
        author_id: None, // TODO: Extract from auth context
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };
    state.repo.create_doc(doc.clone()).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(doc))
}

async fn get_doc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let doc = state.repo.get_doc(&id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match doc {
        Some(d) => Ok(Json(d)),
        None => Err((StatusCode::NOT_FOUND, "Doc not found".to_string())),
    }
}

async fn update_doc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDocRequest>,
) -> Result<Json<VrkbDoc>, (StatusCode, String)> {
    let existing = state.repo.get_doc(&id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(mut doc) = existing {
        doc.title = payload.title;
        doc.content = payload.content;
        doc.parent_id = payload.parent_id;
        doc.updated_at = Utc::now();
        
        // Clone doc because repo.update_doc consumes it
        state.repo.update_doc(doc.clone()).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(doc))
    } else {
        Err((StatusCode::NOT_FOUND, "Doc not found".to_string()))
    }
}

async fn delete_doc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.delete_doc(&id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn restore_doc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.restore_doc(&id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn permanent_delete_doc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.permanent_delete_doc(&id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}
