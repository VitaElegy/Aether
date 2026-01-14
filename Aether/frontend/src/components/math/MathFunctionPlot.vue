<script setup lang="ts">
import { onMounted, ref, watch, nextTick, onBeforeUnmount } from 'vue';
// import functionPlot from 'function-plot'; // Dynamic import used instead

interface Props {
  fn: string;
  xDomain?: [number, number];
  yDomain?: [number, number];
  width?: number;
  height?: number;
  interactive?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  xDomain: () => [-5, 5],
  yDomain: () => [-5, 5], 
  width: 300,
  height: 200,
  interactive: true
});

const emit = defineEmits(['click']);

const container = ref<HTMLElement | null>(null);
let instance: any = null;

const render = async () => {
    if (!container.value || !props.fn) return;
    
    // Dynamic import to prevent init crashes
    let functionPlot: any;
    try {
        const module = await import('function-plot');
        functionPlot = module.default || module;
    } catch (e) {
        console.error('Failed to load function-plot library', e);
        if (container.value) {
            container.value.innerHTML = `<div class="text-red-500 text-xs p-2">Lib Error: ${(e as Error).message}</div>`;
        }
        return;
    }

    // Clear previous content to be safe
    container.value.innerHTML = '';
    
    try {
        await nextTick();
        // Use container dimensions if explicit props not provided/responsive needed
        // For now using props
        
        instance = functionPlot({
            target: container.value,
            width: props.width,
            height: props.height,
            yAxis: { domain: props.yDomain },
            xAxis: { domain: props.xDomain },
            grid: true,
            data: [{
                fn: props.fn,
                color: 'currentColor',
                graphType: 'polyline'
            }],
            disableZoom: !props.interactive
        });
    } catch (e) {
        console.error("MathFunctionPlot error:", e);
        if (container.value) {
            container.value.innerHTML = `<div class="text-red-500 text-xs p-2">Plot Error: ${(e as Error).message}</div>`;
        }
    }
};

watch(() => [props.fn, props.width, props.height], render);

onMounted(render);

// Resize observer for responsive width if needed later
</script>

<template>
    <div class="math-function-plot relative select-none" 
         :class="{ 'cursor-move': interactive, 'cursor-pointer hover:ring-2 hover:ring-accent/20 transition-all': !interactive && $attrs.onClick }"
         @click="$emit('click')">
        <div ref="container" class="text-accent overflow-hidden rounded"></div>
    </div>
</template>

<style scoped>
/* Force text/stroke colors to match theme "current color" (text-accent) */
:deep(.function-plot text) {
    fill: currentColor !important;
    font-family: 'JetBrains Mono', monospace;
    font-size: 10px;
}
:deep(.function-plot path.line) {
    stroke-width: 2px;
}
:deep(.function-plot .domain) {
    stroke: currentColor;
    opacity: 0.2;
}
:deep(.function-plot .tick line) {
    stroke: currentColor;
    opacity: 0.1;
}
:deep(.function-plot .origin) {
    stroke: currentColor;
    opacity: 0.3;
}
</style>
