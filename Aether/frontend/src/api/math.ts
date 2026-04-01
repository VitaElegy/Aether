import axios from 'axios';

const API = '/api/math';

// ── Types (MATH-01) ────────────────────────────────────────────────────

export type MathNodeType =
  | 'theorem'
  | 'lemma'
  | 'definition'
  | 'proposition'
  | 'corollary'
  | 'proof'
  | 'example'
  | 'problem'
  | 'note';

export type MathRelationType =
  | 'depends_on'
  | 'proves'
  | 'uses_definition'
  | 'generalizes'
  | 'special_case_of';

export type ProofStatus = 'complete' | 'incomplete' | 'sketch';

export type MathWorkspaceMode = 'manuscript' | 'archive' | 'workspace';

export type MathExportFormat = 'json_graph' | 'markdown_manuscript' | 'latex_package';

export type LabelCollisionStrategy = 'skip' | 'rename' | 'overwrite';

export interface MathNode {
  id: string;
  kb_id: string;
  node_type: MathNodeType;
  label: string;
  ref_label?: string;
  content: string;
  proof_status?: ProofStatus;
  equation_label?: string;
  metadata: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface MathRelation {
  id: string;
  kb_id: string;
  source_id: string;
  target_id: string;
  relation_type: MathRelationType;
  annotation?: string;
  created_at: string;
}

export interface MathGraph {
  kb_id: string;
  nodes: MathNode[];
  relations: MathRelation[];
}

export interface NodeInspection {
  node: MathNode;
  incoming: MathRelation[];
  outgoing: MathRelation[];
  dependency_depth: number;
}

export interface DependencyAnalysis {
  prerequisites: string[];
  topological_order: string[];
  cycles: string[][];
  unresolved_prerequisites: { node_id: string; node_label: string; missing_ref: string }[];
}

export interface GraphOverview {
  kb_id: string;
  node_count: number;
  relation_count: number;
  node_types: { node_type: MathNodeType; count: number }[];
  has_cycles: boolean;
  incomplete_proofs: number;
}

export interface WorkspaceResult {
  success: boolean;
  message: string;
  affected_ids: string[];
  blockers?: string[];
}

export interface ReferenceValidation {
  theorem_refs: { source_node_id: string; ref_label: string; target_node_id?: string }[];
  definition_refs: { source_node_id: string; ref_label: string; target_node_id?: string }[];
  equation_labels: { source_node_id: string; ref_label: string; target_node_id?: string }[];
  unresolved_refs: { source_node_id: string; source_label: string; ref_label: string }[];
  duplicate_labels: { label: string; node_ids: string[] }[];
}

export interface MathExportResult {
  format: MathExportFormat;
  content: string;
  node_count: number;
  relation_count: number;
}

export interface MathImportResult {
  nodes_created: number;
  nodes_skipped: number;
  nodes_renamed: number;
  relations_created: number;
  collisions: string[];
}

// ── API Client ──────────────────────────────────────────────────────────

export const mathApi = {
  // Graph
  async getGraph(kbId: string): Promise<MathGraph> {
    const res = await axios.get(`${API}/${kbId}/graph`);
    return res.data;
  },

  async getOverview(kbId: string): Promise<GraphOverview> {
    const res = await axios.get(`${API}/${kbId}/overview`);
    return res.data;
  },

  // Nodes
  async addNode(
    kbId: string,
    payload: {
      node_type: MathNodeType;
      label: string;
      content: string;
      ref_label?: string;
      equation_label?: string;
    }
  ): Promise<MathNode> {
    const res = await axios.post(`${API}/${kbId}/nodes`, payload);
    return res.data;
  },

  async getNode(kbId: string, nodeId: string): Promise<MathNode> {
    const res = await axios.get(`${API}/${kbId}/nodes/${nodeId}`);
    return res.data;
  },

  async updateNode(
    kbId: string,
    nodeId: string,
    payload: {
      label?: string;
      content?: string;
      proof_status?: ProofStatus;
      ref_label?: string;
      equation_label?: string;
    }
  ): Promise<MathNode> {
    const res = await axios.put(`${API}/${kbId}/nodes/${nodeId}`, payload);
    return res.data;
  },

  async removeNode(kbId: string, nodeId: string): Promise<void> {
    await axios.delete(`${API}/${kbId}/nodes/${nodeId}`);
  },

  // Relations
  async addRelation(
    kbId: string,
    payload: {
      source_id: string;
      target_id: string;
      relation_type: MathRelationType;
      annotation?: string;
    }
  ): Promise<MathRelation> {
    const res = await axios.post(`${API}/${kbId}/relations`, payload);
    return res.data;
  },

  async removeRelation(kbId: string, relationId: string): Promise<void> {
    await axios.delete(`${API}/${kbId}/relations/${relationId}`);
  },

  // Graph Semantics (MATH-02)
  async inspectNode(kbId: string, nodeId: string): Promise<NodeInspection> {
    const res = await axios.get(`${API}/${kbId}/nodes/${nodeId}/inspect`);
    return res.data;
  },

  async analyzeDependencies(kbId: string): Promise<DependencyAnalysis> {
    const res = await axios.get(`${API}/${kbId}/dependencies`);
    return res.data;
  },

  // Workspace (MATH-03)
  async executeCommand(kbId: string, command: Record<string, any>): Promise<WorkspaceResult> {
    const res = await axios.post(`${API}/${kbId}/workspace/command`, command);
    return res.data;
  },

  // References (MATH-05)
  async validateReferences(kbId: string): Promise<ReferenceValidation> {
    const res = await axios.get(`${API}/${kbId}/references/validate`);
    return res.data;
  },

  // Portability (MATH-06)
  async exportGraph(kbId: string, format: MathExportFormat): Promise<MathExportResult> {
    const res = await axios.post(`${API}/${kbId}/export`, { format });
    return res.data;
  },

  async importGraph(
    kbId: string,
    content: string,
    collisionStrategy?: LabelCollisionStrategy
  ): Promise<MathImportResult> {
    const res = await axios.post(`${API}/${kbId}/import`, {
      content,
      collision_strategy: collisionStrategy,
    });
    return res.data;
  },
};

// ── Helpers ─────────────────────────────────────────────────────────────

export const NODE_TYPE_CONFIG: Record<
  MathNodeType,
  { label: string; icon: string; color: string; bgColor: string }
> = {
  theorem: { label: 'Theorem', icon: 'ri-award-line', color: 'text-purple-600', bgColor: 'bg-purple-50 dark:bg-purple-900/20' },
  lemma: { label: 'Lemma', icon: 'ri-git-branch-line', color: 'text-indigo-600', bgColor: 'bg-indigo-50 dark:bg-indigo-900/20' },
  definition: { label: 'Definition', icon: 'ri-book-open-line', color: 'text-blue-600', bgColor: 'bg-blue-50 dark:bg-blue-900/20' },
  proposition: { label: 'Proposition', icon: 'ri-lightbulb-line', color: 'text-cyan-600', bgColor: 'bg-cyan-50 dark:bg-cyan-900/20' },
  corollary: { label: 'Corollary', icon: 'ri-arrow-right-down-line', color: 'text-teal-600', bgColor: 'bg-teal-50 dark:bg-teal-900/20' },
  proof: { label: 'Proof', icon: 'ri-check-double-line', color: 'text-green-600', bgColor: 'bg-green-50 dark:bg-green-900/20' },
  example: { label: 'Example', icon: 'ri-flask-line', color: 'text-orange-600', bgColor: 'bg-orange-50 dark:bg-orange-900/20' },
  problem: { label: 'Problem', icon: 'ri-question-line', color: 'text-red-600', bgColor: 'bg-red-50 dark:bg-red-900/20' },
  note: { label: 'Note', icon: 'ri-sticky-note-line', color: 'text-gray-600', bgColor: 'bg-gray-50 dark:bg-gray-900/20' },
};

export const RELATION_TYPE_CONFIG: Record<
  MathRelationType,
  { label: string; icon: string; color: string }
> = {
  depends_on: { label: 'Depends On', icon: 'ri-arrow-left-up-line', color: 'text-red-500' },
  proves: { label: 'Proves', icon: 'ri-check-line', color: 'text-green-500' },
  uses_definition: { label: 'Uses Definition', icon: 'ri-link', color: 'text-blue-500' },
  generalizes: { label: 'Generalizes', icon: 'ri-arrow-up-s-line', color: 'text-purple-500' },
  special_case_of: { label: 'Special Case Of', icon: 'ri-arrow-down-s-line', color: 'text-indigo-500' },
};

export const PROOF_STATUS_CONFIG: Record<
  ProofStatus,
  { label: string; icon: string; color: string }
> = {
  complete: { label: 'Complete', icon: 'ri-checkbox-circle-fill', color: 'text-green-500' },
  incomplete: { label: 'Incomplete', icon: 'ri-error-warning-fill', color: 'text-amber-500' },
  sketch: { label: 'Sketch', icon: 'ri-draft-line', color: 'text-blue-500' },
};
