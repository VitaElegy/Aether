<script setup lang="ts">
import { ref, onMounted, computed, defineAsyncComponent, watch } from 'vue';
import { useRoute } from 'vue-router';
import { knowledgeApi } from '@/api/knowledge';
import { getDashboard } from '@/registries/read_layout_registry';

// Lazy load known dashboards to register them
// In a real plugin system these would be auto-discovered
const MathDashboard = defineAsyncComponent(() => import('@/components/dashboard/MathDashboard.vue'));

// Register immediately for now (Manual Registration)
import { registerDashboard } from '@/registries/read_layout_registry';
registerDashboard('math_v1', MathDashboard);

const route = useRoute();
const kbId = computed(() => route.params.id as string);
const kb = ref<any>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const activeLayout = computed(() => {
    if (!kb.value) return null;
    return getDashboard(kb.value.renderer_id);
});

const loadKB = async () => {
    loading.value = true;
    error.value = null;
    try {
        kb.value = await knowledgeApi.get(kbId.value);
    } catch (e) {
        console.error("Failed to load KB", e);
        error.value = "Failed to load Knowledge Base.";
    } finally {
        loading.value = false;
    }
};

onMounted(loadKB);
watch(kbId, loadKB);
</script>

<template>
    <div class="w-full h-screen bg-paper">
        <div v-if="loading" class="flex items-center justify-center h-full">
            <div class="animate-pulse text-accent text-xs font-black uppercase tracking-[0.4em]">Establishing Link...</div>
        </div>

        <div v-else-if="error" class="flex items-center justify-center h-full text-red-500">
            {{ error }}
        </div>

        <!-- Custom Dashboard -->
        <component 
            v-else-if="activeLayout" 
            :is="activeLayout" 
            :kb="kb"
            @refresh="loadKB"
        />

        <!-- Default Dashboard Fallback -->
        <div v-else class="p-20 text-center">
            <h1 class="text-4xl font-black mb-4">{{ kb?.title }}</h1>
            <p class="text-ink/50 mb-8">Standard Layout (Not implemented yet)</p>
            <pre class="text-left bg-ash/10 p-4 rounded">{{ kb }}</pre>
        </div>
    </div>
</template>
