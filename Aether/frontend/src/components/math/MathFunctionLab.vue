<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import MathFunctionPlot from './MathFunctionPlot.vue';

interface Props {
  isOpen: boolean;
  initialFn: string;
  initialDescription?: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['close', 'update']);

const currentFn = ref(props.initialFn);
const currentDesc = ref(props.initialDescription || '');
const xMin = ref(-5);
const xMax = ref(5);
const yMin = ref(-5);
const yMax = ref(5);
const activeTab = ref<'controls' | 'description'>('controls');

// Update local state when prop changes
watch(() => props.initialFn, (newVal) => {
    currentFn.value = newVal;
});
watch(() => props.initialDescription, (newVal) => {
    // If description exists and fn hasn't changed, maybe prioritize description?
    // For now just update ref
    currentDesc.value = newVal || '';
    if (newVal) activeTab.value = 'description';
});

const close = () => {
    emit('close');
};

const updatePlot = () => {
    emit('update', currentFn.value);
};

// Handle ESC key to close
const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && props.isOpen) {
        close();
    }
};

onMounted(() => {
    window.addEventListener('keydown', handleKeydown);
});

onBeforeUnmount(() => {
    window.removeEventListener('keydown', handleKeydown);
});

</script>

<template>
    <div v-if="isOpen" class="scholar-theme fixed inset-0 z-[100] flex items-center justify-center font-serif text-ink">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-paper/90 backdrop-blur-sm" @click="close"></div>

        <!-- Main Card -->
        <div class="relative w-full max-w-5xl h-[80vh] bg-paper border border-ink/10 shadow-2xl flex flex-col md:flex-row rounded-lg overflow-hidden anim-scale-in">
            
            <!-- Left: Sidebar -->
            <div class="w-full md:w-1/3 border-r border-ink/10 bg-ash/10 flex flex-col overflow-hidden">
                <!-- Header -->
                <div class="p-8 pb-4">
                    <h2 class="font-serif text-2xl font-bold text-ink mb-2">Function Lab</h2>
                    <p class="text-xs text-ink/50 uppercase tracking-widest font-bold">Analysis & Modification</p>
                </div>

                <!-- Tabs -->
                <div class="flex border-b border-ink/10 px-8 gap-4">
                    <button 
                        @click="activeTab = 'controls'"
                        class="pb-2 text-xs font-bold uppercase tracking-widest transition-colors border-b-2"
                        :class="activeTab === 'controls' ? 'text-accent border-accent' : 'text-ink/40 border-transparent hover:text-ink/70'"
                    >
                        Controls
                    </button>
                    <button 
                        v-if="currentDesc"
                        @click="activeTab = 'description'"
                        class="pb-2 text-xs font-bold uppercase tracking-widest transition-colors border-b-2"
                        :class="activeTab === 'description' ? 'text-accent border-accent' : 'text-ink/40 border-transparent hover:text-ink/70'"
                    >
                        Description
                    </button>
                </div>

                <!-- Tab Content -->
                <div class="flex-1 overflow-y-auto p-8 custom-scrollbar">
                    
                    <!-- Controls Tab -->
                    <div v-if="activeTab === 'controls'" class="space-y-6">
                        <!-- Function Input -->
                        <div class="space-y-2">
                            <label class="text-xs font-bold text-ink/70 uppercase tracking-wider">Function f(x)</label>
                            <input v-model="currentFn" 
                                   class="w-full bg-white border border-ink/20 p-3 rounded font-mono text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-all text-ink"
                                   placeholder="e.g. x^2 + sin(x)"
                                   @keydown.enter="updatePlot" />
                        </div>

                        <!-- Domain Controls -->
                        <div class="space-y-2">
                            <label class="text-xs font-bold text-ink/70 uppercase tracking-wider">X Domain</label>
                            <div class="flex gap-2">
                                <input v-model.number="xMin" type="number" class="w-1/2 bg-white border border-ink/20 p-2 rounded font-mono text-sm text-ink" />
                                <input v-model.number="xMax" type="number" class="w-1/2 bg-white border border-ink/20 p-2 rounded font-mono text-sm text-ink" />
                            </div>
                        </div>

                        <!-- Range Controls -->
                        <div class="space-y-2">
                            <label class="text-xs font-bold text-ink/70 uppercase tracking-wider">Y Domain</label>
                            <div class="flex gap-2">
                                <input v-model.number="yMin" type="number" class="w-1/2 bg-white border border-ink/20 p-2 rounded font-mono text-sm text-ink" />
                                <input v-model.number="yMax" type="number" class="w-1/2 bg-white border border-ink/20 p-2 rounded font-mono text-sm text-ink" />
                            </div>
                        </div>

                         <div class="mt-8">
                            <button @click="updatePlot" 
                                    class="w-full py-3 bg-accent text-white font-bold uppercase tracking-widest text-xs rounded hover:bg-accent/90 transition-colors shadow-lg">
                                Update Plot
                            </button>
                            <p class="text-center text-[10px] text-ink/40 mt-4">
                                Press <span class="font-mono bg-ink/10 px-1 rounded">Enter</span> to update
                            </p>
                        </div>
                    </div>

                    <!-- Description Tab -->
                    <div v-else-if="activeTab === 'description'" class="prose prose-sm prose-scholar">
                        <div v-html="currentDesc"></div>
                    </div>

                </div>
            </div>

            <!-- Right: Visualization -->
            <div class="flex-1 bg-white relative flex items-center justify-center p-8 overflow-hidden">
                <!-- Close Button -->
                <button @click="close" class="absolute top-4 right-4 p-2 text-ink/30 hover:text-ink transition-colors z-10">
                    <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>

                <!-- Plot Wrapper -->
                <div class="w-full h-full flex items-center justify-center">
                    <MathFunctionPlot 
                        :fn="currentFn"
                        :width="600"
                        :height="400"
                        :x-domain="[xMin, xMax]"
                        :y-domain="[yMin, yMax]"
                        :interactive="true"
                    />
                </div>
            </div>

        </div>
    </div>
</template>

<style scoped>
/* SCHOLAR THEME VARIABLES (Copied from Layout for Teleport scope) */
.scholar-theme {
    --color-paper: 253 251 247; 
    --color-ink: 44 44 44;
    --color-accent: 50 80 120; 
}

.bg-ash\/10 { background-color: rgba(0,0,0,0.02); }

.anim-scale-in {
    animation: scaleIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes scaleIn {
    from { opacity: 0; transform: scale(0.95) translateY(10px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
}

/* Scrollbar Tweaks */
.custom-scrollbar::-webkit-scrollbar { width: 6px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.1); border-radius: 3px; }

/* Prose Scholar Logic */
.prose-scholar :deep(h1),
.prose-scholar :deep(h2),
.prose-scholar :deep(h3) {
    font-family: 'Noto Serif SC', 'Playfair Display', serif;
    font-weight: 500;
}
.prose-scholar :deep(p) { margin-bottom: 0.8em; }
</style>
