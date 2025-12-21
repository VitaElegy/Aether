use axum::{
    Json, extract::Multipart, response::IntoResponse, http::StatusCode,
};
use crate::interface::api::auth::AuthenticatedUser;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

pub async fn upload_handler(
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

            // Validate image
            if !content_type.starts_with("image/") {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Only images are allowed" }))).into_response();
            }

            let data = field.bytes().await.unwrap();

            // Ensure uploads directory exists
            let upload_dir = "uploads/avatars";
            fs::create_dir_all(upload_dir).await.unwrap();

            // Generate unique filename
            let ext = Path::new(&file_name).extension().and_then(|s| s.to_str()).unwrap_or("png");
            let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
            let filepath = format!("{}/{}", upload_dir, new_filename);

            // Save file
            fs::write(&filepath, data).await.unwrap();

            // Return URL (assuming static file serving is set up at /uploads)
            let public_url = format!("/uploads/avatars/{}", new_filename);
            return (StatusCode::OK, Json(serde_json::json!({ "url": public_url }))).into_response();
        }
    }

    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "No file provided" }))).into_response()
}

