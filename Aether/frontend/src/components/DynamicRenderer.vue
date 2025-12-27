<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue';

// Define the shape of our content based on the Rust backend
type ContentType = 'Markdown' | 'CodeSnippet' | 'Video' | 'Custom';

interface Props {
    type: ContentType;
    data: any; // The payload specific to the type
}

const props = defineProps<Props>();

const rendererMap: Record<string, any> = {
    Markdown: defineAsyncComponent(() => import('./renderers/MarkdownRenderer.vue')),
    CodeSnippet: defineAsyncComponent(() => import('./renderers/CodeRenderer.vue')),
    Video: defineAsyncComponent(() => import('./renderers/VideoRenderer.vue')),
};

const activeComponent = computed(() => {
    return rendererMap[props.type] || null;
});
</script>

<template>
    <div class="w-full">
        <transition name="fade" mode="out-in">
            <component :is="activeComponent" v-if="activeComponent" v-bind="props.data" />
            <div v-else class="p-4 bg-ash text-xs font-mono text-ink/40 border border-ash/50">
                Unsupported Type: {{ type }}
            </div>
        </transition>
    </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
