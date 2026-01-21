<template>
  <div class="min-h-screen bg-bg-base text-text-primary flex flex-col font-sans">
    <!-- Top Navigation (VR Specific) -->
    <header class="h-14 border-b border-component-stroke bg-bg-surface px-4 flex items-center justify-between sticky top-0 z-50">
      <div class="flex items-center gap-4">
        <!-- Back to Global -->
        <button 
          @click="goBack"
          class="p-2 hover:bg-component-hover rounded-md text-text-secondary transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
        </button>
        
        <div class="flex flex-col">
          <h1 class="text-sm font-bold text-text-primary leading-tight">{{ kbTitle }}</h1>
          <span class="text-xs text-text-secondary">Vulnerability Research Support System</span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- Tools will go here -->
      </div>
    </header>

    <!-- Main Content Area (Kanban / Dashboard) -->
    <main class="flex-1 overflow-hidden relative">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { knowledgeApi } from '@/api/knowledge';

interface Props {
    article?: any; // The current node/article being viewed
}

const props = defineProps<Props>();
const router = useRouter();

const kbTitle = ref<string>('VR Knowledge Base');

watch(() => props.article, async (newVal) => {
    if (newVal?.knowledge_base_id) {
        try {
            const kb = await knowledgeApi.get(newVal.knowledge_base_id);
            kbTitle.value = kb.title;
        } catch (e) {
            console.error("Failed to load KB title", e);
        }
    }
}, { immediate: true });

const goBack = () => {
    // Determine where to go back. If we have history use it, else go home.
    if (window.history.length > 2) {
        router.back();
    } else {
        router.push('/');
    }
};
</script>
