use axum::{
    extract::{Path, State},
    response::{IntoResponse, sse::{Sse, Event}},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use uuid::Uuid;
use std::time::Duration;
use std::pin::Pin;
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::portability::models::ExportSummary;

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
    let kb = state.repo.find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "KB not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    let renderer_id = kb.renderer_id.unwrap_or_else(|| "default".to_string());
    
    let summary_result = state.portability_service.analyze_export(&renderer_id, kb_id).await;
    
    let summary = match summary_result {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::BAD_REQUEST, serde_json::json!({ "error": e, "renderer_id": renderer_id }).to_string())),
    };

    Ok(Json(summary))
}

async fn start_export(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(kb_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let kb = state.repo.find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "KB not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    let renderer_id = kb.renderer_id.unwrap_or_else(|| "default".to_string());

    let task_id = state.portability_service.start_export(&renderer_id, kb_id)
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}

async fn task_progress(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> impl IntoResponse {
    // Retrieve receiver
    let rx_opt = state.portability_service.get_task_receiver(task_id);
    
    let stream: Pin<Box<dyn Stream<Item = Result<Event, axum::Error>> + Send>> = if let Some(rx) = rx_opt {
        let s = ReceiverStream::new(rx)
            .map(|event| Event::default().json_data(event).map_err(axum::Error::new));
        Box::pin(s)
    } else {
        let s = tokio_stream::once(Ok(Event::default().event("error").data("Task not found or already consumed")));
        Box::pin(s)
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(1)))
}

async fn download_export(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(_task_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: Implement download logic.
    // The export task should have saved the file path somewhere accessible by task_id.
    // For MVP, the "Completed" event in SSE carries the download URL or path.
    // Or we can store the result in PortabilityService.
    
    // Since we didn't implement result storage in PortabilityService yet (just fire and forget),
    // we can't easily serve it here without refactoring.
    
    // FIX: The SSE "Completed" message should contain a direct link to a static file handler 
    // OR we need to store the result.
    
    // For now, let's assume the frontend uses the link provided in the SSE event.
    // But wait, the SSE event just says "Export ready".
    
    Err::<axum::response::Response, (StatusCode, String)>((StatusCode::NOT_IMPLEMENTED, "Download via task ID not yet implemented. Use the link from progress event.".to_string()).into())
}
