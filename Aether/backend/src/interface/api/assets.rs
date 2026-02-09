use axum::{
    extract::{Path, State, Multipart, Query},
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
pub struct AssetQuery {
    context: Option<Uuid>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(upload_asset))
        .route("/:id", get(get_asset))
}

async fn upload_asset(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = user.id;

    // We only process the first field named "file"
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let filename = field.file_name().unwrap_or("unnamed").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let node = state.asset_manager.upload_asset(
                user_id,
                filename,
                content_type,
                &data
            ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

            return Ok(Json(node));
        }
    }

    Err((StatusCode::BAD_REQUEST, "No file field found".to_string()))
}

async fn get_asset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<AssetQuery>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = user.id;

    let (path, mime_type) = state.asset_manager.get_asset_file(id, query.context, user_id).await
        .map_err(|e| (StatusCode::FORBIDDEN, e))?; // Assuming most errors are permission/not found

    // Open file
    let file = File::open(&path).await
        .map_err(|_| (StatusCode::NOT_FOUND, "File not found on disk".to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, mime_type),
        (header::CACHE_CONTROL, "public, max-age=31536000".to_string()), // Cache immutable assets
    ];

    Ok((headers, body))
}
