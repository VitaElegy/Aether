<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import GhostSidebar from '@/components/math/GhostSidebar.vue';
import FocusStage from '@/components/math/FocusStage.vue';
import axios from 'axios';

const props = defineProps({
    article: { type: Object, required: true },
    loading: Boolean,
    canEdit: Boolean
});

const semanticNodes = ref([]);
const loadingGraph = ref(false);

const fetchGraph = async () => {
    if (!props.article?.id) return;
    
    loadingGraph.value = true;
    try {
        const token = localStorage.getItem('token');
        const res = await axios.get(`/api/graph/context?article_id=${props.article.id}`, {
            headers: { Authorization: `Bearer ${token}` }
        });
        semanticNodes.value = res.data;
    } catch (e) {
        console.error("Failed to fetch graph context", e);
    } finally {
        loadingGraph.value = false;
    }
};

onMounted(() => {
    fetchGraph();
});

watch(() => props.article, () => {
    fetchGraph();
}, { deep: true });
</script>

<template>
    <div class="flex h-screen w-full bg-paper overflow-hidden font-sans text-ink transition-colors duration-300 relative">
        <!-- Header Overlay -->
        <div class="absolute top-0 left-0 w-full p-4 flex items-center justify-between z-50 pointer-events-none">
            <div class="pointer-events-auto">
                <button @click="$router.back()" class="flex items-center gap-2 text-ink/50 hover:text-ink transition-colors bg-paper/20 hover:bg-paper/40 backdrop-blur-md px-4 py-2 rounded-full shadow-sm border border-ink/5">
                    <i class="ri-arrow-left-line"></i>
                    <span class="text-xs font-bold tracking-widest uppercase font-mono">Return</span>
                </button>
            </div>
        </div>

        <!-- LEFT: Ghost Sidebar -->
        <div class="w-64 h-full border-r border-ink/5 relative z-10 transition-all duration-500 opacity-60 hover:opacity-100 hover:w-72 bg-paper/50 backdrop-blur-sm">
             <GhostSidebar :nodes="semanticNodes" />
        </div>

        <!-- RIGHT: Focus Stage -->
        <div class="flex-1 h-full relative z-0">
             <div v-if="loadingGraph" class="absolute inset-0 flex items-center justify-center text-accent/50 text-xs font-mono uppercase tracking-widest animate-pulse">
                  Scanning Geometry...
             </div>
             <FocusStage v-else :nodes="semanticNodes" />
        </div>
    </div>
</template>
