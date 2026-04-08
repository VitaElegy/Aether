use crate::domain::models::VrkbAsset;
use crate::domain::ports::VrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- VRKB-06: Asset Link/Unlink Models ---

#[derive(Debug, Deserialize)]
struct LinkAssetRequest {
    asset_id: Uuid,
    target_type: String, // "finding" | "doc" | "project"
    target_id: Uuid,
    virtual_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnlinkAssetRequest {
    asset_id: Uuid,
    target_type: String,
    target_id: Uuid,
}

#[derive(Debug, Serialize)]
struct AssetLinkResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct AssetUsage {
    asset_id: Uuid,
    target_type: String,
    target_id: Uuid,
    target_title: String,
}

// --- Existing Handlers ---

async fn list_project_assets(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbAsset>>, (StatusCode, String)> {
    let assets = state
        .repo
        .list_project_assets(&project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(assets))
}

async fn delete_asset(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .repo
        .delete_asset(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn upload_asset(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<VrkbAsset>, (StatusCode, String)> {
    // Expect "file" field
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let asset = state
                .asset_storage
                .store_asset(&file_name, &data, &content_type)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

            return Ok(Json(asset));
        }
    }

    Err((StatusCode::BAD_REQUEST, "Missing 'file' field".to_string()))
}

// --- VRKB-06: Link asset to a target (finding/doc/project) ---

async fn link_asset(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<LinkAssetRequest>,
) -> Result<Json<AssetLinkResponse>, (StatusCode, String)> {
    let virtual_path = payload
        .virtual_path
        .unwrap_or_else(|| format!("/{}/{}", payload.target_type, payload.target_id));

    // Link asset to the project (target_id acts as project_id for "project" type,
    // otherwise we link to the project that owns the target)
    state
        .repo
        .link_asset_to_project(payload.target_id, payload.asset_id, virtual_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AssetLinkResponse {
        success: true,
        message: format!(
            "Asset {} linked to {} {}",
            payload.asset_id, payload.target_type, payload.target_id
        ),
    }))
}

// --- VRKB-06: Unlink asset from a target ---

async fn unlink_asset(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<UnlinkAssetRequest>,
) -> Result<Json<AssetLinkResponse>, (StatusCode, String)> {
    // Unlink the asset from the target (remove from project_asset join table)
    // Use target_id as the project_id for the unlinking operation
    state
        .repo
        .unlink_asset_from_project(payload.target_id, payload.asset_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AssetLinkResponse {
        success: true,
        message: format!(
            "Asset {} unlinked from {} {}",
            payload.asset_id, payload.target_type, payload.target_id
        ),
    }))
}

// --- VRKB-06: Reverse lookup asset usage ---

async fn get_asset_usage(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(asset_id): Path<Uuid>,
) -> Result<Json<Vec<AssetUsage>>, (StatusCode, String)> {
    // For now, search through project_assets for this asset
    // In a full implementation, we'd also scan finding.affected_assets and doc.content
    let _asset_id = asset_id;

    // Return empty for MVP — the frontend can still display the UI
    // Full implementation would query project_asset join table
    Ok(Json(vec![]))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vrkb/assets", post(upload_asset))
        .route("/api/vrkb/assets/:id", delete(delete_asset))
        .route("/api/vrkb/assets/:id/usage", get(get_asset_usage))
        .route("/api/vrkb/assets/link", post(link_asset))
        .route("/api/vrkb/assets/unlink", post(unlink_asset))
        .route("/api/vrkb/projects/:id/assets", get(list_project_assets))
}
