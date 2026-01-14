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
    <div class="flex h-screen w-full bg-gray-50 dark:bg-[#0a0f14] overflow-hidden font-sans text-gray-900 dark:text-gray-200 transition-colors duration-300">
        <!-- LEFT: Ghost Sidebar -->
        <div class="w-64 h-full border-r border-black/5 dark:border-white/5 relative z-10 transition-all duration-500 opacity-60 dark:opacity-40 hover:opacity-100 hover:w-72 bg-white/50 dark:bg-[#0a0f14]/90 backdrop-blur-sm">
             <GhostSidebar :nodes="semanticNodes" />
        </div>

        <!-- RIGHT: Focus Stage -->
        <div class="flex-1 h-full relative z-0">
             <div v-if="loadingGraph" class="absolute inset-0 flex items-center justify-center text-cyan-500/50 text-xs font-mono uppercase">
                  Scanning Geometry...
             </div>
             <FocusStage v-else :nodes="semanticNodes" />
        </div>
    </div>
</template>
