use crate::domain::portability::models::ExportSummary;
use crate::domain::special_kb::renderer_id_or_default;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
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

async fn download_export(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Fetch file path from completed tasks
    let file_path = state
        .portability_service
        .get_task_result(task_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                "Export archive not found or expired.".to_string(),
            )
        })?;

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
