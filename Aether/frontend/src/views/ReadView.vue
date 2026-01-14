<script setup lang="ts">
import { ref, defineAsyncComponent, onMounted, watch, computed, onErrorCaptured } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useContent } from '@/composables/useContent';
import { knowledgeApi } from '@/api/knowledge';
import { getLayout } from '@/registries/read_layout_registry';

const RendererErrorLayout = defineAsyncComponent(() => import('@/components/layouts/RendererErrorLayout.vue'));

const route = useRoute();
const router = useRouter();

// Composable handles Article Fetching & Auth checks
const { article, loading, isAuthor, canEdit, load } = useContent();

// Local State for KB Details (Renderer)
const rendererId = ref<string | null>(null);
const renderError = ref<Error | null>(null);

const id = route.params.id as string;

// Resolve Layout from Registry
const layout = computed(() => {
    if (renderError.value) return RendererErrorLayout;
    return getLayout(rendererId.value);
});

onErrorCaptured((err) => {
    console.error("ReadView: Layout rendering failed", err);
    renderError.value = err as Error;
    return false; // Prevent propagation
});

const loadContext = async () => {
    // Reset error on load
    renderError.value = null;
    
    if (article.value?.knowledge_base_id) {
        try {
            const kb = await knowledgeApi.get(article.value.knowledge_base_id);
            rendererId.value = kb.renderer_id || 'default';
        } catch (e) {
            console.warn("ReadView: Failed to fetch KB details", e);
            rendererId.value = 'default';
        }
    } else {
        rendererId.value = 'default';
    }
    
    // Inject KB Details back into article object so layouts can access them easily if needed
    if (article.value) {
        (article.value as any).knowledgeBase = { renderer_id: rendererId.value };
    }
};

onMounted(async () => {
    await load(id);
    await loadContext();
});

watch(() => route.params.id, async (newId) => {
    if (newId) {
        renderError.value = null;
        await load(newId as string);
        await loadContext();
    }
});
</script>

<template>
    <div class="h-screen w-full">
        <!-- Loading State -->
        <div v-if="loading && !article" class="h-full flex items-center justify-center bg-paper">
             <div class="animate-pulse text-accent text-xs font-black uppercase tracking-[0.4em]">Establishing Uplink...</div>
        </div>

        <div v-else-if="!article" class="h-full flex items-center justify-center bg-paper">
             <div class="text-ink/40 text-xs font-black uppercase tracking-[0.2em]">Signal Lost (404)</div>
        </div>

        <!-- Dynamic Layout Switcher -->
        <component 
            v-else
            :is="layout"
            :article="article"
            :loading="loading"
            :canEdit="canEdit"
            :rendererId="rendererId || 'default'"
            :error="renderError"
        />
    </div>
</template>
