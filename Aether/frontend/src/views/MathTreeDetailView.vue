<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { knowledgeApi } from '@/api/knowledge';
import AxiomTree from '@/components/math/AxiomTree.vue';

const route = useRoute();
const router = useRouter();
const kbId = computed(() => route.params.id as string);
const kb = ref<any>(null);
const treeData = ref<any | null>(null);
const isEditMode = ref(false);
const showAddModal = ref(false);
const newNodeLabel = ref("");

const selectedNode = ref<any>(null);

const loadData = async () => {
    try {
        const [kbRes, graphRes] = await Promise.all([
            knowledgeApi.get(kbId.value),
            fetch(`/api/kb/${kbId.value}/graph`).then(r => r.json()).catch(() => [])
        ]);
        kb.value = kbRes;
        
        if (!graphRes || graphRes.length === 0) {
            treeData.value = { name: kbRes.title, children: [] };
        } else {
            treeData.value = buildTree(graphRes);
        }
    } catch (e) {
        console.error("Failed to load data", e);
    }
};

const buildTree = (nodes: any[]) => {
    const map = new Map();
    const roots: any[] = [];
    nodes.sort((a: any, b: any) => a.rank - b.rank);
    nodes.forEach(n => map.set(n.id, { ...n, name: n.label, tag: n.data?.tag, children: [] }));
    nodes.forEach(n => {
        const node = map.get(n.id);
        if (n.parent_id && map.has(n.parent_id)) {
            map.get(n.parent_id).children.push(node);
        } else {
            roots.push(node);
        }
    });
    // Virtual root if multiple
    if (roots.length === 1) return roots[0];
    return { name: kb.value?.title || "Root", children: roots };
};

const handleNodeClick = (node: any) => {
    selectedNode.value = node;
};

const createNode = async () => {
    if (!newNodeLabel.value) return;
    try {
        await fetch('/api/nodes', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                kb_id: kbId.value,
                parent_id: selectedNode.value?.id || null, // If no parent, it's root
                label: newNodeLabel.value,
                data: { tag: newNodeLabel.value } 
            })
        });
        newNodeLabel.value = "";
        showAddModal.value = false;
        await loadData(); // Reload
    } catch (e) {
        console.error(e);
    }
};

const deleteNode = async () => {
    if (!selectedNode.value?.id) return;
    if (!confirm("Are you sure? This node and its sub-nodes will be deleted.")) return;
    try {
        await fetch(`/api/nodes/${selectedNode.value.id}`, { method: 'DELETE' });
        selectedNode.value = null;
        await loadData();
    } catch (e) {
        console.error(e);
    }
};

onMounted(loadData);
</script>

<template>
    <div class="w-full h-screen bg-paper text-ink flex overflow-hidden relative transition-colors duration-300">
        <!-- Main Stage: Tree -->
        <div class="flex-1 h-full relative">
            <!-- Header/Nav Overlay -->
            <div class="absolute top-0 left-0 w-full p-4 flex items-center justify-between z-10 pointer-events-none">
                <div class="pointer-events-auto flex items-center gap-4">
                    <button @click="router.push(`/kb/${kbId}`)" class="flex items-center gap-2 text-ink/50 hover:text-ink transition-colors bg-paper/20 hover:bg-paper/40 backdrop-blur px-4 py-2 rounded-full shadow-sm">
                        <i class="ri-arrow-left-line"></i>
                        <span class="text-sm font-bold tracking-widest uppercase">Back</span>
                    </button>
                    <!-- Edit Toggle -->
                    <button @click="isEditMode = !isEditMode" 
                        class="px-4 py-2 rounded-full backdrop-blur border transition-colors flex items-center gap-2 shadow-sm"
                        :class="isEditMode ? 'bg-accent text-white border-accent' : 'bg-paper/20 text-ink/50 border-ink/10 hover:border-ink/30 hover:text-ink'">
                        <i class="ri-edit-circle-line"></i>
                        <span class="text-sm font-bold tracking-widest uppercase">{{ isEditMode ? 'Editing' : 'View Only' }}</span>
                    </button>
                </div>
                <div class="pointer-events-auto">
                     <h1 class="text-3xl font-black text-ink opacity-20 select-none" style="font-family: 'Noto Serif SC', serif;">{{ kb?.title }} <span class="font-thin opacity-50">/</span> AXIOM TREE</h1>
                </div>
            </div>

            <AxiomTree :kbId="kbId" :treeData="treeData" @node-click="handleNodeClick" class="w-full h-full" />
            
            <!-- Add Node Modal (Simple Overlay) -->
            <div v-if="showAddModal" class="absolute inset-0 bg-ink/10 flex items-center justify-center z-50 backdrop-blur-sm pointer-events-auto">
                <div class="bg-paper border border-ink/10 p-6 rounded-xl w-96 shadow-2xl">
                    <h3 class="text-lg font-bold text-ink mb-4" style="font-family: 'Noto Serif SC', serif;">Add Child Node</h3>
                    <p class="text-xs text-ink/50 mb-4 font-mono">PARENT: <span class="text-accent">{{ selectedNode?.name || 'ROOT' }}</span></p>
                    <input v-model="newNodeLabel" placeholder="Node Name" 
                           class="w-full bg-ink/5 border border-ink/10 text-ink p-2 rounded mb-4 focus:border-accent focus:outline-none font-mono text-sm" />
                    <div class="flex justify-end gap-2">
                        <button @click="showAddModal = false" class="px-4 py-2 text-ink/50 hover:text-ink font-mono text-xs uppercase">Cancel</button>
                        <button @click="createNode" class="px-4 py-2 bg-accent text-white font-bold rounded hover:bg-accent/90 font-mono text-xs uppercase">Create</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Right Sidebar: Context -->
        <div class="w-96 border-l border-ink/5 bg-paper/80 backdrop-blur-xl h-full flex flex-col transition-all duration-500 ease-[cubic-bezier(0.23,1,0.32,1)] transform shadow-2xl"
             :class="selectedNode ? 'translate-x-0' : 'translate-x-full absolute right-0'">
            
            <div class="p-8 border-b border-ink/5 flex items-center justify-between">
                <div>
                     <div class="text-[10px] font-mono text-ink/30 mb-1 uppercase tracking-widest">Selected Entity</div>
                     <h2 class="font-bold text-2xl text-ink/90 leading-tight" style="font-family: 'Noto Serif SC', serif;">{{ selectedNode?.name || 'Select Node' }}</h2>
                </div>
                <button @click="selectedNode = null" class="text-ink/20 hover:text-ink transition-colors">
                    <i class="ri-close-line text-2xl"></i>
                </button>
            </div>

            <div class="flex-1 p-6 overflow-y-auto">
                <div v-if="selectedNode">
                    <p class="text-ink/60 text-sm leading-relaxed mb-6 font-serif">
                        Mathematics foundation properties for <span class="font-bold border-b border-ink/20 pb-0.5">{{ selectedNode.name }}</span>.
                        This panel will host graph actions, related theorems, and definition shortcuts.
                    </p>

                    <div class="space-y-4">
                        <h3 class="text-xs font-bold text-ink/30 uppercase tracking-widest font-mono">Actions</h3>
                        <button class="w-full flex items-center gap-3 p-3 rounded-lg bg-ink/5 hover:bg-ink/10 border border-ink/5 transition-colors group">
                            <i class="ri-node-tree text-accent"></i>
                            <div class="text-left">
                                <div class="text-sm font-bold text-ink group-hover:text-accent">Expand Sub-Graph</div>
                                <div class="text-[10px] text-ink/30">Load child dependencies</div>
                            </div>
                        </button>
                         <button class="w-full flex items-center gap-3 p-3 rounded-lg bg-ink/5 hover:bg-ink/10 border border-ink/5 transition-colors group">
                            <i class="ri-file-list-3-line text-accent"></i>
                            <div class="text-left">
                                <div class="text-sm font-bold text-ink group-hover:text-accent">View Related Articles</div>
                                <div class="text-[10px] text-ink/30">Find content tagged with {{ selectedNode.tag }}</div>
                            </div>
                        </button>
                    </div>

                    <!-- Edit Actions -->
                    <div v-if="isEditMode" class="mt-8 pt-8 border-t border-ink/10 space-y-4">
                        <h3 class="text-xs font-bold text-ink/30 uppercase tracking-widest font-mono">Editing</h3>
                        
                        <button @click="showAddModal = true" class="w-full flex items-center gap-3 p-3 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/20 transition-colors group">
                            <i class="ri-add-circle-line text-emerald-500"></i>
                            <div class="text-left">
                                <div class="text-sm font-bold text-emerald-700 dark:text-emerald-300 group-hover:text-emerald-600">Add Child</div>
                            </div>
                        </button>

                        <button @click="deleteNode" class="w-full flex items-center gap-3 p-3 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 transition-colors group">
                            <i class="ri-delete-bin-line text-rose-500"></i>
                            <div class="text-left">
                                <div class="text-sm font-bold text-rose-700 dark:text-rose-300 group-hover:text-rose-600">Delete Node</div>
                            </div>
                        </button>
                    </div>
                </div>
                <div v-else class="h-full flex flex-col items-center justify-center text-ink/20">
                    <i class="ri-node-tree text-4xl mb-4 opacity-50"></i>
                    <p class="text-xs uppercase tracking-widest font-bold">Select a node to view details</p>
                </div>
            </div>
        </div>
    </div>
</template>
