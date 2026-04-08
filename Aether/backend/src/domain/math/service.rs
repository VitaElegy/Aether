use super::models::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// In-memory math graph service.
/// In a production system this would be backed by a repository trait,
/// but for the Math module we keep the graph in memory per KB and
/// persist via the portability layer (MATH-06).
#[derive(Debug)]
pub struct MathService {
    /// Per-KB graph storage
    graphs: RwLock<HashMap<Uuid, MathGraph>>,
}

impl MathService {
    pub fn new() -> Self {
        Self {
            graphs: RwLock::new(HashMap::new()),
        }
    }

    // ── Graph Access ────────────────────────────────────────────────────

    /// Get or create the graph for a knowledge base.
    pub fn get_graph(&self, kb_id: Uuid) -> MathGraph {
        let graphs = self.graphs.read().expect("lock poisoned");
        graphs
            .get(&kb_id)
            .cloned()
            .unwrap_or_else(|| MathGraph::new(kb_id))
    }

    /// Replace the entire graph for a KB (used by import).
    pub fn set_graph(&self, graph: MathGraph) {
        let mut graphs = self.graphs.write().expect("lock poisoned");
        graphs.insert(graph.kb_id, graph);
    }

    // ── MATH-01 / MATH-03: Node Operations ─────────────────────────────

    pub fn add_node(
        &self,
        kb_id: Uuid,
        node_type: MathNodeType,
        label: String,
        content: String,
        ref_label: Option<String>,
        equation_label: Option<String>,
    ) -> MathNode {
        let node = MathNode {
            id: Uuid::new_v4(),
            kb_id,
            node_type,
            label,
            ref_label,
            content,
            proof_status: if node_type == MathNodeType::Proof {
                Some(ProofStatus::Incomplete)
            } else {
                None
            },
            equation_label,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut graphs = self.graphs.write().expect("lock poisoned");
        let graph = graphs.entry(kb_id).or_insert_with(|| MathGraph::new(kb_id));
        graph.nodes.push(node.clone());
        node
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_node(
        &self,
        kb_id: Uuid,
        node_id: Uuid,
        label: Option<String>,
        content: Option<String>,
        proof_status: Option<ProofStatus>,
        ref_label: Option<String>,
        equation_label: Option<String>,
    ) -> Option<MathNode> {
        let mut graphs = self.graphs.write().expect("lock poisoned");
        let graph = graphs.get_mut(&kb_id)?;
        let node = graph.nodes.iter_mut().find(|n| n.id == node_id)?;

        if let Some(l) = label {
            node.label = l;
        }
        if let Some(c) = content {
            node.content = c;
        }
        if let Some(ps) = proof_status {
            node.proof_status = Some(ps);
        }
        if let Some(rl) = ref_label {
            node.ref_label = Some(rl);
        }
        if let Some(el) = equation_label {
            node.equation_label = Some(el);
        }
        node.updated_at = Utc::now();
        Some(node.clone())
    }

    pub fn remove_node(&self, kb_id: Uuid, node_id: Uuid) -> bool {
        let mut graphs = self.graphs.write().expect("lock poisoned");
        if let Some(graph) = graphs.get_mut(&kb_id) {
            let before = graph.nodes.len();
            graph.nodes.retain(|n| n.id != node_id);
            // Also remove relations involving this node
            graph
                .relations
                .retain(|r| r.source_id != node_id && r.target_id != node_id);
            graph.nodes.len() < before
        } else {
            false
        }
    }

    // ── MATH-01 / MATH-03: Relation Operations ─────────────────────────

    pub fn add_relation(
        &self,
        kb_id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        relation_type: MathRelationType,
        annotation: Option<String>,
    ) -> Option<MathRelation> {
        let mut graphs = self.graphs.write().expect("lock poisoned");
        let graph = graphs.get_mut(&kb_id)?;

        // Verify both nodes exist
        let source_exists = graph.nodes.iter().any(|n| n.id == source_id);
        let target_exists = graph.nodes.iter().any(|n| n.id == target_id);
        if !source_exists || !target_exists {
            return None;
        }

        let relation = MathRelation {
            id: Uuid::new_v4(),
            kb_id,
            source_id,
            target_id,
            relation_type,
            annotation,
            created_at: Utc::now(),
        };

        graph.relations.push(relation.clone());
        Some(relation)
    }

    pub fn remove_relation(&self, kb_id: Uuid, relation_id: Uuid) -> bool {
        let mut graphs = self.graphs.write().expect("lock poisoned");
        if let Some(graph) = graphs.get_mut(&kb_id) {
            let before = graph.relations.len();
            graph.relations.retain(|r| r.id != relation_id);
            graph.relations.len() < before
        } else {
            false
        }
    }

    // ── MATH-02: Graph Semantics ────────────────────────────────────────

    pub fn inspect_node(&self, kb_id: Uuid, node_id: Uuid) -> Option<NodeInspection> {
        let graphs = self.graphs.read().expect("lock poisoned");
        let graph = graphs.get(&kb_id)?;
        graph.inspect_node(node_id)
    }

    pub fn analyze_dependencies(&self, kb_id: Uuid) -> DependencyAnalysis {
        let graphs = self.graphs.read().expect("lock poisoned");
        match graphs.get(&kb_id) {
            Some(graph) => graph.analyze_dependencies(),
            None => DependencyAnalysis {
                prerequisites: Vec::new(),
                topological_order: Vec::new(),
                cycles: Vec::new(),
                unresolved_prerequisites: Vec::new(),
            },
        }
    }

    // ── MATH-03: Workspace Commands ─────────────────────────────────────

    pub fn execute_workspace_command(
        &self,
        kb_id: Uuid,
        command: WorkspaceCommand,
    ) -> WorkspaceResult {
        match command {
            WorkspaceCommand::AddNode {
                node_type,
                label,
                content,
                ref_label,
                equation_label,
            } => {
                let node =
                    self.add_node(kb_id, node_type, label, content, ref_label, equation_label);
                WorkspaceResult {
                    success: true,
                    message: format!("Added {} '{}'", node.node_type.label(), node.label),
                    affected_ids: vec![node.id],
                    blockers: None,
                }
            }
            WorkspaceCommand::AddRelation {
                source_id,
                target_id,
                relation_type,
                annotation,
            } => match self.add_relation(kb_id, source_id, target_id, relation_type, annotation) {
                Some(rel) => WorkspaceResult {
                    success: true,
                    message: format!("Added relation '{}'", relation_type.label()),
                    affected_ids: vec![rel.id],
                    blockers: None,
                },
                None => WorkspaceResult {
                    success: false,
                    message: "Source or target node not found".into(),
                    affected_ids: Vec::new(),
                    blockers: None,
                },
            },
            WorkspaceCommand::RemoveNode { node_id } => {
                let removed = self.remove_node(kb_id, node_id);
                WorkspaceResult {
                    success: removed,
                    message: if removed {
                        "Node removed".into()
                    } else {
                        "Node not found".into()
                    },
                    affected_ids: if removed { vec![node_id] } else { Vec::new() },
                    blockers: None,
                }
            }
            WorkspaceCommand::RemoveRelation { relation_id } => {
                let removed = self.remove_relation(kb_id, relation_id);
                WorkspaceResult {
                    success: removed,
                    message: if removed {
                        "Relation removed".into()
                    } else {
                        "Relation not found".into()
                    },
                    affected_ids: if removed {
                        vec![relation_id]
                    } else {
                        Vec::new()
                    },
                    blockers: None,
                }
            }
            WorkspaceCommand::UpdateNode {
                node_id,
                label,
                content,
                proof_status,
                ref_label,
                equation_label,
            } => match self.update_node(
                kb_id,
                node_id,
                label,
                content,
                proof_status,
                ref_label,
                equation_label,
            ) {
                Some(node) => WorkspaceResult {
                    success: true,
                    message: format!("Updated '{}'", node.label),
                    affected_ids: vec![node.id],
                    blockers: None,
                },
                None => WorkspaceResult {
                    success: false,
                    message: "Node not found".into(),
                    affected_ids: Vec::new(),
                    blockers: None,
                },
            },
            WorkspaceCommand::MarkIncompleteProof { node_id } => {
                match self.update_node(
                    kb_id,
                    node_id,
                    None,
                    None,
                    Some(ProofStatus::Incomplete),
                    None,
                    None,
                ) {
                    Some(_) => WorkspaceResult {
                        success: true,
                        message: "Proof marked as incomplete".into(),
                        affected_ids: vec![node_id],
                        blockers: None,
                    },
                    None => WorkspaceResult {
                        success: false,
                        message: "Node not found".into(),
                        affected_ids: Vec::new(),
                        blockers: None,
                    },
                }
            }
            WorkspaceCommand::HighlightBlockers { node_id } => {
                let graphs = self.graphs.read().expect("lock poisoned");
                if let Some(graph) = graphs.get(&kb_id) {
                    // Find all nodes that this node depends on with incomplete proofs
                    let blockers: Vec<Uuid> = graph
                        .relations
                        .iter()
                        .filter(|r| {
                            r.source_id == node_id
                                && r.relation_type == MathRelationType::DependsOn
                        })
                        .filter_map(|r| {
                            graph.nodes.iter().find(|n| {
                                n.id == r.target_id
                                    && n.proof_status == Some(ProofStatus::Incomplete)
                            })
                        })
                        .map(|n| n.id)
                        .collect();

                    WorkspaceResult {
                        success: true,
                        message: format!("Found {} blockers", blockers.len()),
                        affected_ids: blockers.clone(),
                        blockers: Some(blockers),
                    }
                } else {
                    WorkspaceResult {
                        success: false,
                        message: "Graph not found".into(),
                        affected_ids: Vec::new(),
                        blockers: None,
                    }
                }
            }
        }
    }

    // ── MATH-05: Reference Validation ───────────────────────────────────

    pub fn validate_references(&self, kb_id: Uuid) -> ReferenceValidation {
        let graphs = self.graphs.read().expect("lock poisoned");
        match graphs.get(&kb_id) {
            Some(graph) => graph.validate_references(),
            None => ReferenceValidation {
                theorem_refs: Vec::new(),
                definition_refs: Vec::new(),
                equation_labels: Vec::new(),
                unresolved_refs: Vec::new(),
                duplicate_labels: Vec::new(),
            },
        }
    }
}

impl Clone for MathService {
    fn clone(&self) -> Self {
        let graphs = self.graphs.read().expect("lock poisoned");
        let new_graphs = graphs.clone();
        Self {
            graphs: RwLock::new(new_graphs),
        }
    }
}
