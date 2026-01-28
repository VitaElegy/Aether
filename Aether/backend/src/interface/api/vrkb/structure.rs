use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use std::sync::Arc;
use serde_json::Value;

use crate::domain::graph::computed_tree::ComputedTreeService;
use crate::infrastructure::persistence::repositories::block_repository::BlockRepository;
use crate::interface::state::AppState;

// GET /api/kb/:id/structure
pub async fn get_kb_structure(
    Path(kb_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Value>>, (axum::http::StatusCode, String)> {
    // 1. Fetch all raw blocks
    let repo = BlockRepository::new(state.repo.db.clone());
    let blocks = repo.find_by_kb_id(kb_id).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Compute Topology
    // Note: In efficient production, this should be cached or computed on write.
    // For V2 MVP, we compute on read.
    let sorted_blocks = ComputedTreeService::compute_topological_sort(blocks);

    // 3. Serialize
    // Convert to JSON Value for response
    // Ideally we return DTOs, but here we reuse the Block model which is Serialize
    let json_blocks = sorted_blocks.into_iter()
        .map(|b| serde_json::to_value(b).unwrap())
        .collect();

    Ok(Json(json_blocks))
}
