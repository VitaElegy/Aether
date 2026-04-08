use super::models::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// MATH-06: Math Portability - Export and Import operations.

// ── JSON Graph Export/Import ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonGraphPayload {
    version: String,
    kb_id: String,
    nodes: Vec<MathNode>,
    relations: Vec<MathRelation>,
}

pub fn export_json_graph(graph: &MathGraph) -> MathExportResult {
    let payload = JsonGraphPayload {
        version: "1.0".into(),
        kb_id: graph.kb_id.to_string(),
        nodes: graph.nodes.clone(),
        relations: graph.relations.clone(),
    };

    let content = serde_json::to_string_pretty(&payload).unwrap_or_default();
    MathExportResult {
        format: MathExportFormat::JsonGraph,
        content,
        node_count: graph.nodes.len(),
        relation_count: graph.relations.len(),
    }
}

pub fn import_json_graph(
    kb_id: Uuid,
    json_content: &str,
    existing_graph: &MathGraph,
    collision_strategy: LabelCollisionStrategy,
) -> Result<(MathGraph, MathImportResult), String> {
    let payload: JsonGraphPayload =
        serde_json::from_str(json_content).map_err(|e| format!("Invalid JSON: {}", e))?;

    merge_graphs(kb_id, existing_graph, &payload.nodes, &payload.relations, collision_strategy)
}

// ── Markdown Manuscript Export ───────────────────────────────────────────

pub fn export_markdown_manuscript(graph: &MathGraph) -> MathExportResult {
    let mut md = String::new();
    md.push_str("# Mathematical Manuscript\n\n");
    md.push_str(&format!("*Knowledge Base: {}*\n\n", graph.kb_id));
    md.push_str("---\n\n");

    // Group nodes by type
    let type_order = [
        MathNodeType::Definition,
        MathNodeType::Theorem,
        MathNodeType::Lemma,
        MathNodeType::Proposition,
        MathNodeType::Corollary,
        MathNodeType::Proof,
        MathNodeType::Example,
        MathNodeType::Problem,
        MathNodeType::Note,
    ];

    for node_type in &type_order {
        let nodes: Vec<&MathNode> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == *node_type)
            .collect();

        if nodes.is_empty() {
            continue;
        }

        md.push_str(&format!("## {}\n\n", node_type.label()));

        for node in nodes {
            // Label
            if let Some(ref rl) = node.ref_label {
                md.push_str(&format!(
                    "**{} {}** \\label{{{}}}",
                    node_type.label(),
                    node.label,
                    rl
                ));
            } else {
                md.push_str(&format!("**{} {}**", node_type.label(), node.label));
            }
            md.push('\n');

            // Content
            if !node.content.is_empty() {
                md.push_str(&node.content);
                md.push_str("\n\n");
            }

            // Proof status
            if let Some(ps) = &node.proof_status {
                let status_str = match ps {
                    ProofStatus::Complete => "✓ Complete",
                    ProofStatus::Incomplete => "⚠ Incomplete",
                    ProofStatus::Sketch => "✎ Sketch",
                };
                md.push_str(&format!("*Status: {}*\n\n", status_str));
            }

            md.push_str("---\n\n");
        }
    }

    // Relations summary
    if !graph.relations.is_empty() {
        md.push_str("## Relations\n\n");
        md.push_str("| Source | Relation | Target |\n");
        md.push_str("|--------|----------|--------|\n");

        let node_map: HashMap<Uuid, &str> =
            graph.nodes.iter().map(|n| (n.id, n.label.as_str())).collect();

        for rel in &graph.relations {
            let source = node_map.get(&rel.source_id).unwrap_or(&"?");
            let target = node_map.get(&rel.target_id).unwrap_or(&"?");
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                source,
                rel.relation_type.label(),
                target
            ));
        }
        md.push('\n');
    }

    MathExportResult {
        format: MathExportFormat::MarkdownManuscript,
        content: md,
        node_count: graph.nodes.len(),
        relation_count: graph.relations.len(),
    }
}

// ── LaTeX Package Export ────────────────────────────────────────────────

pub fn export_latex_package(graph: &MathGraph) -> MathExportResult {
    let mut tex = String::new();

    tex.push_str("\\documentclass{article}\n");
    tex.push_str("\\usepackage{amsmath,amsthm,amssymb}\n");
    tex.push_str("\\usepackage{hyperref}\n\n");

    // Theorem environments
    tex.push_str("\\newtheorem{theorem}{Theorem}[section]\n");
    tex.push_str("\\newtheorem{lemma}[theorem]{Lemma}\n");
    tex.push_str("\\newtheorem{proposition}[theorem]{Proposition}\n");
    tex.push_str("\\newtheorem{corollary}[theorem]{Corollary}\n");
    tex.push_str("\\theoremstyle{definition}\n");
    tex.push_str("\\newtheorem{definition}[theorem]{Definition}\n");
    tex.push_str("\\newtheorem{example}[theorem]{Example}\n");
    tex.push_str("\\newtheorem{problem}[theorem]{Problem}\n");
    tex.push_str("\\theoremstyle{remark}\n");
    tex.push_str("\\newtheorem{remark}[theorem]{Remark}\n\n");

    tex.push_str("\\begin{document}\n\n");
    tex.push_str("\\title{Mathematical Knowledge Base}\n\\date{\\today}\n\\maketitle\n\n");

    let type_env_map: HashMap<MathNodeType, &str> = [
        (MathNodeType::Theorem, "theorem"),
        (MathNodeType::Lemma, "lemma"),
        (MathNodeType::Definition, "definition"),
        (MathNodeType::Proposition, "proposition"),
        (MathNodeType::Corollary, "corollary"),
        (MathNodeType::Proof, "proof"),
        (MathNodeType::Example, "example"),
        (MathNodeType::Problem, "problem"),
        (MathNodeType::Note, "remark"),
    ]
    .into_iter()
    .collect();

    for node in &graph.nodes {
        let env = type_env_map
            .get(&node.node_type)
            .unwrap_or(&"remark");

        if node.node_type == MathNodeType::Proof {
            tex.push_str("\\begin{proof}\n");
            if let Some(ref rl) = node.ref_label {
                tex.push_str(&format!("\\label{{{}}}\n", rl));
            }
            tex.push_str(&node.content);
            tex.push_str("\n\\end{proof}\n\n");
        } else {
            if let Some(ref rl) = node.ref_label {
                tex.push_str(&format!(
                    "\\begin{{{}}}[{}]\\label{{{}}}\n",
                    env, node.label, rl
                ));
            } else {
                tex.push_str(&format!("\\begin{{{}}}[{}]\n", env, node.label));
            }
            tex.push_str(&node.content);
            tex.push_str(&format!("\n\\end{{{}}}\n\n", env));
        }
    }

    tex.push_str("\\end{document}\n");

    MathExportResult {
        format: MathExportFormat::LatexPackage,
        content: tex,
        node_count: graph.nodes.len(),
        relation_count: graph.relations.len(),
    }
}

// ── Graph Merge Logic ───────────────────────────────────────────────────

fn merge_graphs(
    kb_id: Uuid,
    existing: &MathGraph,
    incoming_nodes: &[MathNode],
    incoming_relations: &[MathRelation],
    strategy: LabelCollisionStrategy,
) -> Result<(MathGraph, MathImportResult), String> {
    let mut result_graph = existing.clone();
    result_graph.kb_id = kb_id;

    let existing_labels: HashMap<String, Uuid> = existing
        .nodes
        .iter()
        .filter_map(|n| n.ref_label.as_ref().map(|l| (l.clone(), n.id)))
        .collect();

    let mut id_remap: HashMap<Uuid, Uuid> = HashMap::new();
    let mut nodes_created = 0;
    let mut nodes_skipped = 0;
    let mut nodes_renamed = 0;
    let mut collisions = Vec::new();

    for node in incoming_nodes {
        let has_collision = node
            .ref_label
            .as_ref()
            .map(|l| existing_labels.contains_key(l))
            .unwrap_or(false);

        if has_collision {
            let label = node.ref_label.as_ref().unwrap().clone();
            collisions.push(label.clone());

            match strategy {
                LabelCollisionStrategy::Skip => {
                    nodes_skipped += 1;
                    // Map old ID to existing node ID
                    if let Some(existing_id) = existing_labels.get(&label) {
                        id_remap.insert(node.id, *existing_id);
                    }
                    continue;
                }
                LabelCollisionStrategy::Rename => {
                    let new_label = format!("{}_imported", label);
                    let mut new_node = node.clone();
                    new_node.id = Uuid::new_v4();
                    new_node.kb_id = kb_id;
                    new_node.ref_label = Some(new_label);
                    id_remap.insert(node.id, new_node.id);
                    result_graph.nodes.push(new_node);
                    nodes_renamed += 1;
                    nodes_created += 1;
                }
                LabelCollisionStrategy::Overwrite => {
                    // Remove existing node with that label
                    if let Some(existing_id) = existing_labels.get(&label) {
                        result_graph.nodes.retain(|n| n.id != *existing_id);
                        result_graph
                            .relations
                            .retain(|r| r.source_id != *existing_id && r.target_id != *existing_id);
                    }
                    let mut new_node = node.clone();
                    new_node.id = Uuid::new_v4();
                    new_node.kb_id = kb_id;
                    id_remap.insert(node.id, new_node.id);
                    result_graph.nodes.push(new_node);
                    nodes_created += 1;
                }
            }
        } else {
            let mut new_node = node.clone();
            new_node.id = Uuid::new_v4();
            new_node.kb_id = kb_id;
            id_remap.insert(node.id, new_node.id);
            result_graph.nodes.push(new_node);
            nodes_created += 1;
        }
    }

    // Remap relations
    let mut relations_created = 0;
    for rel in incoming_relations {
        let source = id_remap.get(&rel.source_id);
        let target = id_remap.get(&rel.target_id);

        if let (Some(&new_source), Some(&new_target)) = (source, target) {
            let new_rel = MathRelation {
                id: Uuid::new_v4(),
                kb_id,
                source_id: new_source,
                target_id: new_target,
                relation_type: rel.relation_type,
                annotation: rel.annotation.clone(),
                created_at: chrono::Utc::now(),
            };
            result_graph.relations.push(new_rel);
            relations_created += 1;
        }
    }

    let import_result = MathImportResult {
        nodes_created,
        nodes_skipped,
        nodes_renamed,
        relations_created,
        collisions,
    };

    Ok((result_graph, import_result))
}

/// Dispatch export by format.
pub fn export_graph(graph: &MathGraph, format: MathExportFormat) -> MathExportResult {
    match format {
        MathExportFormat::JsonGraph => export_json_graph(graph),
        MathExportFormat::MarkdownManuscript => export_markdown_manuscript(graph),
        MathExportFormat::LatexPackage => export_latex_package(graph),
    }
}

/// Dispatch import (currently only JSON supported).
pub fn import_graph(
    kb_id: Uuid,
    content: &str,
    existing: &MathGraph,
    collision_strategy: LabelCollisionStrategy,
) -> Result<(MathGraph, MathImportResult), String> {
    import_json_graph(kb_id, content, existing, collision_strategy)
}
