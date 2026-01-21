<template>
    <div class="my-6 p-4 border border-border/50 rounded-lg bg-paper-2 transition-all hover:border-accent/30 relative group">
        <!-- Label -->
        <div class="flex items-center gap-2 mb-2">
            <span class="text-xs font-bold uppercase tracking-wider text-accent/80 font-mono">
                {{ payload.math_type }}
            </span>
            <span v-if="payload.label" class="text-xs text-ink-3">
                ({{ payload.label }})
            </span>
        </div>

        <!-- KaTeX Render -->
        <div ref="mathContainer" class="text-lg text-ink font-serif overflow-x-auto py-2"></div>

        <!-- Raw (Hidden by default, shown on hover/edit could go here) -->
        <div class="hidden group-hover:block absolute top-2 right-2 opacity-50 text-xs font-mono text-ink-3">
            Source: {{ payload.latex }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import katex from 'katex';
import 'katex/dist/katex.min.css';

const props = defineProps<{
    block: any;
}>();

const payload = computed(() => props.block.payload || {});
const mathContainer = ref<HTMLElement | null>(null);

const renderMath = () => {
    if (mathContainer.value && payload.value.latex) {
        try {
            katex.render(payload.value.latex, mathContainer.value, {
                throwOnError: false,
                displayMode: true
            });
        } catch (e) {
            console.error("KaTeX Error", e);
            mathContainer.value.innerText = payload.value.latex;
        }
    }
};

onMounted(renderMath);
watch(() => payload.value.latex, renderMath);
</script>
