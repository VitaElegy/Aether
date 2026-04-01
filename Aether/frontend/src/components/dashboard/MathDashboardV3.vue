<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import KnowledgeBaseLayout from '@/components/layouts/KnowledgeBaseLayout.vue';
import { contentApi } from '@/api/content';
import {
  mathApi,
  NODE_TYPE_CONFIG,
  RELATION_TYPE_CONFIG,
  PROOF_STATUS_CONFIG,
  type MathNode,
  type MathRelation,
  type MathGraph,
  type MathNodeType,
  type MathRelationType,
  type MathWorkspaceMode,
  type MathExportFormat,
  type LabelCollisionStrategy,
  type ProofStatus,
  type GraphOverview,
  type DependencyAnalysis,
  type ReferenceValidation,
  type NodeInspection,
} from '@/api/math';

interface Props {
  kb: any;
}

const props = defineProps<Props>();
const router = useRouter();
const emit = defineEmits(['open-settings']);

// ── State ───────────────────────────────────────────────────────────────

// MATH-04: Workspace modes
const currentMode = ref<MathWorkspaceMode>('workspace');

// Graph data
const graph = ref<MathGraph>({ kb_id: '', nodes: [], relations: [] });
const overview = ref<GraphOverview | null>(null);
const loading = ref(true);
const articles = ref<any[]>([]);

// Selection
const selectedNodeId = ref<string | null>(null);
const selectedInspection = ref<NodeInspection | null>(null);

// Dependency analysis (MATH-02)
const depAnalysis = ref<DependencyAnalysis | null>(null);

// Reference validation (MATH-05)
const refValidation = ref<ReferenceValidation | null>(null);

// MATH-03: Add node form
const showAddNodeForm = ref(false);
const newNodeType = ref<MathNodeType>('theorem');
const newNodeLabel = ref('');
const newNodeContent = ref('');
const newNodeRefLabel = ref('');
const newNodeEqLabel = ref('');

// MATH-03: Add relation form
const showAddRelationForm = ref(false);
const newRelSource = ref('');
const newRelTarget = ref('');
const newRelType = ref<MathRelationType>('depends_on');
const newRelAnnotation = ref('');

// MATH-06: Export/Import
const showExportPanel = ref(false);
const exportFormat = ref<MathExportFormat>('json_graph');
const exportResult = ref<string | null>(null);
const showImportPanel = ref(false);
const importContent = ref('');
const importStrategy = ref<LabelCollisionStrategy>('rename');
const importResult = ref<any>(null);

// Status message
const statusMessage = ref('');
const statusType = ref<'success' | 'error' | 'info'>('info');

// ── Computed ────────────────────────────────────────────────────────────

const nodeTypeOptions = computed(() =>
  (Object.keys(NODE_TYPE_CONFIG) as MathNodeType[]).map((k) => ({
    value: k,
    ...NODE_TYPE_CONFIG[k],
  }))
);

const relationTypeOptions = computed(() =>
  (Object.keys(RELATION_TYPE_CONFIG) as MathRelationType[]).map((k) => ({
    value: k,
    ...RELATION_TYPE_CONFIG[k],
  }))
);

const selectedNode = computed(() =>
  graph.value.nodes.find((n) => n.id === selectedNodeId.value) || null
);

const nodeRelations = computed(() => {
  if (!selectedNodeId.value) return { incoming: [], outgoing: [] };
  return {
    incoming: graph.value.relations.filter((r) => r.target_id === selectedNodeId.value),
    outgoing: graph.value.relations.filter((r) => r.source_id === selectedNodeId.value),
  };
});

const incompleteProofs = computed(() =>
  graph.value.nodes.filter((n) => n.proof_status === 'incomplete')
);

const hasCycles = computed(() => depAnalysis.value?.cycles?.length ? depAnalysis.value.cycles.length > 0 : false);

const unresolvedCount = computed(() => refValidation.value?.unresolved_refs?.length ?? 0);
const duplicateCount = computed(() => refValidation.value?.duplicate_labels?.length ?? 0);

// ── Lifecycle ───────────────────────────────────────────────────────────

onMounted(async () => {
  await refreshAll();
});

async function refreshAll() {
  loading.value = true;
  try {
    const [g, o, arts] = await Promise.all([
      mathApi.getGraph(props.kb.id),
      mathApi.getOverview(props.kb.id),
      contentApi.list({ knowledge_base_id: props.kb.id }).catch(() => []),
    ]);
    graph.value = g;
    overview.value = o;
    articles.value = arts;

    // Auto-analyze
    depAnalysis.value = await mathApi.analyzeDependencies(props.kb.id);
    refValidation.value = await mathApi.validateReferences(props.kb.id);
  } catch (e) {
    console.error('Failed to load math graph:', e);
  } finally {
    loading.value = false;
  }
}

// ── Node Actions (MATH-03) ─────────────────────────────────────────────

async function handleAddNode() {
  if (!newNodeLabel.value.trim()) return;
  try {
    await mathApi.addNode(props.kb.id, {
      node_type: newNodeType.value,
      label: newNodeLabel.value,
      content: newNodeContent.value,
      ref_label: newNodeRefLabel.value || undefined,
      equation_label: newNodeEqLabel.value || undefined,
    });
    newNodeLabel.value = '';
    newNodeContent.value = '';
    newNodeRefLabel.value = '';
    newNodeEqLabel.value = '';
    showAddNodeForm.value = false;
    setStatus('Node added', 'success');
    await refreshAll();
  } catch (e) {
    setStatus('Failed to add node', 'error');
  }
}

async function handleRemoveNode(nodeId: string) {
  try {
    await mathApi.removeNode(props.kb.id, nodeId);
    if (selectedNodeId.value === nodeId) selectedNodeId.value = null;
    setStatus('Node removed', 'success');
    await refreshAll();
  } catch (e) {
    setStatus('Failed to remove node', 'error');
  }
}

async function selectNode(nodeId: string) {
  selectedNodeId.value = nodeId;
  try {
    selectedInspection.value = await mathApi.inspectNode(props.kb.id, nodeId);
  } catch {
    selectedInspection.value = null;
  }
}

// ── Relation Actions (MATH-03) ─────────────────────────────────────────

async function handleAddRelation() {
  if (!newRelSource.value || !newRelTarget.value) return;
  try {
    await mathApi.addRelation(props.kb.id, {
      source_id: newRelSource.value,
      target_id: newRelTarget.value,
      relation_type: newRelType.value,
      annotation: newRelAnnotation.value || undefined,
    });
    showAddRelationForm.value = false;
    newRelAnnotation.value = '';
    setStatus('Relation added', 'success');
    await refreshAll();
  } catch (e) {
    setStatus('Failed to add relation', 'error');
  }
}

async function handleRemoveRelation(relId: string) {
  try {
    await mathApi.removeRelation(props.kb.id, relId);
    setStatus('Relation removed', 'success');
    await refreshAll();
  } catch (e) {
    setStatus('Failed to remove relation', 'error');
  }
}

// ── Workspace Commands (MATH-03) ───────────────────────────────────────

async function markIncompleteProof(nodeId: string) {
  const result = await mathApi.executeCommand(props.kb.id, {
    action: 'mark_incomplete_proof',
    node_id: nodeId,
  });
  setStatus(result.message, result.success ? 'success' : 'error');
  await refreshAll();
}

async function highlightBlockers(nodeId: string) {
  const result = await mathApi.executeCommand(props.kb.id, {
    action: 'highlight_blockers',
    node_id: nodeId,
  });
  setStatus(result.message, result.success ? 'success' : 'error');
}

// ── Export / Import (MATH-06) ───────────────────────────────────────────

async function handleExport() {
  try {
    const result = await mathApi.exportGraph(props.kb.id, exportFormat.value);
    exportResult.value = result.content;
    setStatus(`Exported ${result.node_count} nodes, ${result.relation_count} relations`, 'success');
  } catch (e) {
    setStatus('Export failed', 'error');
  }
}

function downloadExport() {
  if (!exportResult.value) return;
  const ext = exportFormat.value === 'json_graph' ? 'json' : exportFormat.value === 'latex_package' ? 'tex' : 'md';
  const blob = new Blob([exportResult.value], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `math_export.${ext}`;
  a.click();
  URL.revokeObjectURL(url);
}

async function handleImport() {
  if (!importContent.value.trim()) return;
  try {
    const result = await mathApi.importGraph(props.kb.id, importContent.value, importStrategy.value);
    importResult.value = result;
    setStatus(`Imported: ${result.nodes_created} created, ${result.nodes_skipped} skipped`, 'success');
    await refreshAll();
  } catch (e) {
    setStatus('Import failed', 'error');
  }
}

// ── Helpers ─────────────────────────────────────────────────────────────

function setStatus(msg: string, type: 'success' | 'error' | 'info') {
  statusMessage.value = msg;
  statusType.value = type;
  setTimeout(() => {
    statusMessage.value = '';
  }, 4000);
}

function getNodeLabel(id: string): string {
  return graph.value.nodes.find((n) => n.id === id)?.label || id.slice(0, 8);
}

const handleCreate = () => router.push({ path: '/editor', query: { kb: props.kb.id } });
const handleSettings = () => emit('open-settings');
const goArticle = (id: string) => router.push(`/article/${id}`);
</script>

<template>
  <KnowledgeBaseLayout
    :title="kb.title"
    :loading="false"
    :can-edit="true"
    @create-article="handleCreate"
    @open-settings="handleSettings"
  >
    <template #header>
      <div
        class="w-full border-b border-black/5 dark:border-white/5 bg-gray-50 dark:bg-[#0d1117]"
      >
        <!-- Mode Switcher (MATH-04) -->
        <div class="flex items-center justify-between px-6 py-3 border-b border-black/5 dark:border-white/5">
          <div class="flex items-center gap-1">
            <button
              v-for="mode in (['manuscript', 'archive', 'workspace'] as MathWorkspaceMode[])"
              :key="mode"
              @click="currentMode = mode"
              :class="[
                'px-4 py-2 text-[10px] font-black uppercase tracking-[0.15em] rounded-md transition-all',
                currentMode === mode
                  ? 'bg-accent text-white shadow-lg shadow-accent/20'
                  : 'text-ink/40 hover:text-ink/70 hover:bg-ink/5',
              ]"
            >
              <i
                :class="[
                  mode === 'manuscript' ? 'ri-quill-pen-line' : mode === 'archive' ? 'ri-archive-line' : 'ri-node-tree',
                  'mr-1',
                ]"
              ></i>
              {{ mode }}
            </button>
          </div>

          <!-- Stats Bar -->
          <div class="flex items-center gap-4 text-[10px] font-mono text-ink/40">
            <span>{{ overview?.node_count ?? 0 }} nodes</span>
            <span>{{ overview?.relation_count ?? 0 }} relations</span>
            <span v-if="hasCycles" class="text-red-500 font-bold">⚠ CYCLES</span>
            <span v-if="overview?.incomplete_proofs" class="text-amber-500">{{ overview.incomplete_proofs }} incomplete</span>
            <span v-if="unresolvedCount" class="text-orange-500">{{ unresolvedCount }} unresolved</span>
            <span v-if="duplicateCount" class="text-red-400">{{ duplicateCount }} duplicates</span>
          </div>

          <!-- Portability (MATH-06) -->
          <div class="flex items-center gap-2">
            <button
              @click="showExportPanel = !showExportPanel; showImportPanel = false"
              class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-ink/40 hover:text-accent border border-ink/10 hover:border-accent/30 rounded transition-all"
            >
              <i class="ri-download-2-line mr-1"></i>Export
            </button>
            <button
              @click="showImportPanel = !showImportPanel; showExportPanel = false"
              class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-ink/40 hover:text-accent border border-ink/10 hover:border-accent/30 rounded transition-all"
            >
              <i class="ri-upload-2-line mr-1"></i>Import
            </button>
          </div>
        </div>

        <!-- Status Bar -->
        <Transition name="slide-down">
          <div
            v-if="statusMessage"
            :class="[
              'px-6 py-2 text-xs font-bold',
              statusType === 'success' ? 'bg-green-500/10 text-green-600' : statusType === 'error' ? 'bg-red-500/10 text-red-600' : 'bg-blue-500/10 text-blue-600',
            ]"
          >
            {{ statusMessage }}
          </div>
        </Transition>
      </div>
    </template>

    <template #content>
      <div class="flex h-full overflow-hidden">
        <!-- ═══════════════════════════════════════════════════════════════ -->
        <!-- WORKSPACE MODE (MATH-03) -->
        <!-- ═══════════════════════════════════════════════════════════════ -->
        <template v-if="currentMode === 'workspace'">
          <!-- Left Panel: Node List + Actions -->
          <div class="w-80 border-r border-ink/5 flex flex-col overflow-hidden">
            <!-- Actions -->
            <div class="p-4 border-b border-ink/5 flex gap-2">
              <button
                @click="showAddNodeForm = !showAddNodeForm; showAddRelationForm = false"
                class="flex-1 py-2 text-[10px] font-bold uppercase tracking-widest bg-accent/10 text-accent hover:bg-accent/20 rounded transition-colors"
              >
                <i class="ri-add-circle-line mr-1"></i>Node
              </button>
              <button
                @click="showAddRelationForm = !showAddRelationForm; showAddNodeForm = false"
                class="flex-1 py-2 text-[10px] font-bold uppercase tracking-widest bg-blue-500/10 text-blue-600 hover:bg-blue-500/20 rounded transition-colors"
              >
                <i class="ri-links-line mr-1"></i>Relation
              </button>
            </div>

            <!-- Add Node Form (MATH-03) -->
            <Transition name="slide-down">
              <div v-if="showAddNodeForm" class="p-4 border-b border-ink/5 bg-accent/5 space-y-3">
                <select
                  v-model="newNodeType"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                >
                  <option v-for="opt in nodeTypeOptions" :key="opt.value" :value="opt.value">
                    {{ opt.label }}
                  </option>
                </select>
                <input
                  v-model="newNodeLabel"
                  placeholder="Label (e.g., Bolzano-Weierstrass)"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                />
                <textarea
                  v-model="newNodeContent"
                  placeholder="Content (LaTeX or Markdown)"
                  rows="3"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink font-mono"
                ></textarea>
                <div class="flex gap-2">
                  <input
                    v-model="newNodeRefLabel"
                    placeholder="Ref label (thm:bw)"
                    class="flex-1 px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                  />
                  <input
                    v-model="newNodeEqLabel"
                    placeholder="Eq label (eq:1)"
                    class="flex-1 px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                  />
                </div>
                <button
                  @click="handleAddNode"
                  class="w-full py-2 text-xs font-bold uppercase tracking-widest bg-accent text-white rounded hover:brightness-110 transition-all"
                >
                  Add {{ NODE_TYPE_CONFIG[newNodeType].label }}
                </button>
              </div>
            </Transition>

            <!-- Add Relation Form (MATH-03) -->
            <Transition name="slide-down">
              <div v-if="showAddRelationForm" class="p-4 border-b border-ink/5 bg-blue-500/5 space-y-3">
                <select
                  v-model="newRelSource"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                >
                  <option value="" disabled>Source node</option>
                  <option v-for="n in graph.nodes" :key="n.id" :value="n.id">
                    {{ n.label }} ({{ NODE_TYPE_CONFIG[n.node_type]?.label }})
                  </option>
                </select>
                <select
                  v-model="newRelType"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                >
                  <option v-for="opt in relationTypeOptions" :key="opt.value" :value="opt.value">
                    {{ opt.label }}
                  </option>
                </select>
                <select
                  v-model="newRelTarget"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                >
                  <option value="" disabled>Target node</option>
                  <option v-for="n in graph.nodes" :key="n.id" :value="n.id">
                    {{ n.label }} ({{ NODE_TYPE_CONFIG[n.node_type]?.label }})
                  </option>
                </select>
                <input
                  v-model="newRelAnnotation"
                  placeholder="Annotation (optional)"
                  class="w-full px-3 py-2 text-xs border border-ink/10 rounded bg-paper text-ink"
                />
                <button
                  @click="handleAddRelation"
                  class="w-full py-2 text-xs font-bold uppercase tracking-widest bg-blue-600 text-white rounded hover:brightness-110 transition-all"
                >
                  Add Relation
                </button>
              </div>
            </Transition>

            <!-- Node List -->
            <div class="flex-1 overflow-y-auto">
              <div v-if="loading" class="p-8 text-center text-xs font-bold text-ink/30 uppercase tracking-widest animate-pulse">
                Loading graph...
              </div>
              <div v-else-if="graph.nodes.length === 0" class="p-8 text-center">
                <p class="text-ink/40 text-sm mb-4">No nodes yet</p>
                <button @click="showAddNodeForm = true" class="text-accent text-xs font-bold hover:underline">
                  Add your first node
                </button>
              </div>
              <div v-else>
                <div
                  v-for="node in graph.nodes"
                  :key="node.id"
                  @click="selectNode(node.id)"
                  :class="[
                    'px-4 py-3 border-b border-ink/5 cursor-pointer transition-all group',
                    selectedNodeId === node.id ? 'bg-accent/10 border-l-2 border-l-accent' : 'hover:bg-ink/5',
                  ]"
                >
                  <div class="flex items-center gap-2 mb-1">
                    <i :class="[NODE_TYPE_CONFIG[node.node_type]?.icon, NODE_TYPE_CONFIG[node.node_type]?.color, 'text-sm']"></i>
                    <span class="text-[10px] font-bold uppercase tracking-widest" :class="NODE_TYPE_CONFIG[node.node_type]?.color">
                      {{ NODE_TYPE_CONFIG[node.node_type]?.label }}
                    </span>
                    <span v-if="node.proof_status" :class="[PROOF_STATUS_CONFIG[node.proof_status]?.color, 'text-[10px] ml-auto']">
                      <i :class="PROOF_STATUS_CONFIG[node.proof_status]?.icon"></i>
                    </span>
                  </div>
                  <div class="font-bold text-sm text-ink leading-tight">{{ node.label }}</div>
                  <div v-if="node.ref_label" class="text-[10px] font-mono text-ink/40 mt-1">\\ref{<span class="text-accent">{{ node.ref_label }}</span>}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Center Panel: Node Detail / Inspector (MATH-02) -->
          <div class="flex-1 overflow-y-auto">
            <div v-if="!selectedNode" class="h-full flex items-center justify-center text-ink/30">
              <div class="text-center">
                <i class="ri-node-tree text-5xl mb-4 block opacity-30"></i>
                <p class="text-sm font-bold">Select a node to inspect</p>
                <p class="text-xs mt-1">Or add a new node to get started</p>
              </div>
            </div>

            <div v-else class="p-8">
              <!-- Node Header -->
              <div class="flex items-start justify-between mb-6">
                <div>
                  <div class="flex items-center gap-3 mb-2">
                    <span :class="['px-3 py-1 text-[10px] font-bold uppercase tracking-widest rounded-full', NODE_TYPE_CONFIG[selectedNode.node_type]?.bgColor, NODE_TYPE_CONFIG[selectedNode.node_type]?.color]">
                      <i :class="NODE_TYPE_CONFIG[selectedNode.node_type]?.icon" class="mr-1"></i>
                      {{ NODE_TYPE_CONFIG[selectedNode.node_type]?.label }}
                    </span>
                    <span v-if="selectedNode.proof_status" :class="['px-3 py-1 text-[10px] font-bold uppercase tracking-widest rounded-full', PROOF_STATUS_CONFIG[selectedNode.proof_status]?.color, 'bg-current/10']">
                      {{ PROOF_STATUS_CONFIG[selectedNode.proof_status]?.label }}
                    </span>
                  </div>
                  <h2 class="text-2xl font-black text-ink">{{ selectedNode.label }}</h2>
                  <div class="flex items-center gap-4 mt-2 text-[10px] font-mono text-ink/40">
                    <span v-if="selectedNode.ref_label">ref: {{ selectedNode.ref_label }}</span>
                    <span v-if="selectedNode.equation_label">eq: {{ selectedNode.equation_label }}</span>
                    <span>depth: {{ selectedInspection?.dependency_depth ?? 0 }}</span>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    v-if="selectedNode.node_type === 'proof'"
                    @click="markIncompleteProof(selectedNode.id)"
                    class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-amber-500 border border-amber-500/30 rounded hover:bg-amber-500/10 transition-colors"
                  >
                    <i class="ri-error-warning-line mr-1"></i>Mark Incomplete
                  </button>
                  <button
                    @click="highlightBlockers(selectedNode.id)"
                    class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-red-500 border border-red-500/30 rounded hover:bg-red-500/10 transition-colors"
                  >
                    <i class="ri-alarm-warning-line mr-1"></i>Blockers
                  </button>
                  <button
                    @click="handleRemoveNode(selectedNode.id)"
                    class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-red-400 border border-red-400/30 rounded hover:bg-red-400/10 transition-colors"
                  >
                    <i class="ri-delete-bin-line"></i>
                  </button>
                </div>
              </div>

              <!-- Content -->
              <div class="mb-8 p-6 bg-ink/5 rounded-xl border border-ink/5 font-mono text-sm text-ink/80 whitespace-pre-wrap">{{ selectedNode.content || '(no content)' }}</div>

              <!-- Relations (MATH-02) -->
              <div class="grid grid-cols-2 gap-6 mb-8">
                <!-- Outgoing -->
                <div>
                  <h3 class="text-[10px] font-black uppercase tracking-widest text-ink/40 mb-3">
                    <i class="ri-arrow-right-line mr-1"></i>Outgoing Relations ({{ nodeRelations.outgoing.length }})
                  </h3>
                  <div v-if="nodeRelations.outgoing.length === 0" class="text-xs text-ink/30 italic">None</div>
                  <div
                    v-for="rel in nodeRelations.outgoing"
                    :key="rel.id"
                    class="flex items-center justify-between p-3 mb-2 bg-paper border border-ink/5 rounded-lg group"
                  >
                    <div class="flex items-center gap-2">
                      <i :class="[RELATION_TYPE_CONFIG[rel.relation_type]?.icon, RELATION_TYPE_CONFIG[rel.relation_type]?.color]"></i>
                      <span class="text-xs font-bold">{{ RELATION_TYPE_CONFIG[rel.relation_type]?.label }}</span>
                      <span class="text-xs text-ink/60">→ {{ getNodeLabel(rel.target_id) }}</span>
                    </div>
                    <button @click.stop="handleRemoveRelation(rel.id)" class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600 transition-all">
                      <i class="ri-close-line"></i>
                    </button>
                  </div>
                </div>

                <!-- Incoming -->
                <div>
                  <h3 class="text-[10px] font-black uppercase tracking-widest text-ink/40 mb-3">
                    <i class="ri-arrow-left-line mr-1"></i>Incoming Relations ({{ nodeRelations.incoming.length }})
                  </h3>
                  <div v-if="nodeRelations.incoming.length === 0" class="text-xs text-ink/30 italic">None</div>
                  <div
                    v-for="rel in nodeRelations.incoming"
                    :key="rel.id"
                    class="flex items-center justify-between p-3 mb-2 bg-paper border border-ink/5 rounded-lg group"
                  >
                    <div class="flex items-center gap-2">
                      <span class="text-xs text-ink/60">{{ getNodeLabel(rel.source_id) }} →</span>
                      <i :class="[RELATION_TYPE_CONFIG[rel.relation_type]?.icon, RELATION_TYPE_CONFIG[rel.relation_type]?.color]"></i>
                      <span class="text-xs font-bold">{{ RELATION_TYPE_CONFIG[rel.relation_type]?.label }}</span>
                    </div>
                    <button @click.stop="handleRemoveRelation(rel.id)" class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600 transition-all">
                      <i class="ri-close-line"></i>
                    </button>
                  </div>
                </div>
              </div>

              <!-- Dependency Analysis Summary (MATH-02) -->
              <div v-if="depAnalysis" class="p-6 bg-paper border border-ink/5 rounded-xl">
                <h3 class="text-[10px] font-black uppercase tracking-widest text-ink/40 mb-4">
                  <i class="ri-flow-chart mr-1"></i>Dependency Analysis
                </h3>
                <div class="grid grid-cols-3 gap-4 text-center">
                  <div class="p-3 bg-ink/5 rounded-lg">
                    <div class="text-2xl font-black text-ink">{{ depAnalysis.topological_order.length }}</div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Sorted Nodes</div>
                  </div>
                  <div :class="['p-3 rounded-lg', hasCycles ? 'bg-red-500/10' : 'bg-green-500/10']">
                    <div :class="['text-2xl font-black', hasCycles ? 'text-red-500' : 'text-green-500']">
                      {{ depAnalysis.cycles.length }}
                    </div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Cycles</div>
                  </div>
                  <div :class="['p-3 rounded-lg', depAnalysis.unresolved_prerequisites.length ? 'bg-amber-500/10' : 'bg-green-500/10']">
                    <div :class="['text-2xl font-black', depAnalysis.unresolved_prerequisites.length ? 'text-amber-500' : 'text-green-500']">
                      {{ depAnalysis.unresolved_prerequisites.length }}
                    </div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Unresolved</div>
                  </div>
                </div>

                <!-- Cycle Details -->
                <div v-if="hasCycles" class="mt-4 p-3 bg-red-500/5 border border-red-500/20 rounded-lg">
                  <div class="text-xs font-bold text-red-500 mb-2">Circular Dependencies Detected:</div>
                  <div v-for="(cycle, idx) in depAnalysis.cycles" :key="idx" class="text-xs font-mono text-red-400">
                    {{ cycle.map(id => getNodeLabel(id)).join(' → ') }} → ⟲
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Right Panel: Reference Validation (MATH-05) -->
          <div class="w-72 border-l border-ink/5 overflow-y-auto">
            <div class="p-4">
              <h3 class="text-[10px] font-black uppercase tracking-widest text-ink/40 mb-4">
                <i class="ri-links-line mr-1"></i>Reference Integrity
              </h3>

              <!-- Unresolved Refs -->
              <div v-if="refValidation?.unresolved_refs?.length" class="mb-6">
                <div class="text-[10px] font-bold uppercase tracking-widest text-orange-500 mb-2">
                  ⚠ Unresolved References ({{ refValidation.unresolved_refs.length }})
                </div>
                <div
                  v-for="ref in refValidation.unresolved_refs"
                  :key="`${ref.source_node_id}-${ref.ref_label}`"
                  class="p-2 mb-1 text-xs bg-orange-500/5 border border-orange-500/10 rounded"
                >
                  <span class="font-mono text-orange-600">\\ref{<span class="font-bold">{{ ref.ref_label }}</span>}</span>
                  <div class="text-ink/50 mt-0.5">in {{ ref.source_label }}</div>
                </div>
              </div>

              <!-- Duplicate Labels -->
              <div v-if="refValidation?.duplicate_labels?.length" class="mb-6">
                <div class="text-[10px] font-bold uppercase tracking-widest text-red-500 mb-2">
                  ✖ Duplicate Labels ({{ refValidation.duplicate_labels.length }})
                </div>
                <div
                  v-for="dup in refValidation.duplicate_labels"
                  :key="dup.label"
                  class="p-2 mb-1 text-xs bg-red-500/5 border border-red-500/10 rounded"
                >
                  <span class="font-mono font-bold text-red-600">{{ dup.label }}</span>
                  <div class="text-ink/50 mt-0.5">{{ dup.node_ids.length }} nodes</div>
                </div>
              </div>

              <!-- Theorem Refs -->
              <div class="mb-4">
                <div class="text-[10px] font-bold uppercase tracking-widest text-purple-500 mb-2">
                  Theorem Refs ({{ refValidation?.theorem_refs?.length ?? 0 }})
                </div>
                <div v-for="r in (refValidation?.theorem_refs ?? []).slice(0, 10)" :key="r.ref_label" class="text-xs font-mono text-ink/60 mb-1">
                  {{ r.ref_label }}
                </div>
              </div>

              <!-- Definition Refs -->
              <div class="mb-4">
                <div class="text-[10px] font-bold uppercase tracking-widest text-blue-500 mb-2">
                  Definition Refs ({{ refValidation?.definition_refs?.length ?? 0 }})
                </div>
                <div v-for="r in (refValidation?.definition_refs ?? []).slice(0, 10)" :key="r.ref_label" class="text-xs font-mono text-ink/60 mb-1">
                  {{ r.ref_label }}
                </div>
              </div>

              <!-- Equation Labels -->
              <div class="mb-4">
                <div class="text-[10px] font-bold uppercase tracking-widest text-green-500 mb-2">
                  Equation Labels ({{ refValidation?.equation_labels?.length ?? 0 }})
                </div>
                <div v-for="r in (refValidation?.equation_labels ?? []).slice(0, 10)" :key="r.ref_label" class="text-xs font-mono text-ink/60 mb-1">
                  {{ r.ref_label }}
                </div>
              </div>

              <!-- Incomplete Proofs -->
              <div v-if="incompleteProofs.length" class="mb-4">
                <div class="text-[10px] font-bold uppercase tracking-widest text-amber-500 mb-2">
                  <i class="ri-error-warning-fill mr-1"></i>Incomplete Proofs ({{ incompleteProofs.length }})
                </div>
                <div
                  v-for="p in incompleteProofs"
                  :key="p.id"
                  @click="selectNode(p.id)"
                  class="p-2 mb-1 text-xs bg-amber-500/5 border border-amber-500/10 rounded cursor-pointer hover:bg-amber-500/10 transition-colors"
                >
                  {{ p.label }}
                </div>
              </div>
            </div>
          </div>
        </template>

        <!-- ═══════════════════════════════════════════════════════════════ -->
        <!-- MANUSCRIPT MODE (MATH-04) -->
        <!-- ═══════════════════════════════════════════════════════════════ -->
        <template v-else-if="currentMode === 'manuscript'">
          <div class="flex-1 overflow-y-auto">
            <div class="max-w-3xl mx-auto py-12 px-8">
              <h1 class="text-4xl font-black font-serif mb-2">{{ kb.title }}</h1>
              <p class="text-ink/40 text-sm mb-12 font-mono">{{ graph.nodes.length }} formal objects · {{ graph.relations.length }} relations</p>

              <div v-if="graph.nodes.length === 0" class="text-center py-20 border border-dashed border-ink/10 rounded-xl">
                <p class="text-ink/40 mb-4">No formal objects yet.</p>
                <button @click="currentMode = 'workspace'; showAddNodeForm = true" class="text-accent text-xs font-bold uppercase tracking-widest hover:underline">
                  Switch to Workspace to add nodes
                </button>
              </div>

              <!-- Render nodes grouped by type in manuscript style -->
              <template v-for="nodeType in (['definition', 'theorem', 'lemma', 'proposition', 'corollary', 'proof', 'example', 'problem', 'note'] as MathNodeType[])" :key="nodeType">
                <template v-if="graph.nodes.filter(n => n.node_type === nodeType).length > 0">
                  <div v-for="node in graph.nodes.filter(n => n.node_type === nodeType)" :key="node.id" class="mb-8">
                    <div class="flex items-center gap-3 mb-3">
                      <span :class="['px-2 py-0.5 text-[10px] font-bold uppercase tracking-widest rounded', NODE_TYPE_CONFIG[node.node_type]?.bgColor, NODE_TYPE_CONFIG[node.node_type]?.color]">
                        {{ NODE_TYPE_CONFIG[node.node_type]?.label }}
                      </span>
                      <span v-if="node.ref_label" class="text-[10px] font-mono text-ink/40">({{ node.ref_label }})</span>
                    </div>
                    <h3 class="text-xl font-bold font-serif text-ink mb-2">{{ node.label }}</h3>
                    <div class="prose prose-sm font-serif text-ink/80 leading-relaxed whitespace-pre-wrap">{{ node.content }}</div>
                    <div v-if="node.proof_status" class="mt-2 text-xs font-mono" :class="PROOF_STATUS_CONFIG[node.proof_status]?.color">
                      <i :class="PROOF_STATUS_CONFIG[node.proof_status]?.icon" class="mr-1"></i>
                      {{ PROOF_STATUS_CONFIG[node.proof_status]?.label }}
                    </div>
                    <div class="w-16 h-px bg-ink/10 mt-6"></div>
                  </div>
                </template>
              </template>
            </div>
          </div>
        </template>

        <!-- ═══════════════════════════════════════════════════════════════ -->
        <!-- ARCHIVE MODE (MATH-04) -->
        <!-- ═══════════════════════════════════════════════════════════════ -->
        <template v-else-if="currentMode === 'archive'">
          <div class="flex-1 overflow-y-auto p-8">
            <div class="flex items-center justify-between mb-8">
              <h2 class="text-xs font-black uppercase tracking-widest text-ink/40">Archive Contents</h2>
              <span class="text-[10px] font-mono text-ink/30">{{ articles.length }} articles · {{ graph.nodes.length }} formal objects</span>
            </div>

            <!-- Articles Grid -->
            <div v-if="articles.length > 0" class="mb-12">
              <h3 class="text-[10px] font-black uppercase tracking-widest text-ink/30 mb-4">Articles</h3>
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                <div
                  v-for="article in articles"
                  :key="article.id"
                  @click="goArticle(article.id)"
                  class="h-32 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-lg p-5 hover:border-accent/40 hover:shadow-xl hover:shadow-accent/5 hover:-translate-y-1 transition-all duration-300 cursor-pointer group flex flex-col"
                >
                  <div class="w-8 h-1 bg-accent/20 mb-auto group-hover:bg-accent transition-colors"></div>
                  <h3 class="font-bold text-ink leading-tight mb-1 group-hover:text-accent transition-colors">{{ article.title }}</h3>
                  <div class="text-[10px] font-mono text-ink/40 uppercase tracking-widest">{{ new Date(article.created_at).toLocaleDateString() }}</div>
                </div>
              </div>
            </div>

            <!-- Formal Objects Table -->
            <div>
              <h3 class="text-[10px] font-black uppercase tracking-widest text-ink/30 mb-4">Formal Objects</h3>
              <div v-if="graph.nodes.length === 0" class="text-center py-12 border border-dashed border-ink/10 rounded-xl text-ink/40">
                No formal objects in this knowledge base yet.
              </div>
              <table v-else class="w-full text-sm">
                <thead>
                  <tr class="text-[10px] font-bold uppercase tracking-widest text-ink/40 border-b border-ink/10">
                    <th class="text-left py-2 px-3">Type</th>
                    <th class="text-left py-2 px-3">Label</th>
                    <th class="text-left py-2 px-3">Ref</th>
                    <th class="text-left py-2 px-3">Status</th>
                    <th class="text-left py-2 px-3">Updated</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="node in graph.nodes"
                    :key="node.id"
                    @click="currentMode = 'workspace'; selectNode(node.id)"
                    class="border-b border-ink/5 hover:bg-ink/5 cursor-pointer transition-colors"
                  >
                    <td class="py-2 px-3">
                      <span :class="[NODE_TYPE_CONFIG[node.node_type]?.color, 'text-xs font-bold']">
                        <i :class="NODE_TYPE_CONFIG[node.node_type]?.icon" class="mr-1"></i>
                        {{ NODE_TYPE_CONFIG[node.node_type]?.label }}
                      </span>
                    </td>
                    <td class="py-2 px-3 font-bold">{{ node.label }}</td>
                    <td class="py-2 px-3 font-mono text-xs text-ink/50">{{ node.ref_label || '-' }}</td>
                    <td class="py-2 px-3">
                      <span v-if="node.proof_status" :class="PROOF_STATUS_CONFIG[node.proof_status]?.color" class="text-xs">
                        {{ PROOF_STATUS_CONFIG[node.proof_status]?.label }}
                      </span>
                      <span v-else class="text-ink/30 text-xs">-</span>
                    </td>
                    <td class="py-2 px-3 text-xs text-ink/40 font-mono">{{ new Date(node.updated_at).toLocaleDateString() }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </template>
      </div>

      <!-- ═══════════════════════════════════════════════════════════════ -->
      <!-- Export Panel Overlay (MATH-06) -->
      <!-- ═══════════════════════════════════════════════════════════════ -->
      <Transition name="slide-down">
        <div v-if="showExportPanel" class="fixed inset-x-0 top-24 z-50 flex justify-center">
          <div class="w-[600px] bg-paper border border-ink/10 rounded-xl shadow-2xl p-6">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-sm font-black uppercase tracking-widest">Export Graph</h3>
              <button @click="showExportPanel = false" class="text-ink/40 hover:text-ink">
                <i class="ri-close-line text-lg"></i>
              </button>
            </div>
            <div class="flex gap-3 mb-4">
              <button
                v-for="fmt in (['json_graph', 'markdown_manuscript', 'latex_package'] as MathExportFormat[])"
                :key="fmt"
                @click="exportFormat = fmt"
                :class="['px-4 py-2 text-xs font-bold uppercase tracking-widest rounded border transition-all', exportFormat === fmt ? 'bg-accent text-white border-accent' : 'border-ink/10 text-ink/60 hover:border-accent/30']"
              >
                {{ fmt === 'json_graph' ? 'JSON' : fmt === 'markdown_manuscript' ? 'Markdown' : 'LaTeX' }}
              </button>
            </div>
            <button @click="handleExport" class="w-full py-2 bg-accent text-white text-xs font-bold uppercase tracking-widest rounded hover:brightness-110 transition-all mb-4">
              Generate Export
            </button>
            <div v-if="exportResult" class="relative">
              <textarea readonly :value="exportResult" class="w-full h-48 p-4 text-xs font-mono bg-ink/5 border border-ink/10 rounded-lg text-ink/70 resize-none"></textarea>
              <button @click="downloadExport" class="absolute top-2 right-2 px-3 py-1 bg-accent text-white text-[10px] font-bold uppercase tracking-widest rounded hover:brightness-110 transition-all">
                <i class="ri-download-line mr-1"></i>Download
              </button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Import Panel Overlay (MATH-06) -->
      <Transition name="slide-down">
        <div v-if="showImportPanel" class="fixed inset-x-0 top-24 z-50 flex justify-center">
          <div class="w-[600px] bg-paper border border-ink/10 rounded-xl shadow-2xl p-6">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-sm font-black uppercase tracking-widest">Import Graph</h3>
              <button @click="showImportPanel = false" class="text-ink/40 hover:text-ink">
                <i class="ri-close-line text-lg"></i>
              </button>
            </div>
            <textarea
              v-model="importContent"
              placeholder="Paste JSON graph export here..."
              class="w-full h-48 p-4 text-xs font-mono bg-ink/5 border border-ink/10 rounded-lg text-ink resize-none mb-4"
            ></textarea>
            <div class="flex items-center gap-3 mb-4">
              <span class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Collision Strategy:</span>
              <button
                v-for="strat in (['skip', 'rename', 'overwrite'] as LabelCollisionStrategy[])"
                :key="strat"
                @click="importStrategy = strat"
                :class="['px-3 py-1 text-[10px] font-bold uppercase tracking-widest rounded border transition-all', importStrategy === strat ? 'bg-accent text-white border-accent' : 'border-ink/10 text-ink/60 hover:border-accent/30']"
              >
                {{ strat }}
              </button>
            </div>
            <button @click="handleImport" class="w-full py-2 bg-accent text-white text-xs font-bold uppercase tracking-widest rounded hover:brightness-110 transition-all">
              Import
            </button>
            <div v-if="importResult" class="mt-4 p-4 bg-green-500/5 border border-green-500/20 rounded-lg text-xs">
              <div class="font-bold text-green-600 mb-2">Import Complete</div>
              <div>Created: {{ importResult.nodes_created }} · Skipped: {{ importResult.nodes_skipped }} · Renamed: {{ importResult.nodes_renamed }}</div>
              <div>Relations: {{ importResult.relations_created }}</div>
              <div v-if="importResult.collisions.length" class="mt-2 text-amber-600">
                Collisions: {{ importResult.collisions.join(', ') }}
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </template>
  </KnowledgeBaseLayout>
</template>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
