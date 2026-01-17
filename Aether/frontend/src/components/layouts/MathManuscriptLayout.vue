<script setup lang="ts">
import { ref, computed, onUpdated, nextTick, watch, onMounted, onBeforeUnmount } from 'vue';
import { useRouter } from 'vue-router';
// We'll import the View Component that renders the article content (reusing reuse StandardReadLayout logic or creating new?)
// For now, let's assume we wrap the slot or a specific render component.
// To keep it simple and consistent with Aether architecture, this Layout will render the <router-view> or specific content components.
// But usually Layouts in Aether wrap the content. 

import MathContextPanel from '../math/MathContextPanel.vue';
import MathIndexPanel from '../math/MathIndexPanel.vue';
import TopNavBar from '@/components/TopNavBar.vue';
import DynamicRenderer from '@/components/DynamicRenderer.vue';
import MathFunctionLab from '../math/MathFunctionLab.vue';

interface Props {
    article: any;
    loading: boolean;
}

const props = defineProps<Props>();
const router = useRouter();

// State
const isIndexOpen = ref(false);
const isContextOpen = ref(false);
const selectedContext = ref<{ id: string, type: any, title: string, content: string } | null>(null);

// Lab State
const isLabOpen = ref(false);
const labFn = ref('x^2');
const labDescription = ref('');

const openLab = (fn: string, description: string = '') => {
    labFn.value = fn;
    labDescription.value = description;
    isLabOpen.value = true;
};

// ...

// Actions
const toggleIndex = () => {
    isIndexOpen.value = !isIndexOpen.value;
};

const openContext = (data: any) => {
    selectedContext.value = data;
    isContextOpen.value = true;
};

const closeContext = () => {
    isContextOpen.value = false;
    selectedContext.value = null;
};

// Hydration Logic (Simplified & Safer)
const scrollContainer = ref<HTMLElement | null>(null);

const hydratePlotsInNode = async () => {
    // Dynamic import to prevent init crashes
    let functionPlot: any;
    try {
        const module = await import('function-plot');
        functionPlot = module.default || module;
    } catch (e) {
        console.error('Failed to load function-plot library', e);
        return;
    }

    // Select all inline plot containers
    // We check the entire document to be safe, or scope to scrollContainer if available
    const root = scrollContainer.value || document;
    const targets = root.querySelectorAll('.math-function-inline');
    
    targets.forEach((el) => {
        const target = el as HTMLElement;
        const fn = target.dataset.fn;
        const rawDesc = target.dataset.description || '';
        const isHydrated = target.dataset.hydrated === 'true';

        // Check children.length instead of hasChildNodes to ignore comments/whitespace
        if (fn && !isHydrated && target.children.length === 0) {
            try {
                // MARK AS HYDRATED IMMEDIATELY to prevent race conditions
                target.dataset.hydrated = 'true';
                
                // Decode Description
                let description = '';
                if (rawDesc) {
                    try {
                        description = decodeURIComponent(rawDesc);
                    } catch (e) { console.error('Failed to decode description', e); }
                }

                // Add click listener for Lab
                target.style.cursor = 'pointer';
                target.title = 'Click to analyze in Lab';
                target.addEventListener('click', () => openLab(fn, description));

                const width = target.clientWidth || 600; 
                functionPlot({
                    target: target,
                    width: width,
                    height: 200,
                    yAxis: { domain: [-5, 5] },
                    xAxis: { domain: [-5, 5] },
                    grid: true,
                    data: [{
                        fn: fn,
                        color: 'currentColor'
                    }],
                    disableZoom: false
                });
            } catch (e) {
                console.error('Failed to hydrate plot', fn, e);
                // On error, maybe remove attribute so we can retry? 
                // Or leave it to prevent infinite error loops.
                // target.removeAttribute('data-hydrated');
            }
        }
    });
};

// ...

// Template Usage Update
// <MathFunctionLab 
//                 :is-open="isLabOpen" 
//                 :initial-fn="labFn" 
//                 :initial-description="labDescription"
//                 @close="isLabOpen = false"
//                 @update="(newFn) => labFn = newFn"
//             />

// Robust Hydration Triggers
onMounted(() => {
    // Try immediately
    hydratePlotsInNode();
    // Try again after a short delay to allow layout to settle
    setTimeout(hydratePlotsInNode, 500);
    // And one more time for good measure (e.g. valid for slow network/rendering)
    setTimeout(hydratePlotsInNode, 1500);
});

// Watch for article content changes
watch(() => props.article, async () => {
    await nextTick();
    hydratePlotsInNode();
    setTimeout(hydratePlotsInNode, 300);
}, { deep: true });

// CSS Grid State
const gridStyle = computed(() => {
    // Left | Center | Right
    // 0px / 280px | 1fr | 0px / 320px
    const leftWidth = isIndexOpen.value ? '280px' : '0px';
    const rightWidth = isContextOpen.value ? '320px' : '0px';
    
    return {
        display: 'grid',
        gridTemplateColumns: `${leftWidth} 1fr ${rightWidth}`,
        transition: 'grid-template-columns 0.5s cubic-bezier(0.19, 1, 0.22, 1)' // ease-out-expo
    };
});

// Markdown Pre-processing is now handled by MarkdownRenderer extensions (marked.use)
// This ensures that inner content (especially math) is parsed correctly by the lexer.

// Mock Logic to simulate context opening (since we haven't wired up the markdown parser yet)
const simulateLinkClick = () => {
    openContext({
        id: 'thm:1.2',
        type: 'theorem',
        title: 'Uniqueness of Identity',
        content: 'In any group G, there exists a unique element e such that eg = ge = g for all g in G.'
    });
};
</script>

<template>
    <div class="scholar-theme w-full h-screen bg-paper text-ink overflow-hidden font-serif">
        <div class="w-full h-full" :style="gridStyle">
            
            <!-- LEFT PANEL (Index) -->
            <div class="h-full overflow-hidden bg-paper border-r border-ink/5 relative z-20" 
                 :class="{ 'opacity-0 pointer-events-none': !isIndexOpen, 'duration-300 transition-opacity': true }">
                <MathIndexPanel />
            </div>

            <!-- CENTER PANEL (Manuscript) -->
            <div class="h-full flex flex-col relative bg-paper z-10 shadow-xl transition-shadow duration-500 min-h-0 overflow-hidden"
                 :class="{ 'shadow-none': !isIndexOpen && !isContextOpen }">
                
                <!-- Navigation -->
                <TopNavBar class="!absolute !bg-transparent !backdrop-blur-none mix-blend-normal z-50">
                    <template #left>
                        <button @click="toggleIndex" class="ml-4 group flex items-baseline gap-2 text-ink/30 hover:text-ink transition-colors">
                            <span class="font-serif text-sm italic border-b border-transparent group-hover:border-ink/30 transition-all">Index</span>
                            <span class="text-[10px] opacity-0 group-hover:opacity-100 transition-opacity font-mono">{{ isIndexOpen ? '(-)' : '(+)' }}</span>
                        </button>
                    </template>
                </TopNavBar>

                <!-- Scrollable Content Area -->
                <!-- Removed 'flex justify-center' which can break overflow. Added ref="scrollContainer" -->
                <div ref="scrollContainer" class="flex-1 min-h-0 overflow-y-auto w-full custom-scrollbar relative">
                    <article class="w-full max-w-3xl mx-auto px-12 py-32 prose prose-lg prose-scholar">
                        
                        <!-- Header -->
                        <div class="text-center mb-24 anim-enter">
                           <!-- ... -->
                            <div class="text-xs font-bold uppercase tracking-[0.3em] text-ink/30 mb-6">Chapter One</div>
                            <h1 class="font-serif font-medium text-5xl text-ink mb-6 !leading-tight">
                                {{ article?.title || 'Untitled Manuscript' }}
                            </h1>
                            
                            <!-- Author Info (Paper Style) -->
                            <div v-if="article?.author_name" class="flex items-center justify-center gap-2 mb-8 text-ink/50 font-serif text-lg italic anim-enter delay-100">
                                <span>By</span>
                                <router-link 
                                    :to="`/user/${article.author_id}`" 
                                    class="hover:text-accent hover:underline decoration-1 underline-offset-4 transition-colors"
                                >
                                    {{ article.author_name }}
                                </router-link>
                            </div>

                            <div class="w-12 h-1 bg-ink/10 mx-auto"></div>
                        </div>

                        <!-- Content Render Slot -->
                        <div class="anim-enter delay-100 relative">
                             <!-- Direct Rendering: Pass raw body, MarkdownRenderer handles extensions now -->
                             <DynamicRenderer 
                                :type="'Markdown'" 
                                :data="{ content: (typeof article?.body === 'string' ? article.body : '') }" 
                             />

                             <!-- TEMP: Debug interaction -->
                             <hr class="my-12 border-ink/10"/>
                             <p class="text-sm text-ink/50 text-center font-mono">
                                [Debug Interaction Area] <br/>
                                <button @click="simulateLinkClick" class="text-accent underline hover:text-accent/80 transition-colors">
                                    Click here to simulate a Theorem Link
                                </button>
                             </p>
                        </div>
                        
                    </article>
                </div>

            </div>

            <!-- RIGHT PANEL -->
            <div class="h-full overflow-hidden bg-paper border-l border-ink/5 relative z-20">
                <div class="h-full w-full relative">
                    <!-- Close Button -->
                    <button v-if="isContextOpen" @click="closeContext" class="absolute top-4 right-4 z-50 text-ink/30 hover:text-ink transition-colors">
                        <span class="sr-only">Close</span>
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M6 18L18 6M6 6l12 12"></path></svg>
                    </button>

                    <MathContextPanel 
                        :nodeId="selectedContext?.id"
                        :nodeType="selectedContext?.type"
                        :title="selectedContext?.title"
                        :content="selectedContext?.content"
                        @open-lab="openLab"
                    />
                </div>
            </div>

        </div>

        <!-- Function Lab Modal -->
        <Teleport to="body">
            <MathFunctionLab 
                :is-open="isLabOpen" 
                :initial-fn="labFn" 
                :initial-description="labDescription"
                @close="isLabOpen = false"
                @update="(newFn) => labFn = newFn"
            />
        </Teleport>
    </div>
</template>

<style scoped>
/* SCHOLAR THEME OVERRIDES */
.scholar-theme {
    /* Rice White / Warm Paper */
    --color-paper: 253 251 247; 
    /* Soft Black Ink */
    --color-ink: 44 44 44;
    /* Muted Accent (Deep Blue) */
    --color-accent: 50 80 120; 
}

/* Scrollbar hiding for that "Clean Paper" look, but usability maintained */
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(0,0,0,0.05);
    border-radius: 3px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
    background: rgba(0,0,0,0.1);
}

/* Animations */
.anim-enter {
    animation: fadeInUp 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    opacity: 0;
    transform: translateY(20px);
}
.delay-100 { animation-delay: 0.1s; }

@keyframes fadeInUp {
    to { opacity: 1; transform: translateY(0); }
}

/* Typography Tweaks for Scholar Mode */
.prose-scholar :deep(h1),
.prose-scholar :deep(h2),
.prose-scholar :deep(h3) {
    font-family: 'Noto Serif SC', 'Playfair Display', serif;
    font-weight: 500;
}

.prose-scholar :deep(p) {
    font-family: 'Noto Serif SC', 'Times New Roman', serif;
    font-size: 1.1rem;
    line-height: 1.8;
}
</style>
