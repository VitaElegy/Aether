use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

// ── MATH-01: Formal Object Model ────────────────────────────────────────

/// The nine canonical node types for a formal mathematics knowledge base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathNodeType {
    Theorem,
    Lemma,
    Definition,
    Proposition,
    Corollary,
    Proof,
    Example,
    Problem,
    Note,
}

impl MathNodeType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Theorem => "Theorem",
            Self::Lemma => "Lemma",
            Self::Definition => "Definition",
            Self::Proposition => "Proposition",
            Self::Corollary => "Corollary",
            Self::Proof => "Proof",
            Self::Example => "Example",
            Self::Problem => "Problem",
            Self::Note => "Note",
        }
    }

    pub fn all() -> &'static [MathNodeType] {
        &[
            Self::Theorem,
            Self::Lemma,
            Self::Definition,
            Self::Proposition,
            Self::Corollary,
            Self::Proof,
            Self::Example,
            Self::Problem,
            Self::Note,
        ]
    }
}

/// The five canonical relation types between math nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathRelationType {
    DependsOn,
    Proves,
    UsesDefinition,
    Generalizes,
    SpecialCaseOf,
}

impl MathRelationType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DependsOn => "depends on",
            Self::Proves => "proves",
            Self::UsesDefinition => "uses definition",
            Self::Generalizes => "generalizes",
            Self::SpecialCaseOf => "is special case of",
        }
    }

    pub fn inverse(&self) -> Option<MathRelationType> {
        match self {
            Self::Generalizes => Some(Self::SpecialCaseOf),
            Self::SpecialCaseOf => Some(Self::Generalizes),
            Self::Proves => None, // "is proved by" not modelled as separate type
            Self::DependsOn => None,
            Self::UsesDefinition => None,
        }
    }

    pub fn all() -> &'static [MathRelationType] {
        &[
            Self::DependsOn,
            Self::Proves,
            Self::UsesDefinition,
            Self::Generalizes,
            Self::SpecialCaseOf,
        ]
    }
}

// ── MATH-05: Formula Labels and References ──────────────────────────────

/// Status of a proof node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofStatus {
    Complete,
    Incomplete,
    Sketch,
}

impl Default for ProofStatus {
    fn default() -> Self {
        Self::Incomplete
    }
}

/// A formal math node (theorem, lemma, definition, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathNode {
    pub id: Uuid,
    pub kb_id: Uuid,
    pub node_type: MathNodeType,
    /// A human-readable label (e.g., "Theorem 3.1", "Definition of Continuity")
    pub label: String,
    /// A short machine-friendly label for cross-references (e.g., "thm:bolzano", "def:cont")
    pub ref_label: Option<String>,
    /// LaTeX or markdown content
    pub content: String,
    /// Proof status (relevant for proof nodes, optional otherwise)
    pub proof_status: Option<ProofStatus>,
    /// Equation/formula label for referencing (e.g., "eq:euler")
    pub equation_label: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A directed relation between two math nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathRelation {
    pub id: Uuid,
    pub kb_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relation_type: MathRelationType,
    /// Optional annotation on the edge (e.g., "by induction", "via contrapositive")
    pub annotation: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── MATH-02: Graph Semantics ────────────────────────────────────────────

/// A complete math knowledge graph for a knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathGraph {
    pub kb_id: Uuid,
    pub nodes: Vec<MathNode>,
    pub relations: Vec<MathRelation>,
}

/// Result of inspecting a single node's relations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInspection {
    pub node: MathNode,
    pub incoming: Vec<MathRelation>,
    pub outgoing: Vec<MathRelation>,
    pub dependency_depth: usize,
}

/// Result of dependency graph analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// Nodes that the target depends on (transitive closure)
    pub prerequisites: Vec<Uuid>,
    /// Topologically sorted node ordering
    pub topological_order: Vec<Uuid>,
    /// Circular dependency cycles, if any
    pub cycles: Vec<Vec<Uuid>>,
    /// Nodes whose prerequisites are not yet in the graph
    pub unresolved_prerequisites: Vec<UnresolvedPrerequisite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedPrerequisite {
    pub node_id: Uuid,
    pub node_label: String,
    pub missing_ref: String,
}

// ── MATH-03: Workspace Mode ─────────────────────────────────────────────

/// A workspace action command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WorkspaceCommand {
    AddNode {
        node_type: MathNodeType,
        label: String,
        content: String,
        ref_label: Option<String>,
        equation_label: Option<String>,
    },
    AddRelation {
        source_id: Uuid,
        target_id: Uuid,
        relation_type: MathRelationType,
        annotation: Option<String>,
    },
    RemoveNode {
        node_id: Uuid,
    },
    RemoveRelation {
        relation_id: Uuid,
    },
    UpdateNode {
        node_id: Uuid,
        label: Option<String>,
        content: Option<String>,
        proof_status: Option<ProofStatus>,
        ref_label: Option<String>,
        equation_label: Option<String>,
    },
    MarkIncompleteProof {
        node_id: Uuid,
    },
    HighlightBlockers {
        node_id: Uuid,
    },
}

/// Result of a workspace command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceResult {
    pub success: bool,
    pub message: String,
    /// Affected node/relation IDs
    pub affected_ids: Vec<Uuid>,
    /// For HighlightBlockers: the blocker node IDs
    pub blockers: Option<Vec<Uuid>>,
}

// ── MATH-04: Workspace Modes ────────────────────────────────────────────

/// The three workspace modes for a math knowledge base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathWorkspaceMode {
    /// Writing mode: create and edit mathematical content
    Manuscript,
    /// Reading/archive mode: browse and search existing content
    Archive,
    /// Graph workspace: manipulate relations and dependencies
    Workspace,
}

impl Default for MathWorkspaceMode {
    fn default() -> Self {
        Self::Manuscript
    }
}

// ── MATH-05: Reference Validation ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceValidation {
    /// All theorem references found
    pub theorem_refs: Vec<RefEntry>,
    /// All definition references found
    pub definition_refs: Vec<RefEntry>,
    /// All equation labels found
    pub equation_labels: Vec<RefEntry>,
    /// References that point to non-existent targets
    pub unresolved_refs: Vec<UnresolvedRef>,
    /// Labels that appear more than once
    pub duplicate_labels: Vec<DuplicateLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefEntry {
    pub source_node_id: Uuid,
    pub ref_label: String,
    pub target_node_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedRef {
    pub source_node_id: Uuid,
    pub source_label: String,
    pub ref_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateLabel {
    pub label: String,
    pub node_ids: Vec<Uuid>,
}

// ── MATH-06: Portability Models ─────────────────────────────────────────

/// Export format for the math knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathExportFormat {
    JsonGraph,
    MarkdownManuscript,
    LatexPackage,
}

/// Import conflict handling strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelCollisionStrategy {
    /// Skip nodes with colliding labels
    Skip,
    /// Rename incoming labels with a suffix
    Rename,
    /// Overwrite existing nodes
    Overwrite,
}

impl Default for LabelCollisionStrategy {
    fn default() -> Self {
        Self::Rename
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathExportResult {
    pub format: MathExportFormat,
    pub content: String,
    pub node_count: usize,
    pub relation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathImportRequest {
    pub content: String,
    pub collision_strategy: LabelCollisionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathImportResult {
    pub nodes_created: usize,
    pub nodes_skipped: usize,
    pub nodes_renamed: usize,
    pub relations_created: usize,
    pub collisions: Vec<String>,
}

// ── Graph Algorithm Implementations ─────────────────────────────────────

impl MathGraph {
    pub fn new(kb_id: Uuid) -> Self {
        Self {
            kb_id,
            nodes: Vec::new(),
            relations: Vec::new(),
        }
    }

    /// Build an adjacency list from relations (source -> targets).
    fn adjacency_list(&self) -> HashMap<Uuid, Vec<(Uuid, &MathRelation)>> {
        let mut adj: HashMap<Uuid, Vec<(Uuid, &MathRelation)>> = HashMap::new();
        for rel in &self.relations {
            adj.entry(rel.source_id)
                .or_default()
                .push((rel.target_id, rel));
        }
        adj
    }

    /// Build a reverse adjacency list (target -> sources).
    #[allow(dead_code)]
    fn reverse_adjacency_list(&self) -> HashMap<Uuid, Vec<(Uuid, &MathRelation)>> {
        let mut adj: HashMap<Uuid, Vec<(Uuid, &MathRelation)>> = HashMap::new();
        for rel in &self.relations {
            adj.entry(rel.target_id)
                .or_default()
                .push((rel.source_id, rel));
        }
        adj
    }

    /// MATH-02: Inspect a node's relations.
    pub fn inspect_node(&self, node_id: Uuid) -> Option<NodeInspection> {
        let node = self.nodes.iter().find(|n| n.id == node_id)?.clone();
        let outgoing: Vec<MathRelation> = self
            .relations
            .iter()
            .filter(|r| r.source_id == node_id)
            .cloned()
            .collect();
        let incoming: Vec<MathRelation> = self
            .relations
            .iter()
            .filter(|r| r.target_id == node_id)
            .cloned()
            .collect();

        // Compute dependency depth via BFS on depends_on relations
        let adj = self.adjacency_list();
        let depth = self.compute_dependency_depth(node_id, &adj);

        Some(NodeInspection {
            node,
            incoming,
            outgoing,
            dependency_depth: depth,
        })
    }

    fn compute_dependency_depth(
        &self,
        node_id: Uuid,
        adj: &HashMap<Uuid, Vec<(Uuid, &MathRelation)>>,
    ) -> usize {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut max_depth = 0;
        queue.push_back((node_id, 0usize));
        visited.insert(node_id);

        while let Some((current, depth)) = queue.pop_front() {
            if let Some(neighbors) = adj.get(&current) {
                for (target, rel) in neighbors {
                    if rel.relation_type == MathRelationType::DependsOn && !visited.contains(target)
                    {
                        visited.insert(*target);
                        let new_depth = depth + 1;
                        if new_depth > max_depth {
                            max_depth = new_depth;
                        }
                        queue.push_back((*target, new_depth));
                    }
                }
            }
        }
        max_depth
    }

    /// MATH-02: Full dependency analysis with cycle detection.
    pub fn analyze_dependencies(&self) -> DependencyAnalysis {
        let node_ids: HashSet<Uuid> = self.nodes.iter().map(|n| n.id).collect();
        let node_map: HashMap<Uuid, &MathNode> = self.nodes.iter().map(|n| (n.id, n)).collect();

        // Build adjacency for depends_on only
        let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();

        for id in &node_ids {
            adj.entry(*id).or_default();
            in_degree.entry(*id).or_insert(0);
        }

        for rel in &self.relations {
            if rel.relation_type == MathRelationType::DependsOn && node_ids.contains(&rel.source_id) && node_ids.contains(&rel.target_id) {
                adj.entry(rel.source_id)
                    .or_default()
                    .push(rel.target_id);
                *in_degree.entry(rel.target_id).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm for topological sort + cycle detection
        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut topological_order = Vec::new();

        while let Some(node) = queue.pop_front() {
            topological_order.push(node);
            if let Some(neighbors) = adj.get(&node) {
                for target in neighbors {
                    if let Some(deg) = in_degree.get_mut(target) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(*target);
                        }
                    }
                }
            }
        }

        // Detect cycles: nodes not in topological order are in cycles
        let ordered_set: HashSet<Uuid> = topological_order.iter().copied().collect();
        let cycle_nodes: Vec<Uuid> = node_ids
            .iter()
            .filter(|id| !ordered_set.contains(id))
            .copied()
            .collect();

        let cycles = if cycle_nodes.is_empty() {
            Vec::new()
        } else {
            self.find_cycles(&cycle_nodes, &adj)
        };

        // Compute transitive prerequisites for all nodes
        let mut prerequisites = Vec::new();
        for rel in &self.relations {
            if rel.relation_type == MathRelationType::DependsOn {
                prerequisites.push(rel.target_id);
            }
        }
        prerequisites.sort();
        prerequisites.dedup();

        // Find unresolved: nodes that reference labels not in the graph
        let unresolved = self.find_unresolved_prerequisites(&node_map);

        DependencyAnalysis {
            prerequisites,
            topological_order,
            cycles,
            unresolved_prerequisites: unresolved,
        }
    }

    /// Find actual cycles using DFS.
    fn find_cycles(
        &self,
        cycle_candidates: &[Uuid],
        adj: &HashMap<Uuid, Vec<Uuid>>,
    ) -> Vec<Vec<Uuid>> {
        let candidate_set: HashSet<Uuid> = cycle_candidates.iter().copied().collect();
        let mut visited = HashSet::new();
        let mut cycles = Vec::new();

        for &start in cycle_candidates {
            if visited.contains(&start) {
                continue;
            }
            // Simple cycle extraction: follow the chain
            let mut path = Vec::new();
            let mut current = start;
            let mut path_set = HashSet::new();

            loop {
                if path_set.contains(&current) {
                    // Found cycle start
                    let cycle_start_idx = path.iter().position(|&n| n == current).unwrap();
                    let cycle: Vec<Uuid> = path[cycle_start_idx..].to_vec();
                    if !cycle.is_empty() {
                        cycles.push(cycle);
                    }
                    break;
                }
                if !candidate_set.contains(&current) {
                    break;
                }
                path.push(current);
                path_set.insert(current);
                visited.insert(current);

                // Follow first neighbor in cycle candidates
                if let Some(neighbors) = adj.get(&current) {
                    if let Some(next) = neighbors.iter().find(|n| candidate_set.contains(n)) {
                        current = *next;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        cycles
    }

    fn find_unresolved_prerequisites(
        &self,
        _node_map: &HashMap<Uuid, &MathNode>,
    ) -> Vec<UnresolvedPrerequisite> {
        let mut unresolved = Vec::new();
        let ref_labels: HashSet<String> = self
            .nodes
            .iter()
            .filter_map(|n| n.ref_label.clone())
            .collect();

        // Scan content for \ref{...} patterns
        let ref_regex = regex::Regex::new(r"\\ref\{([^}]+)\}").unwrap();

        for node in &self.nodes {
            for cap in ref_regex.captures_iter(&node.content) {
                let ref_label = cap[1].to_string();
                if !ref_labels.contains(&ref_label) {
                    unresolved.push(UnresolvedPrerequisite {
                        node_id: node.id,
                        node_label: node.label.clone(),
                        missing_ref: ref_label,
                    });
                }
            }
        }

        unresolved
    }

    // ── MATH-05: Reference Validation ───────────────────────────────────

    pub fn validate_references(&self) -> ReferenceValidation {
        let ref_label_map: HashMap<String, Vec<Uuid>> = {
            let mut map: HashMap<String, Vec<Uuid>> = HashMap::new();
            for node in &self.nodes {
                if let Some(ref lbl) = node.ref_label {
                    map.entry(lbl.clone()).or_default().push(node.id);
                }
                if let Some(ref lbl) = node.equation_label {
                    map.entry(lbl.clone()).or_default().push(node.id);
                }
            }
            map
        };

        let mut theorem_refs = Vec::new();
        let mut definition_refs = Vec::new();
        let mut equation_labels = Vec::new();
        let mut unresolved_refs = Vec::new();

        // Collect refs from node content
        let ref_regex = regex::Regex::new(r"\\ref\{([^}]+)\}").unwrap();
        let eqref_regex = regex::Regex::new(r"\\eqref\{([^}]+)\}").unwrap();

        for node in &self.nodes {
            // Collect ref_label entries
            if let Some(ref lbl) = node.ref_label {
                let entry = RefEntry {
                    source_node_id: node.id,
                    ref_label: lbl.clone(),
                    target_node_id: Some(node.id),
                };
                match node.node_type {
                    MathNodeType::Theorem | MathNodeType::Lemma | MathNodeType::Proposition | MathNodeType::Corollary => {
                        theorem_refs.push(entry);
                    }
                    MathNodeType::Definition => {
                        definition_refs.push(entry);
                    }
                    _ => {}
                }
            }

            if let Some(ref lbl) = node.equation_label {
                equation_labels.push(RefEntry {
                    source_node_id: node.id,
                    ref_label: lbl.clone(),
                    target_node_id: Some(node.id),
                });
            }

            // Scan content for references
            for cap in ref_regex.captures_iter(&node.content) {
                let ref_label = cap[1].to_string();
                if !ref_label_map.contains_key(&ref_label) {
                    unresolved_refs.push(UnresolvedRef {
                        source_node_id: node.id,
                        source_label: node.label.clone(),
                        ref_label,
                    });
                }
            }
            for cap in eqref_regex.captures_iter(&node.content) {
                let ref_label = cap[1].to_string();
                if !ref_label_map.contains_key(&ref_label) {
                    unresolved_refs.push(UnresolvedRef {
                        source_node_id: node.id,
                        source_label: node.label.clone(),
                        ref_label,
                    });
                }
            }
        }

        // Find duplicate labels
        let duplicate_labels: Vec<DuplicateLabel> = ref_label_map
            .into_iter()
            .filter(|(_, ids)| ids.len() > 1)
            .map(|(label, node_ids)| DuplicateLabel { label, node_ids })
            .collect();

        ReferenceValidation {
            theorem_refs,
            definition_refs,
            equation_labels,
            unresolved_refs,
            duplicate_labels,
        }
    }
}
