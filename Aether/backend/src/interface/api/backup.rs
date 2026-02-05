use axum::{
    extract::{Path, State, Multipart},
    response::IntoResponse,
    http::{StatusCode, header},
    routing::{get, post},
    Json, Router,
    body::Body,
};
use tokio_util::io::ReaderStream;
use tokio::fs::File;
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;

#[derive(serde::Deserialize)]
pub struct CreateBackupRequest {
    kb_id: Uuid,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_backups).post(create_backup))
        .route("/download/:filename", get(download_backup))
        .route("/restore", post(restore_backup))
}

async fn create_backup(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateBackupRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Trigger Backup
    let filename = state.backup_service.create_backup(payload.kb_id, user.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "filename": filename
    })))
}

async fn list_backups(
    State(state): State<AppState>,
    _user: AuthenticatedUser, // Require auth
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let files = state.backup_service.list_backups().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(files))
}

async fn download_backup(
    State(state): State<AppState>,
    user: AuthenticatedUser, // Require auth
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Security: Prevent path traversal
    if filename.contains("..") || filename.contains("/") || !filename.ends_with(".akb") {
        return Err((StatusCode::BAD_REQUEST, "Invalid filename".to_string()));
    }

    // Check ownership
    // This requires a DB lookup.
    let parts: Vec<&str> = filename.split('_').collect();
    if parts.len() < 2 {
        return Err((StatusCode::BAD_REQUEST, "Invalid filename format".to_string()));
    }
    
    let kb_id_str = parts[0];
    let kb_id = Uuid::parse_str(kb_id_str).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid KB ID in filename".to_string()))?;

    use crate::domain::ports::KnowledgeBaseRepository;
    let kb = state.repo.find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id)).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Knowledge Base not found".to_string()))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    // Serve File
    let path = std::path::Path::new("backups").join(&filename); // Hardcoded relative path matching service
    
    let file = File::open(&path).await
        .map_err(|_| (StatusCode::NOT_FOUND, "Backup file not found".to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    ];

    Ok((headers, body))
}

async fn restore_backup(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Receive File
    let mut file_path = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("backup.akb").to_string();
            // Validate extension
            if !filename.ends_with(".akb") && !filename.ends_with(".zip") {
                return Err((StatusCode::BAD_REQUEST, "Invalid file type. Must be .akb or .zip".to_string()));
            }

            // Save to temp
            let temp_dir = std::env::temp_dir();
            let target_path = temp_dir.join(format!("restore_{}_{}", Uuid::new_v4(), filename));
            
            let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            tokio::fs::write(&target_path, data).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
            file_path = Some(target_path);
            break; // One file only
        }
    }

    let path = file_path.ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    // 2. Trigger Restore
    let new_kb_id = state.backup_service.restore_backup(path.clone(), user.id).await
        .map_err(|e| {
            // Try to cleanup
            let _ = std::fs::remove_file(&path);
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        })?;

    // Cleanup
    let _ = std::fs::remove_file(path);

    Ok(Json(serde_json::json!({
        "status": "success",
        "new_kb_id": new_kb_id
    })))
}
