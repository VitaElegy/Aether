use axum::{
    Json, extract::{Multipart, State}, response::IntoResponse, http::StatusCode,
};
use crate::interface::api::auth::AuthenticatedUser;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

pub async fn upload_handler(
    State(repo): State<std::sync::Arc<crate::infrastructure::persistence::repositories::system_settings_repository::SystemSettingsRepository>>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let max_mb = repo.get_int("max_upload_size_mb", 5).await;
    let max_file_size = (max_mb as usize) * 1024 * 1024;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

            // Validate image
            if !content_type.starts_with("image/") {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Only images are allowed" }))).into_response();
            }

            // Stream and validate size
            let mut data = Vec::new();
            let mut stream = field;

            while let Ok(Some(chunk)) = stream.chunk().await {
                if data.len() + chunk.len() > max_file_size {
                    return (StatusCode::PAYLOAD_TOO_LARGE, Json(serde_json::json!({ "error": format!("File size exceeds {}MB limit", max_mb) }))).into_response();
                }
                data.extend_from_slice(&chunk);
            }

            if data.is_empty() {
                 return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Empty file" }))).into_response();
            }

            // Ensure uploads directory exists
            let upload_dir = "uploads/avatars";
            if let Err(e) = fs::create_dir_all(upload_dir).await {
                eprintln!("Failed to create upload dir: {}", e);
                 return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Server storage error" }))).into_response();
            }

            // Generate unique filename
            let ext = Path::new(&file_name).extension().and_then(|s| s.to_str()).unwrap_or("png");
            let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
            let filepath = format!("{}/{}", upload_dir, new_filename);

            // Save file
            if let Err(e) = fs::write(&filepath, data).await {
                 eprintln!("Failed to write file: {}", e);
                 return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to save file" }))).into_response();
            }

            // Return URL (assuming static file serving is set up at /uploads)
            let public_url = format!("/uploads/avatars/{}", new_filename);
            return (StatusCode::OK, Json(serde_json::json!({ "url": public_url }))).into_response();
        }
    }

    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "No file field found" }))).into_response()
}

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::post;
    axum::Router::new()
        .route("/api/upload", post(upload_handler))
}

