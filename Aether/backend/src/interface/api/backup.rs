use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateBackupRequest {
    kb_id: Uuid,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_backups).post(create_backup))
        .route("/download/:filename", get(download_backup))
        .route("/restore", post(restore_backup))
        .route("/preview", post(preview_backup))
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
struct BackupDiagnosticPayload {
    error: String,
    code: &'static str,
    details: Option<String>,
    hint: Option<&'static str>,
    stage: &'static str,
}

fn classify_backup_diagnostic(
    stage: &'static str,
    raw: &str,
) -> (&'static str, String, Option<&'static str>) {
    if raw.contains("Invalid file type") {
        return (
            "invalid_file_type",
            "Invalid backup file type. Only .akb and .zip files are supported.".to_string(),
            Some("Select a backup exported by Aether, or rename the file only if you are certain the archive is valid."),
        );
    }

    if raw == "No file uploaded" {
        return (
            "no_file_uploaded",
            "No backup file was received by the server.".to_string(),
            Some("Choose a file first, then retry the upload."),
        );
    }

    if raw.contains("Failed to parse upload form") {
        return (
            "multipart_parse_failed",
            "The server could not read the uploaded form data.".to_string(),
            Some("Retry the upload. If it still fails, check reverse proxy or browser upload limits."),
        );
    }

    if raw.contains("Failed to read uploaded backup") {
        return (
            "upload_read_failed",
            "The server failed while reading the uploaded backup file.".to_string(),
            Some("Retry with the same file. If it keeps failing, inspect browser/network upload limits."),
        );
    }

    if raw.contains("Failed to write uploaded backup") {
        return (
            "temp_write_failed",
            "The server could not persist the uploaded backup to temporary storage.".to_string(),
            Some("Check disk permissions and available space on the server."),
        );
    }

    if raw == "Uploaded backup file is empty" {
        return (
            "empty_backup",
            "The uploaded backup file is empty.".to_string(),
            Some("Export the knowledge base again and verify the file size is not 0 bytes."),
        );
    }

    if raw.contains("Failed to inspect uploaded backup") {
        return (
            "inspect_failed",
            "The server could not inspect the uploaded backup file.".to_string(),
            Some("Retry the upload. If the error persists, inspect server filesystem permissions."),
        );
    }

    if raw.contains("Failed to open uploaded backup") {
        return (
            "open_failed",
            "The server could not open the uploaded backup file.".to_string(),
            Some("Retry the upload. If it persists, inspect temporary storage permissions on the server."),
        );
    }

    if raw.contains("Invalid backup archive:") || raw.contains("invalid Zip archive") {
        return (
            "invalid_archive",
            "The uploaded file is not a valid Aether backup archive.".to_string(),
            Some("The file may be corrupted, incomplete, or not actually a ZIP-based .akb export."),
        );
    }

    if raw.contains("missing meta.json") {
        return (
            "missing_meta",
            "The backup archive is missing meta.json.".to_string(),
            Some("This usually means the file was not created by Aether, or the archive is incomplete."),
        );
    }

    if raw.contains("Portability export detected") {
        return (
            "wrong_archive_type",
            "This Smart Portability export package does not contain an embedded restorable snapshot.".to_string(),
            Some("Re-export the knowledge base with the current version of Aether, or use Legacy System Backup/Create Snapshot for a direct restore package."),
        );
    }

    if raw.contains("Failed to read meta.json") {
        return (
            "meta_read_failed",
            "The server found meta.json but could not read it.".to_string(),
            Some("The archive contents may be damaged. Re-export the backup and try again."),
        );
    }

    if raw.contains("Invalid meta.json") {
        return (
            "invalid_meta",
            "The backup metadata format is invalid.".to_string(),
            Some("The backup may come from an incompatible version or a damaged archive."),
        );
    }

    if stage == "restore" {
        return (
            "restore_failed",
            "The backup upload succeeded, but restoration failed.".to_string(),
            Some("Inspect the detailed error below to find the failing restore step."),
        );
    }

    (
        "preview_failed",
        "The server could not preview this backup file.".to_string(),
        Some("Inspect the detailed error below to determine whether the archive is invalid or the server could not read it."),
    )
}

fn backup_diagnostic_payload(stage: &'static str, raw: impl Into<String>) -> BackupDiagnosticPayload {
    let raw = raw.into();
    let (code, error, hint) = classify_backup_diagnostic(stage, &raw);
    BackupDiagnosticPayload {
        error,
        code,
        details: Some(raw),
        hint,
        stage,
    }
}

fn backup_diagnostic_response(
    stage: &'static str,
    status: StatusCode,
    raw: impl Into<String>,
) -> (StatusCode, Json<BackupDiagnosticPayload>) {
    (status, Json(backup_diagnostic_payload(stage, raw)))
}

async fn create_backup(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateBackupRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Trigger Backup
    let filename = state
        .backup_service
        .create_backup(payload.kb_id, user.id)
        .await
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
    let files = state
        .backup_service
        .list_backups()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(files))
}

async fn download_backup(
    State(state): State<AppState>,
    user: AuthenticatedUser, // Require auth
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Security: Prevent path traversal
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") || !filename.ends_with(".akb") {
        return Err((StatusCode::BAD_REQUEST, "Invalid filename".to_string()));
    }

    // Check ownership
    // This requires a DB lookup.
    let parts: Vec<&str> = filename.split('_').collect();
    if parts.len() < 2 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid filename format".to_string(),
        ));
    }

    let kb_id_str = parts[0];
    let kb_id = Uuid::parse_str(kb_id_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid KB ID in filename".to_string(),
        )
    })?;

    use crate::domain::ports::KnowledgeBaseRepository;
    let kb = state
        .repo
        .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Knowledge Base not found".to_string(),
        ))?;

    if kb.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    // Serve File
    let path = std::path::Path::new("backups").join(&filename); // Hardcoded relative path matching service

    let file = File::open(&path)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Backup file not found".to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        ),
    ];

    Ok((headers, body))
}

async fn restore_backup(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<BackupDiagnosticPayload>)> {
    // 1. Receive File
    let mut file_path = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            backup_diagnostic_response(
                "restore",
                StatusCode::BAD_REQUEST,
                format!("Failed to parse upload form: {}", e),
            )
        })?
    {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("backup.akb").to_string();
            // Validate extension
            if !filename.ends_with(".akb") && !filename.ends_with(".zip") {
                return Err(backup_diagnostic_response(
                    "restore",
                    StatusCode::BAD_REQUEST,
                    "Invalid file type. Must be .akb or .zip".to_string(),
                ));
            }

            // Save to temp
            let temp_dir = std::env::temp_dir();
            let target_path = temp_dir.join(format!("restore_{}_{}", Uuid::new_v4(), filename));

            let data = field
                .bytes()
                .await
                .map_err(|e| {
                    backup_diagnostic_response(
                        "restore",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read uploaded backup: {}", e),
                    )
                })?;
            tokio::fs::write(&target_path, data)
                .await
                .map_err(|e| {
                    backup_diagnostic_response(
                        "restore",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write uploaded backup: {}", e),
                    )
                })?;

            file_path = Some(target_path);
            break; // One file only
        }
    }

    let path = file_path.ok_or_else(|| {
        backup_diagnostic_response(
            "restore",
            StatusCode::BAD_REQUEST,
            "No file uploaded".to_string(),
        )
    })?;

    // 2. Trigger Restore
    let new_kb_id = state
        .backup_service
        .restore_backup(path.clone(), user.id, None, None)
        .await
        .map_err(|e| {
            // Try to cleanup
            let _ = std::fs::remove_file(&path);
            backup_diagnostic_response("restore", StatusCode::INTERNAL_SERVER_ERROR, e)
        })?;

    // Cleanup
    let _ = std::fs::remove_file(path);

    Ok(Json(serde_json::json!({
        "status": "success",
        "new_kb_id": new_kb_id
    })))
}

async fn preview_backup(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<BackupDiagnosticPayload>)> {
    // 1. Receive File
    let mut file_path = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            backup_diagnostic_response(
                "preview",
                StatusCode::BAD_REQUEST,
                format!("Failed to parse upload form: {}", e),
            )
        })?
    {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("backup.akb").to_string();
            // Validate extension
            if !filename.ends_with(".akb") && !filename.ends_with(".zip") {
                return Err(backup_diagnostic_response(
                    "preview",
                    StatusCode::BAD_REQUEST,
                    "Invalid file type. Must be .akb or .zip".to_string(),
                ));
            }

            // Save to temp
            let temp_dir = std::env::temp_dir();
            let target_path = temp_dir.join(format!("preview_{}_{}", Uuid::new_v4(), filename));

            let data = field
                .bytes()
                .await
                .map_err(|e| {
                    backup_diagnostic_response(
                        "preview",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read uploaded backup: {}", e),
                    )
                })?;
            tokio::fs::write(&target_path, data)
                .await
                .map_err(|e| {
                    backup_diagnostic_response(
                        "preview",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write uploaded backup: {}", e),
                    )
                })?;

            file_path = Some(target_path);
            break; // One file only
        }
    }

    let path = file_path.ok_or_else(|| {
        backup_diagnostic_response(
            "preview",
            StatusCode::BAD_REQUEST,
            "No file uploaded".to_string(),
        )
    })?;

    // 2. Trigger Preview
    let summary = state.backup_service.preview_backup(&path).map_err(|e| {
        // Try to cleanup
        let _ = std::fs::remove_file(&path);
        backup_diagnostic_response("preview", StatusCode::BAD_REQUEST, e)
    })?;

    // Cleanup
    let _ = std::fs::remove_file(path);

    Ok(Json(summary))
}

#[cfg(test)]
mod tests {
    use super::{backup_diagnostic_payload, classify_backup_diagnostic};

    #[test]
    fn classifies_missing_meta_preview_errors() {
        let payload = backup_diagnostic_payload("preview", "Invalid backup: missing meta.json");

        assert_eq!(payload.code, "missing_meta");
        assert_eq!(payload.error, "The backup archive is missing meta.json.");
        assert!(payload
            .hint
            .expect("missing hint")
            .contains("not created by Aether"));
    }

    #[test]
    fn classifies_invalid_archive_errors() {
        let (code, message, hint) = classify_backup_diagnostic(
            "preview",
            "Invalid backup archive: invalid Zip archive: Could not find EOCD",
        );

        assert_eq!(code, "invalid_archive");
        assert_eq!(message, "The uploaded file is not a valid Aether backup archive.");
        assert!(hint.expect("missing hint").contains("corrupted"));
    }

    #[test]
    fn classifies_portability_export_archives() {
        let payload = backup_diagnostic_payload(
            "preview",
            "Portability export detected: this archive does not contain an embedded snapshot.akb and cannot be restored directly",
        );

        assert_eq!(payload.code, "wrong_archive_type");
        assert_eq!(
            payload.error,
            "This Smart Portability export package does not contain an embedded restorable snapshot."
        );
        assert!(payload
            .hint
            .expect("missing hint")
            .contains("Re-export"));
    }

    #[test]
    fn falls_back_to_restore_failed_for_unknown_restore_errors() {
        let payload = backup_diagnostic_payload("restore", "Content missing: content/root.md");

        assert_eq!(payload.code, "restore_failed");
        assert_eq!(
            payload.error,
            "The backup upload succeeded, but restoration failed."
        );
        assert_eq!(
            payload.details.as_deref(),
            Some("Content missing: content/root.md")
        );
    }
}
