<script setup lang="ts">
import { ref, onMounted, computed, defineAsyncComponent } from 'vue';
import { useRouter } from 'vue-router';
import KnowledgeBaseLayout from '@/components/layouts/KnowledgeBaseLayout.vue';
import { knowledgeApi } from '@/api/knowledge';
import { contentApi } from '@/api/content';
import AxiomTree from '@/components/math/AxiomTree.vue';
import DailyTheorem from '@/components/math/DailyTheorem.vue';

interface Props {
    kb: any;
}

const props = defineProps<Props>();
const router = useRouter();
const articles = ref<any[]>([]);
const loadingArticles = ref(true);
const treeData = ref<any | null>(null);

const selectedTag = ref<string | null>(null);

const filteredArticles = computed(() => {
    if (!selectedTag.value) return articles.value;
    return articles.value.filter(a => {
        if (!a.tags) return false;
        return a.tags.includes(selectedTag.value);
    });
});

const handleTopicSelect = (topic: string | null) => {
    selectedTag.value = topic;
};

// Default Data for Fallback
const DEFAULT_TREE = {
    name: "Math V2 Test Config",
    children: [
        { name: "Orthogonal Test", tag: "Orthogonal", children: [{ name: "Node A" }, { name: "Node B" }] },
        { name: "Function Test", tag: "Functions", children: [{ name: "Linear" }, { name: "Quadratic" }] }
    ]
};

// Fetch articles and graph for this KB
onMounted(async () => {
    try {
        const [res, graphRes] = await Promise.all([
            contentApi.list({ knowledge_base_id: props.kb.id }),
            fetch(`/api/kb/${props.kb.id}/graph`).then(r => r.json()).catch(() => [])
        ]);
        
        articles.value = res;
        
        // Transform Graph Res (Flat List) to Tree
        // If empty, use default
        if (!graphRes || graphRes.length === 0) {
             treeData.value = DEFAULT_TREE;
        } else {
             treeData.value = buildTree(graphRes);
        }

    } catch (e) {
        console.error("Failed to fetch dashboard data", e);
        treeData.value = DEFAULT_TREE; // Fallback
    } finally {
        loadingArticles.value = false;
    }
});

const buildTree = (nodes: any[]) => {
    const map = new Map();
    const roots: any[] = [];
    
    // Sort by rank
    nodes.sort((a: any, b: any) => a.rank - b.rank);

    nodes.forEach(n => {
        map.set(n.id, { ...n, name: n.label, tag: n.data?.tag, children: [] });
    });

    nodes.forEach(n => {
        const node = map.get(n.id);
        if (n.parent_id && map.has(n.parent_id)) {
            map.get(n.parent_id).children.push(node);
        } else {
            roots.push(node);
        }
    });
    
    // Return first root or a virtual root if multiple
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

const handleSettings = () => {
    alert("Settings dialog would open here (use KnowledgeModule logic)");
};

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
            <div class="h-[50vh] w-full flex flex-col md:flex-row border-b border-black/5 dark:border-white/5 bg-gray-50 dark:bg-[#0d1117] relative">
                <!-- V2 Badge -->
                <div class="absolute top-0 right-0 z-50 p-2 bg-red-500 text-white text-[10px] font-bold uppercase tracking-widest shadow-lg">
                    Math V2 / Orthogonal Test Engine
                </div>

                <!-- Left: Axiom Tree (Orthogonal) -->
                <div class="w-full md:w-2/3 h-full border-b md:border-b-0 md:border-r border-black/5 dark:border-white/5 relative overflow-hidden group">
                     <!-- Interactive Tree -->
                     <AxiomTree :kbId="kb.id" :treeData="treeData" @select-topic="handleTopicSelect" />
                     
                     <!-- Overlay Label -->
                     <div class="absolute top-4 left-6 pointer-events-none transition-opacity duration-300" :class="selectedTag ? 'opacity-0' : 'opacity-100'">
                        <span class="text-[10px] font-black uppercase tracking-widest text-ink/30">Orthogonal Structure (Test)</span>
                     </div>
                     
                     <!-- Active Filter Label -->
                     <div class="absolute bottom-4 right-6 pointer-events-none" v-if="selectedTag">
                        <span class="text-xs font-bold text-accent bg-accent/10 px-3 py-1 rounded-full backdrop-blur-sm border border-accent/20">
                            Filtering: {{ selectedTag }}
                        </span>
                     </div>
                </div>

                <!-- Right: Testing Panel -->
                <div class="w-full md:w-1/3 h-full p-6 flex flex-col bg-paper relative">
                    <div class="text-xs font-black uppercase tracking-widest text-ink/40 mb-4">Diagnostic Panel</div>
                    
                    <div class="flex-1 overflow-y-auto space-y-4">
                        <div class="p-4 border border-ink/10 rounded-lg">
                            <h4 class="font-bold text-sm mb-2">Orthogonal Graph</h4>
                            <p class="text-xs text-ink/60 mb-2">Verify that edges connect in a PCB-like (Manhattan) fashion.</p>
                            <div class="text-[10px] font-mono p-2 bg-ash/20 rounded">
                                d3.linkHorizontal() <br/>
                                PCB Trace Logic Active
                            </div>
                        </div>

                        <div class="p-4 border border-ink/10 rounded-lg">
                            <h4 class="font-bold text-sm mb-2">Function Plots</h4>
                            <p class="text-xs text-ink/60 mb-2">Open an article to verify 'function-plot' hydration.</p>
                            <button @click="handleCreate" class="w-full py-2 bg-accent/10 text-accent text-xs font-bold uppercase tracking-widest rounded hover:bg-accent/20 transition-colors">
                                Create Test Plot
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </template>

        <template #content>
            <div class="p-8">
                 <div class="flex items-center justify-between mb-8">
                    <div class="flex items-center gap-4">
                        <h2 class="text-xs font-black uppercase tracking-widest text-ink/40">Knowledge Base Contents</h2>
                        <span v-if="selectedTag" class="text-xs font-bold text-accent flex items-center gap-1 cursor-pointer hover:text-red-500" @click="handleTopicSelect(null)">
                            / {{ selectedTag }} <i class="ri-close-circle-fill"></i>
                        </span>
                    </div>
                    <span class="text-[10px] font-mono text-ink/30">{{ filteredArticles.length }} ENTRIES</span>
                 </div>
                 
                 <div v-if="loadingArticles" class="text-center py-20 animate-pulse text-xs font-bold uppercase tracking-widest text-ink/30">
                    Loading Indices...
                 </div>

                 <div v-else-if="filteredArticles.length === 0" class="text-center py-20 border border-dashed border-ink/10 rounded-xl">
                    <p class="text-ink/40 mb-4">No content found.</p>
                    <button @click="handleCreate" class="text-xs font-bold uppercase tracking-widest text-accent hover:underline">Start Test</button>
                 </div>

                 <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    <div v-for="article in filteredArticles" :key="article.id" 
                        @click="goArticle(article.id)"
                        class="h-32 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-lg p-5 hover:border-accent/40 hover:shadow-xl hover:shadow-accent/5 hover:-translate-y-1 transition-all duration-300 cursor-pointer group flex flex-col relative overflow-hidden backdrop-blur-sm">
                        
                        <div class="absolute inset-0 bg-gradient-to-br from-white/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                        <div class="w-8 h-1 bg-accent/20 mb-auto group-hover:bg-accent transition-colors"></div>
                        <h3 class="font-bold text-ink leading-tight mb-1 group-hover:text-accent transition-colors">{{ article.title }}</h3>
                        
                        <div class="flex items-center justify-between mt-auto">
                            <div class="text-[10px] font-mono text-ink/40 uppercase tracking-widest">{{ new Date(article.created_at).toLocaleDateString() }}</div>
                        </div>
                    </div>
                 </div>
            </div>
        </template>
    </KnowledgeBaseLayout>
</template>
