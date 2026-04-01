use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::math::models::*;
use crate::domain::math::portability;
use crate::interface::state::AppState;

// ── Request / Response DTOs ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AddNodeRequest {
    pub node_type: MathNodeType,
    pub label: String,
    pub content: String,
    pub ref_label: Option<String>,
    pub equation_label: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateNodeRequest {
    pub label: Option<String>,
    pub content: Option<String>,
    pub proof_status: Option<ProofStatus>,
    pub ref_label: Option<String>,
    pub equation_label: Option<String>,
}

#[derive(Deserialize)]
pub struct AddRelationRequest {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relation_type: MathRelationType,
    pub annotation: Option<String>,
}

#[derive(Deserialize)]
pub struct WorkspaceCommandRequest {
    #[serde(flatten)]
    pub command: WorkspaceCommand,
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub format: MathExportFormat,
}

#[derive(Deserialize)]
pub struct ImportRequest {
    pub content: String,
    pub collision_strategy: Option<LabelCollisionStrategy>,
}

#[derive(Serialize)]
pub struct GraphOverview {
    pub kb_id: Uuid,
    pub node_count: usize,
    pub relation_count: usize,
    pub node_types: Vec<NodeTypeCount>,
    pub has_cycles: bool,
    pub incomplete_proofs: usize,
}

#[derive(Serialize)]
pub struct NodeTypeCount {
    pub node_type: MathNodeType,
    pub count: usize,
}

// ── Router ──────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        // Graph overview
        .route("/api/math/:kb_id/graph", get(get_graph))
        .route("/api/math/:kb_id/overview", get(get_overview))
        // Node CRUD
        .route("/api/math/:kb_id/nodes", post(add_node))
        .route("/api/math/:kb_id/nodes/:node_id", get(get_node))
        .route("/api/math/:kb_id/nodes/:node_id", put(update_node))
        .route("/api/math/:kb_id/nodes/:node_id", delete(remove_node))
        // Relation CRUD
        .route("/api/math/:kb_id/relations", post(add_relation))
        .route(
            "/api/math/:kb_id/relations/:relation_id",
            delete(remove_relation),
        )
        // Graph semantics (MATH-02)
        .route(
            "/api/math/:kb_id/nodes/:node_id/inspect",
            get(inspect_node),
        )
        .route(
            "/api/math/:kb_id/dependencies",
            get(analyze_dependencies),
        )
        // Workspace commands (MATH-03)
        .route(
            "/api/math/:kb_id/workspace/command",
            post(workspace_command),
        )
        // Reference validation (MATH-05)
        .route(
            "/api/math/:kb_id/references/validate",
            get(validate_references),
        )
        // Portability (MATH-06)
        .route("/api/math/:kb_id/export", post(export_graph))
        .route("/api/math/:kb_id/import", post(import_graph))
}

// ── Handlers ────────────────────────────────────────────────────────────

async fn get_graph(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
) -> Json<MathGraph> {
    let graph = state.math_service.get_graph(kb_id);
    Json(graph)
}

async fn get_overview(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
) -> Json<GraphOverview> {
    let graph = state.math_service.get_graph(kb_id);
    let analysis = graph.analyze_dependencies();

    let mut type_counts = std::collections::HashMap::new();
    for node in &graph.nodes {
        *type_counts.entry(node.node_type).or_insert(0usize) += 1;
    }

    let node_types: Vec<NodeTypeCount> = MathNodeType::all()
        .iter()
        .map(|t| NodeTypeCount {
            node_type: *t,
            count: *type_counts.get(t).unwrap_or(&0),
        })
        .collect();

    let incomplete_proofs = graph
        .nodes
        .iter()
        .filter(|n| n.proof_status == Some(ProofStatus::Incomplete))
        .count();

    Json(GraphOverview {
        kb_id,
        node_count: graph.nodes.len(),
        relation_count: graph.relations.len(),
        node_types,
        has_cycles: !analysis.cycles.is_empty(),
        incomplete_proofs,
    })
}

async fn add_node(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
    Json(req): Json<AddNodeRequest>,
) -> Result<Json<MathNode>, StatusCode> {
    let node = state.math_service.add_node(
        kb_id,
        req.node_type,
        req.label,
        req.content,
        req.ref_label,
        req.equation_label,
    );
    Ok(Json(node))
}

async fn get_node(
    State(state): State<AppState>,
    Path((kb_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MathNode>, StatusCode> {
    let graph = state.math_service.get_graph(kb_id);
    graph
        .nodes
        .into_iter()
        .find(|n| n.id == node_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_node(
    State(state): State<AppState>,
    Path((kb_id, node_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<MathNode>, StatusCode> {
    state
        .math_service
        .update_node(
            kb_id,
            node_id,
            req.label,
            req.content,
            req.proof_status,
            req.ref_label,
            req.equation_label,
        )
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn remove_node(
    State(state): State<AppState>,
    Path((kb_id, node_id)): Path<(Uuid, Uuid)>,
) -> StatusCode {
    if state.math_service.remove_node(kb_id, node_id) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn add_relation(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
    Json(req): Json<AddRelationRequest>,
) -> Result<Json<MathRelation>, StatusCode> {
    state
        .math_service
        .add_relation(
            kb_id,
            req.source_id,
            req.target_id,
            req.relation_type,
            req.annotation,
        )
        .map(Json)
        .ok_or(StatusCode::BAD_REQUEST)
}

async fn remove_relation(
    State(state): State<AppState>,
    Path((kb_id, relation_id)): Path<(Uuid, Uuid)>,
) -> StatusCode {
    if state.math_service.remove_relation(kb_id, relation_id) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn inspect_node(
    State(state): State<AppState>,
    Path((kb_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<NodeInspection>, StatusCode> {
    state
        .math_service
        .inspect_node(kb_id, node_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn analyze_dependencies(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
) -> Json<DependencyAnalysis> {
    Json(state.math_service.analyze_dependencies(kb_id))
}

async fn workspace_command(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
    Json(req): Json<WorkspaceCommandRequest>,
) -> Json<WorkspaceResult> {
    Json(state.math_service.execute_workspace_command(kb_id, req.command))
}

async fn validate_references(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
) -> Json<ReferenceValidation> {
    Json(state.math_service.validate_references(kb_id))
}

async fn export_graph(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Json<MathExportResult> {
    let graph = state.math_service.get_graph(kb_id);
    Json(portability::export_graph(&graph, req.format))
}

async fn import_graph(
    State(state): State<AppState>,
    Path(kb_id): Path<Uuid>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<MathImportResult>, StatusCode> {
    let existing = state.math_service.get_graph(kb_id);
    let strategy = req.collision_strategy.unwrap_or_default();

    match portability::import_graph(kb_id, &req.content, &existing, strategy) {
        Ok((new_graph, result)) => {
            state.math_service.set_graph(new_graph);
            Ok(Json(result))
        }
        Err(e) => {
            tracing::error!("Math import failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
