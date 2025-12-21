<script setup lang="ts">
import { computed, defineAsyncComponent, h } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';

// Define the shape of our content based on the Rust backend
// In a real project, we would generate these types automatically.
type ContentType = 'Markdown' | 'CodeSnippet' | 'Video' | 'Custom';

interface Props {
  type: ContentType;
  data: any; // The payload specific to the type
}

const props = defineProps<Props>();

// Strategy Pattern: Map types to Lazy-Loaded Components
// This keeps the initial bundle size small and allows for infinite plugin extension.
const rendererMap: Record<string, any> = {
  Markdown: defineAsyncComponent(() => import('./renderers/MarkdownRenderer.vue')),
  CodeSnippet: defineAsyncComponent(() => import('./renderers/CodeRenderer.vue')),
  Video: defineAsyncComponent(() => import('./renderers/VideoRenderer.vue')),
};

const activeComponent = computed(() => {
  const component = rendererMap[props.type];
  if (!component) {
    MessagePlugin.warning(`No renderer found for type: ${props.type}`);
    return null;
  }
  return component;
});
</script>

<template>
  <div class="aether-content-wrapper">
    <transition name="fade" mode="out-in">
      <component
        :is="activeComponent"
        v-if="activeComponent"
        v-bind="props.data"
        class="aether-renderer"
      />
      <div v-else class="fallback">
        Unknown Content Type: {{ type }}
      </div>
    </transition>
  </div>
</template>

<style scoped>
.aether-content-wrapper {
  /* TDesign Token usage for consistency */
  font-family: var(--td-font-family);
  color: var(--td-text-color-primary);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

