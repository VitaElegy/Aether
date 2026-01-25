use axum::{
    extract::{Multipart, State, Path},
    http::StatusCode,
    response::{Json},
    routing::{post, get, delete},
    Router,
};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::models::VrkbAsset;
use crate::domain::ports::VrkbRepository;

async fn list_project_assets(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbAsset>>, (StatusCode, String)> {
    let assets = state.repo.list_project_assets(&project_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(assets))
}

async fn delete_asset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.delete_asset(&id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn upload_asset(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<VrkbAsset>, (StatusCode, String)> {
    // Expect "file" field
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let asset = state.asset_storage.store_asset(&file_name, &data, &content_type)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
                
            return Ok(Json(asset));
        }
    }

    Err((StatusCode::BAD_REQUEST, "Missing 'file' field".to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/assets", post(upload_asset))
        .route("/api/vrkb/assets/:id", delete(delete_asset))
        .route("/api/vrkb/projects/:id/assets", get(list_project_assets))
}
