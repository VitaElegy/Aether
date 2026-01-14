use axum::{
    extract::{Query, State},
    Json, Router, routing::get,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use crate::interface::state::AppState;
use crate::infrastructure::persistence::entities::{semantic_node, semantic_edge};

#[derive(Debug, Deserialize)]
pub struct ContextParams {
    article_id: Uuid,
}

#[derive(Serialize)]
pub struct HelperNode {
    id: Uuid,
    client_id: String,
    title: Option<String>,
    content: Option<String>,
    r#type: String,
    metrics: serde_json::Value,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/graph/context", get(get_context))
        .route("/api/graph/global", get(get_global))
}

async fn get_context(
    State(state): State<AppState>,
    Query(params): Query<ContextParams>,
) -> Result<Json<Vec<HelperNode>>, StatusCode> {
    let nodes = semantic_node::Entity::find()
        .filter(semantic_node::Column::ArticleId.eq(params.article_id))
        .all(&state.repo.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let helper_nodes = nodes.into_iter().map(|n| HelperNode {
        id: n.id,
        client_id: n.client_id,
        title: n.title,
        content: n.content,
        r#type: n.r#type,
        metrics: n.metrics,
    }).collect();

    Ok(Json(helper_nodes))
}

async fn get_global(
    State(state): State<AppState>,
) -> Result<Json<Vec<HelperNode>>, StatusCode> {
    let nodes = semantic_node::Entity::find()
        .all(&state.repo.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let helper_nodes = nodes.into_iter().map(|n| HelperNode {
        id: n.id,
        client_id: n.client_id,
        title: n.title,
        content: n.content,
        r#type: n.r#type,
        metrics: n.metrics,
    }).collect();

    Ok(Json(helper_nodes))
}
