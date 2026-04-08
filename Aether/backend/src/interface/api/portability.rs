use crate::domain::portability::models::{ExportSummary, ImportPreview};
use crate::domain::special_kb::renderer_id_or_default;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::pin::Pin;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::domain::ports::KnowledgeBaseRepository; // Import Trait

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:kb_id/export/preview", get(analyze_export))
        .route("/:kb_id/export/start", post(start_export))
        .route("/:kb_id/import/analyze", post(analyze_import))
        .route("/:kb_id/import/start", post(start_import))
        .route("/tasks/:task_id/progress", get(task_progress))
        .route("/tasks/:task_id/download", get(download_export))
}

async fn analyze_export(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(kb_id): Path<Uuid>,
) -> Result<Json<ExportSummary>, (StatusCode, String)> {
    // 1. Get KB to find type
    let kb = state
        .repo
        .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "KB not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    let renderer_id = renderer_id_or_default(kb.renderer_id.as_deref());

    let summary_result = state
        .portability_service
        .analyze_export(&renderer_id, kb_id)
        .await;

    let summary = match summary_result {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                serde_json::json!({ "error": e, "renderer_id": renderer_id }).to_string(),
            ))
        }
    };

    Ok(Json(summary))
}

async fn start_export(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(kb_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let kb = state
        .repo
        .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "KB not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    let renderer_id = renderer_id_or_default(kb.renderer_id.as_deref());

    let task_id = state
        .portability_service
        .start_export(&renderer_id, kb_id, user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}

async fn task_progress(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> impl IntoResponse {
    // Retrieve receiver
    let rx_opt = state.portability_service.get_task_receiver(task_id);

    let stream: Pin<Box<dyn Stream<Item = Result<Event, axum::Error>> + Send>> =
        if let Some(rx) = rx_opt {
            let s = ReceiverStream::new(rx).map(|event| {
                Event::default()
                    .event("message")
                    .json_data(event)
                    .map_err(axum::Error::new)
            });
            Box::pin(s)
        } else {
            let s = tokio_stream::once(Ok(Event::default()
                .event("error")
                .data("Task not found or already consumed")));
            Box::pin(s)
        };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(1)))
}

use axum::body::Body;
use axum::http::header;

/// PLAT-04: Analyze an import file - returns preview with conflicts and suggested actions
async fn analyze_import(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(kb_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<ImportPreview>, (StatusCode, String)> {
    // 1. Get KB to find type
    let kb = state
        .repo
        .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "KB not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    // 2. Extract file from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read upload: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let file_name = field.file_name().unwrap_or("import.zip").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file data: {}", e)))?;

    // 3. Save to temp file
    let temp_dir = std::env::temp_dir().join("aether_imports");
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create temp dir: {}", e)))?;

    let temp_path = temp_dir.join(format!("{}_{}", Uuid::new_v4(), file_name));
    tokio::fs::write(&temp_path, &data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {}", e)))?;

    // 4. Analyze using provider
    let renderer_id = renderer_id_or_default(kb.renderer_id.as_deref());
    let preview = state
        .portability_service
        .analyze_import(&renderer_id, temp_path)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(preview))
}

/// PLAT-04: Start import request body
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StartImportRequest {
    file_path: Option<String>,
    merge_strategy: Option<String>,
}

/// PLAT-04: Start the import process using the provider
async fn start_import(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(kb_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let kb = state
        .repo
        .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "KB not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    // Extract file from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read upload: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let file_name = field.file_name().unwrap_or("import.zip").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file data: {}", e)))?;

    // Save to temp file
    let temp_dir = std::env::temp_dir().join("aether_imports");
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create temp dir: {}", e)))?;

    let temp_path = temp_dir.join(format!("{}_{}", Uuid::new_v4(), file_name));
    tokio::fs::write(&temp_path, &data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {}", e)))?;

    let renderer_id = renderer_id_or_default(kb.renderer_id.as_deref());

    // PLAT-04: Use provider-based import (not backup_service)
    let task_id = state
        .portability_service
        .start_import_with_provider(&renderer_id, kb_id, temp_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}

/// PLAT-04: Query params for token-validated download
#[derive(Debug, Deserialize)]
struct DownloadQuery {
    token: Option<String>,
}

async fn download_export(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Query(query): Query<DownloadQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // PLAT-04: Token validation and expiry check
    let file_path = if let Some(token) = &query.token {
        state
            .portability_service
            .validate_download_token(task_id, token)
            .map_err(|e| (StatusCode::FORBIDDEN, e))?
    } else {
        // Fallback: allow without token for backward compatibility, but check expiry
        state
            .portability_service
            .get_task_result(task_id)
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    "Export archive not found or expired.".to_string(),
                )
            })?
    };

    // 2. Open the file asynchronously
    let file = tokio::fs::File::open(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open export file: {}", e),
        )
    })?;

    // 3. Create a stream from the file
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 4. Build the final response with headers
    let filename = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("export.zip");

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        ),
    ];

    Ok((headers, body))
}
