<script setup lang="ts">
import { onMounted, ref, watch, nextTick } from 'vue';
import type { SemanticNode } from '@/utils/SemanticBlockParser';
import functionPlot from 'function-plot';

interface Props {
  node: SemanticNode;
  interactive?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  interactive: true
});

const emit = defineEmits(['click']);
const plotContainer = ref<HTMLElement | null>(null);

const getTypeColor = (type: string) => {
  switch (type.toLowerCase()) {
    case 'theorem': return 'text-purple-600 dark:text-purple-400 border-purple-200 dark:border-purple-500/20 bg-purple-50 dark:bg-purple-500/5';
    case 'axiom': return 'text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-500/20 bg-blue-50 dark:bg-blue-500/5';
    case 'function': return 'text-orange-600 dark:text-orange-400 border-orange-200 dark:border-orange-500/20 bg-orange-50 dark:bg-orange-500/5';
    case 'lemma': return 'text-teal-600 dark:text-teal-400 border-teal-200 dark:border-teal-500/20 bg-teal-50 dark:bg-teal-500/5';
    default: return 'text-gray-600 dark:text-gray-400 border-gray-200 dark:border-white/10 bg-gray-50 dark:bg-white/5';
  }
};

const renderPlot = async () => {
    if (!plotContainer.value) return;
    
    // Check if node has plotting metrics
    // Expecting metrics: { fn: "x^2", xDomain: [-2, 2], ... }
    const metrics = props.node.metrics || {};
    if (metrics.fn) {
        // Clear previous
        plotContainer.value.innerHTML = '';
        
        try {
            await nextTick();
            const width = plotContainer.value.clientWidth;
            const height = 150; // Fixed small height for card

            functionPlot({
                target: plotContainer.value,
                width,
                height,
                yAxis: { domain: [-5, 5] },
                xAxis: { domain: metrics.xDomain || [-5, 5] },
                grid: true,
                data: [{
                    fn: metrics.fn,
                    color: 'currentColor' // Use current text color for theme adaptability
                }],
                disableZoom: !props.interactive
            });
        } catch (e) {
            console.warn("Plotting failed for node", props.node.id, e);
        }
    }
};

onMounted(() => {
    renderPlot();
});

watch(() => props.node, renderPlot, { deep: true });

const handleClick = () => {
    if (props.interactive) {
        emit('click', props.node);
    }
};
</script>

<template>
  <div 
    class="relative group rounded-lg border transition-all duration-300 overflow-hidden bg-white dark:bg-[#161b22]"
    :class="[
      getTypeColor(node.type),
      interactive ? 'hover:shadow-md hover:scale-[1.02] cursor-pointer' : ''
    ]"
    @click="handleClick"
  >
    <!-- Header -->
    <div class="px-4 py-2 border-b border-inherit flex items-center justify-between">
      <span class="text-[10px] font-black uppercase tracking-widest opacity-80">{{ node.type }}</span>
      <span v-if="node.metrics?.id" class="text-[9px] font-mono opacity-50">{{ node.metrics.id }}</span>
    </div>

    <!-- Body -->
    <div class="p-4">
      <h3 class="font-bold text-lg leading-tight mb-2 text-gray-900 dark:text-gray-100">{{ node.title || 'Untitled Node' }}</h3>
      
      <!-- Content Preview (Text) -->
      <div class="text-xs text-gray-600 dark:text-gray-400 line-clamp-3 font-serif mb-3">
        {{ node.content }}
      </div>

      <!-- Plot Area -->
      <div v-if="node.metrics?.fn" class="w-full h-[150px] bg-gray-50 dark:bg-white/5 rounded relative text-accent">
          <div ref="plotContainer" class="w-full h-full"></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Scoped styles for function-plot tweaks if needed */
:deep(.function-plot text) {
    fill: currentColor !important;
    font-family: monospace;
}
:deep(.function-plot path.line) {
    stroke-width: 2px;
}
</style>
