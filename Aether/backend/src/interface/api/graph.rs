use axum::{
    extract::{Query, State},
    Json, Router, routing::get,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use crate::interface::state::AppState;
use crate::infrastructure::persistence::entities::{semantic_node};

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

// --- Manual Graph Editing API ---

#[derive(Deserialize)]
pub struct CreateNodeRequest {
    kb_id: Uuid,
    parent_id: Option<Uuid>,
    label: String,
    data: serde_json::Value,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/graph/context", get(get_context))
        .route("/api/graph/global", get(get_global))
        // New Manual Editing Routes
        .route("/api/kb/:id/graph", get(get_kb_graph))
        .route("/api/nodes", axum::routing::post(create_node))
        .route("/api/nodes/:id", axum::routing::delete(delete_node))
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

async fn get_kb_graph(
    State(state): State<AppState>,
    axum::extract::Path(kb_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<crate::domain::models::GraphNode>>, StatusCode> {
    let nodes = state.graph_service.get_knowledge_tree(kb_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch graph: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(nodes))
}

async fn create_node(
    State(state): State<AppState>,
    Json(payload): Json<CreateNodeRequest>,
) -> Result<Json<Uuid>, StatusCode> {
    let id = state.graph_service.add_node(
        payload.kb_id,
        payload.parent_id,
        payload.label,
        payload.data,
    ).await.map_err(|e| {
        tracing::error!("Failed to create node: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(id))
}

async fn delete_node(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.graph_service.delete_node(id).await
        .map_err(|e| {
            tracing::error!("Failed to delete node: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::OK)
}
