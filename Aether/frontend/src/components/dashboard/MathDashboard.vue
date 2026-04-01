<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import KnowledgeBaseLayout from '@/components/layouts/KnowledgeBaseLayout.vue';
import { contentApi } from '@/api/content';
import AxiomTree from '@/components/math/AxiomTree.vue';
import DailyTheorem from '@/components/math/DailyTheorem.vue';
import {
  mathApi,
  NODE_TYPE_CONFIG,
  PROOF_STATUS_CONFIG,
  type MathNode,
  type MathWorkspaceMode,
  type GraphOverview,
} from '@/api/math';

interface Props {
  kb: any;
}

const props = defineProps<Props>();
const router = useRouter();
const articles = ref<any[]>([]);
const loadingArticles = ref(true);
const treeData = ref<any | null>(null);
const overview = ref<GraphOverview | null>(null);
const mathNodes = ref<MathNode[]>([]);

const selectedTag = ref<string | null>(null);

const filteredArticles = computed(() => {
  if (!selectedTag.value) return articles.value;
  return articles.value.filter((a) => {
    if (!a.tags) return false;
    return a.tags.includes(selectedTag.value);
  });
});

const handleTopicSelect = (topic: string | null) => {
  selectedTag.value = topic;
};

// Default Data for Fallback
const DEFAULT_TREE = {
  name: 'Mathematics',
  children: [
    { name: 'Logic', tag: 'Logic', children: [{ name: 'Prop. Logic' }, { name: 'First-Order' }] },
    {
      name: 'Set Theory',
      tag: 'Set Theory',
      children: [{ name: 'ZFC' }, { name: 'Ordinals' }],
    },
  ],
};

// Build tree from math graph nodes (MATH-01 integration)
const buildTreeFromMathNodes = (nodes: MathNode[]) => {
  if (nodes.length === 0) return DEFAULT_TREE;

  // Group by node type
  const groups: Record<string, any[]> = {};
  for (const node of nodes) {
    const cfg = NODE_TYPE_CONFIG[node.node_type];
    const typeName = cfg?.label || node.node_type;
    if (!groups[typeName]) groups[typeName] = [];
    groups[typeName].push({
      name: node.label,
      tag: node.ref_label || node.label,
      id: node.id,
    });
  }

  const children = Object.entries(groups).map(([name, items]) => ({
    name,
    tag: name,
    children: items,
  }));

  return {
    name: props.kb.title || 'Mathematics',
    children,
  };
};

// Fetch articles and graph for this KB
onMounted(async () => {
  try {
    const [res, graphRes, overviewRes] = await Promise.all([
      contentApi.list({ knowledge_base_id: props.kb.id }),
      fetch(`/api/kb/${props.kb.id}/graph`)
        .then((r) => r.json())
        .catch(() => []),
      mathApi.getOverview(props.kb.id).catch(() => null),
    ]);

    articles.value = res;
    overview.value = overviewRes;

    // Try to load math graph nodes first (MATH-01)
    try {
      const mathGraph = await mathApi.getGraph(props.kb.id);
      mathNodes.value = mathGraph.nodes;
      if (mathGraph.nodes.length > 0) {
        treeData.value = buildTreeFromMathNodes(mathGraph.nodes);
      } else if (!graphRes || graphRes.length === 0) {
        treeData.value = DEFAULT_TREE;
      } else {
        treeData.value = buildTree(graphRes);
      }
    } catch {
      // Fallback to legacy graph
      if (!graphRes || graphRes.length === 0) {
        treeData.value = DEFAULT_TREE;
      } else {
        treeData.value = buildTree(graphRes);
      }
    }
  } catch (e) {
    console.error('Failed to fetch dashboard data', e);
    treeData.value = DEFAULT_TREE;
  } finally {
    loadingArticles.value = false;
  }
});

const buildTree = (nodes: any[]) => {
  const map = new Map();
  const roots: any[] = [];
  nodes.sort((a: any, b: any) => a.rank - b.rank);
  nodes.forEach((n) => {
    map.set(n.id, { ...n, name: n.label, tag: n.data?.tag, children: [] });
  });
  nodes.forEach((n) => {
    const node = map.get(n.id);
    if (n.parent_id && map.has(n.parent_id)) {
      map.get(n.parent_id).children.push(node);
    } else {
      roots.push(node);
    }
  });
  if (roots.length === 1) return roots[0];
  return { name: props.kb.title, children: roots };
};

const handleCreate = () => {
  const query: any = { kb: props.kb.id };
  if (selectedTag.value) {
    query.tags = selectedTag.value;
  }
  router.push({ path: '/editor', query });
};

const emit = defineEmits(['open-settings']);
const handleSettings = () => emit('open-settings');
const goArticle = (id: string) => router.push(`/article/${id}`);
</script>

<template>
  <div v-if="!kb" class="flex items-center justify-center h-full text-ink/40 font-bold">
    Loading Context...
  </div>
  <KnowledgeBaseLayout
    v-else
    :title="kb.title"
    :loading="false"
    :can-edit="true"
    @create-article="handleCreate"
    @open-settings="handleSettings"
  >
    <template #header>
      <div
        class="h-[45vh] w-full flex flex-col md:flex-row border-b border-black/5 dark:border-white/5 bg-gray-50 dark:bg-[#0d1117] relative"
      >
        <!-- Left: Axiom Tree -->
        <div
          class="w-full md:w-2/3 h-full border-b md:border-b-0 md:border-r border-black/5 dark:border-white/5 relative overflow-hidden group"
        >
          <AxiomTree :kbId="kb.id" :treeData="treeData" @select-topic="handleTopicSelect" />

          <div
            class="absolute top-4 left-6 pointer-events-none transition-opacity duration-300"
            :class="selectedTag ? 'opacity-0' : 'opacity-100'"
          >
            <span class="text-[10px] font-black uppercase tracking-widest text-ink/30"
              >Axiomatic Structure</span
            >
          </div>

          <button
            @click="router.push(`/kb/${kb.id}/tree`)"
            class="absolute top-4 right-4 text-ink/20 hover:text-accent transition-colors bg-white/50 dark:bg-black/20 hover:bg-white dark:hover:bg-black backdrop-blur p-2 rounded-lg border border-transparent hover:border-accent/10"
          >
            <i class="ri-fullscreen-line"></i>
          </button>

          <!-- Graph Stats Overlay (MATH-02) -->
          <div v-if="overview" class="absolute bottom-4 left-6 flex gap-3 pointer-events-none">
            <span class="text-[10px] font-mono text-ink/30 bg-paper/80 px-2 py-0.5 rounded backdrop-blur">
              {{ overview.node_count }} nodes
            </span>
            <span class="text-[10px] font-mono text-ink/30 bg-paper/80 px-2 py-0.5 rounded backdrop-blur">
              {{ overview.relation_count }} relations
            </span>
            <span v-if="overview.has_cycles" class="text-[10px] font-mono text-red-500 bg-paper/80 px-2 py-0.5 rounded backdrop-blur">
              ⚠ cycles
            </span>
            <span v-if="overview.incomplete_proofs" class="text-[10px] font-mono text-amber-500 bg-paper/80 px-2 py-0.5 rounded backdrop-blur">
              {{ overview.incomplete_proofs }} incomplete
            </span>
          </div>

          <div class="absolute bottom-4 right-6 pointer-events-none" v-if="selectedTag">
            <span
              class="text-xs font-bold text-accent bg-accent/10 px-3 py-1 rounded-full backdrop-blur-sm border border-accent/20"
            >
              Filtering: {{ selectedTag }}
            </span>
          </div>
        </div>

        <!-- Right: Theorem of the Day -->
        <div class="w-full md:w-1/3 h-full p-6 flex flex-col">
          <DailyTheorem />
        </div>
      </div>
    </template>

    <template #content>
      <div class="p-8">
        <div class="flex items-center justify-between mb-8">
          <div class="flex items-center gap-4">
            <h2 class="text-xs font-black uppercase tracking-widest text-ink/40">
              Archive Contents
            </h2>
            <span
              v-if="selectedTag"
              class="text-xs font-bold text-accent flex items-center gap-1 cursor-pointer hover:text-red-500"
              @click="handleTopicSelect(null)"
            >
              / {{ selectedTag }} <i class="ri-close-circle-fill"></i>
            </span>
          </div>
          <span class="text-[10px] font-mono text-ink/30"
            >{{ filteredArticles.length }} ENTRIES</span
          >
        </div>

        <div
          v-if="loadingArticles"
          class="text-center py-20 animate-pulse text-xs font-bold uppercase tracking-widest text-ink/30"
        >
          Loading Indices...
        </div>

        <div
          v-else-if="filteredArticles.length === 0"
          class="text-center py-20 border border-dashed border-ink/10 rounded-xl"
        >
          <p class="text-ink/40 mb-4">No theorems found for this criteria.</p>
          <button
            @click="handleCreate"
            class="text-xs font-bold uppercase tracking-widest text-accent hover:underline"
          >
            Start Formalizing
          </button>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <div
            v-for="article in filteredArticles"
            :key="article.id"
            @click="goArticle(article.id)"
            class="h-32 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-lg p-5 hover:border-accent/40 hover:shadow-xl hover:shadow-accent/5 hover:-translate-y-1 transition-all duration-300 cursor-pointer group flex flex-col relative overflow-hidden backdrop-blur-sm"
          >
            <div
              class="absolute inset-0 bg-gradient-to-br from-white/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
            ></div>

            <div class="w-8 h-1 bg-accent/20 mb-auto group-hover:bg-accent transition-colors"></div>

            <h3
              class="font-bold text-ink leading-tight mb-1 group-hover:text-accent transition-colors"
            >
              {{ article.title }}
            </h3>

            <div class="flex items-center justify-between mt-auto">
              <div class="text-[10px] font-mono text-ink/40 uppercase tracking-widest">
                {{ new Date(article.created_at).toLocaleDateString() }}
              </div>
              <div class="flex gap-1" v-if="selectedTag && article.tags">
                <span class="w-1.5 h-1.5 rounded-full bg-accent"></span>
              </div>
            </div>

            <div
              class="absolute top-4 right-4 text-ink/10 group-hover:text-accent/20 transition-colors"
            >
              <i v-if="article.type === 'Folder'" class="ri-folder-fill text-xl"></i>
              <i v-else class="ri-file-text-line text-xl"></i>
            </div>
          </div>
        </div>
      </div>
    </template>
  </KnowledgeBaseLayout>
</template>
