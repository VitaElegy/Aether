<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import ComputedTreeGraph from '../math/ComputedTreeGraph.vue'; 
import SemanticNodeCard from '../math/SemanticNodeCard.vue';
import type { SemanticNode } from '@/utils/SemanticBlockParser';

interface Props {
    article: any;
    loading: boolean;
    canEdit: boolean;
}

const props = defineProps<Props>();
const router = useRouter();
const selectedNode = ref<SemanticNode | null>(null);

const handleEdit = () => {
    if (props.article) {
         router.push(`/editor/${props.article.id}`);
    }
};

const onNodeSelected = (node: SemanticNode) => {
    selectedNode.value = node;
};

const navigateToNode = (node: SemanticNode) => {
    console.log("Navigating to node:", node.id);
    // router.push(`/node/${node.id}`); 
};
</script>

<template>
    <div class="h-screen w-full bg-paper text-ink relative overflow-hidden flex flex-col">
        <!-- Math Navigation Bar (Minimalist) -->
        <div class="absolute top-0 left-0 w-full p-6 flex justify-between items-start z-50 pointer-events-none">
            <!-- Left Anchor: Back & Context -->
            <div class="pointer-events-auto flex flex-col gap-2">
                <button @click="router.push('/space')" 
                    class="group flex items-center gap-2 text-ink/40 hover:text-accent transition-colors">
                    <i class="ri-arrow-left-line text-xl group-hover:-translate-x-1 transition-transform"></i>
                    <span class="text-xs font-black uppercase tracking-widest">Self Space</span>
                </button>
                
                <h1 v-if="article" class="text-3xl font-black uppercase tracking-tighter leading-none text-ink">
                    {{ article.title }}
                </h1>
                <span v-if="article?.knowledgeBase?.renderer_id" class="text-[10px] font-mono uppercase tracking-[0.2em] text-ink/30">
                    Math Knowledge System v2.0
                </span>
            </div>

            <!-- Right Anchor: Tools -->
            <div class="pointer-events-auto flex items-center gap-4">
                <button v-if="canEdit" @click="handleEdit"
                     class="px-4 py-2 border border-ash/50 hover:border-accent text-[10px] font-black uppercase tracking-widest text-ink/50 hover:text-accent transition-all bg-paper/80 backdrop-blur-sm rounded-sm">
                    Modify Node
                </button>
            </div>
        </div>

        <!-- Main Canvas Area -->
        <div class="flex-1 relative bg-grid-ash/5 flex">
            
            <!-- Graph Container -->
            <div class="flex-1 relative">
                <div v-if="loading" class="absolute inset-0 flex items-center justify-center">
                     <div class="animate-pulse text-accent text-xs font-black uppercase tracking-[0.4em]">Computing Topology...</div>
                </div>
                
                <div v-else class="w-full h-full text-ink/20 font-mono text-sm relative">
                    <ComputedTreeGraph 
                        :article="article" 
                        @node-selected="onNodeSelected"
                    />
                    
                    <div class="absolute bottom-4 left-4 text-[10px] opacity-50 pointer-events-none">
                        Renderer ID: {{ article?.knowledgeBase?.renderer_id || 'Unknown' }}
                    </div>
                </div>
            </div>

            <!-- Detail Sidebar (Right) -->
            <div v-if="selectedNode" class="w-80 border-l border-ash/10 bg-paper/90 backdrop-blur shadow-2xl z-40 p-6 flex flex-col gap-6 overflow-y-auto transition-all animate-slide-left">
                <div class="flex items-center justify-between pb-4 border-b border-ash/10">
                    <span class="text-[10px] font-black uppercase tracking-widest text-ink/40">Inspector</span>
                    <button @click="selectedNode = null" class="text-ink/40 hover:text-accent">
                        <i class="ri-close-line"></i>
                    </button>
                </div>
                
                <SemanticNodeCard 
                    :node="selectedNode" 
                    :interactive="true"
                    @click="navigateToNode"
                />

                <div class="text-xs text-ink/60 font-serif leading-relaxed">
                    <p>
                        This block represents a discrete Semantic Node in the knowledge graph. 
                        It is part of the "{{ article.title }}" article.
                    </p>
                </div>
            </div>

        </div>
    </div>
</template>

<style scoped>
.bg-grid-ash\/5 {
    background-image: radial-gradient(rgba(0,0,0,0.05) 1px, transparent 1px);
    background-size: 20px 20px;
}
</style>
