<script setup lang="ts">
import { computed } from 'vue';
import MathFunctionPlot from './MathFunctionPlot.vue';

interface Props {
    nodeId?: string;
    nodeType?: 'theorem' | 'definition' | 'axiom' | 'proof' | 'link' | 'function';
    title?: string;
    content?: string;
    metrics?: any;
}

const props = defineProps<Props>();
const emit = defineEmits(['open-lab']);

const typeLabel = computed(() => {
    switch (props.nodeType) {
        case 'theorem': return 'Theorem';
        case 'definition': return 'Definition';
        case 'axiom': return 'Axiom';
        case 'proof': return 'Proof Strategy';
        case 'link': return 'Reference';
        case 'function': return 'Function';
        default: return 'Context';
    }
});

const typeColorClass = computed(() => {
    switch (props.nodeType) {
        case 'theorem': return 'text-purple-700 bg-purple-50 border-purple-100';
        case 'definition': return 'text-blue-700 bg-blue-50 border-blue-100';
        case 'axiom': return 'text-red-700 bg-red-50 border-red-100';
        case 'function': return 'text-orange-700 bg-orange-50 border-orange-100';
        default: return 'text-gray-700 bg-gray-50 border-gray-100';
    }
});
</script>

<template>
    <div class="h-full flex flex-col p-8 font-serif">
        <div v-if="!nodeId" class="flex-1 flex items-center justify-center text-ink/30 italic">
            Select a term to view context...
        </div>

        <div v-else class="animate-fade-in-up">
            <!-- Header -->
            <div class="mb-6">
                <span :class="['px-2 py-1 text-[10px] uppercase tracking-widest font-bold border rounded-sm', typeColorClass]">
                    {{ typeLabel }}
                </span>
                <h2 class="mt-4 text-2xl font-bold text-ink leading-tight">{{ title || 'Untitled Node' }}</h2>
                <div class="text-[10px] font-mono text-ink/40 mt-2 uppercase tracking-widest">ID: {{ nodeId }}</div>
            </div>

            <!-- Divider -->
            <div class="w-12 h-px bg-ink/10 mb-6"></div>

            <!-- Function Plot (if active) -->
            <div v-if="nodeType === 'function'" class="mb-6 rounded-lg border border-ink/5 bg-paper shadow-inner overflow-hidden">
                <MathFunctionPlot 
                    :fn="metrics?.fn || (content && content.length < 50 ? content : 'x^2')" 
                    :width="260"
                    :height="180"
                    :interactive="false"
                    @click="$emit('open-lab', metrics?.fn || (content && content.length < 50 ? content : 'x^2'))"
                />
            </div>

            <!-- Content Preview -->
            <div class="prose prose-sm prose-scholar font-serif text-ink/80 leading-relaxed">
                <p v-if="content">{{ content }}</p>
                <p v-else class="italic text-ink/40">No preview content available.</p>
            </div>

            <!-- Actions -->
            <div class="mt-8 pt-6 border-t border-ink/5 flex flex-col gap-3">
                <button class="w-full py-3 border border-ink/10 hover:border-ink/30 hover:bg-ink/5 transition-colors text-xs font-bold uppercase tracking-widest text-ink/60">
                    Navigate to Page
                </button>
                <button class="w-full py-3 border border-ink/10 hover:border-ink/30 hover:bg-ink/5 transition-colors text-xs font-bold uppercase tracking-widest text-ink/60">
                    Show Dependency Graph
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.animate-fade-in-up {
    animation: fadeInUp 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes fadeInUp {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

/* Scholar Typography Overrides */
.prose-scholar {
    --tw-prose-body: var(--color-ink);
    --tw-prose-headings: var(--color-ink);
}
</style>
