<script setup lang="ts">
import { computed, ref } from 'vue';
import { marked } from 'marked';

const props = defineProps<{
    block: {
        payload: {
            steps: string;
            qcd_symbol?: string;
        }
    }
}>();

const isExpanded = ref(false);

const renderedContent = computed(() => {
    return marked.parse(props.block.payload.steps || '');
});
</script>

<template>
    <div class="math-block proof border-l-4 border-green-500 bg-green-50/10 my-4 rounded-r-md overflow-hidden">
        <button 
            @click="isExpanded = !isExpanded"
            class="w-full text-left px-4 py-2 flex items-center justify-between hover:bg-green-50/50 transition-colors"
        >
            <div class="flex items-center gap-2 text-green-700 font-serif font-bold tracking-wide">
                <span class="uppercase text-xs tracking-widest bg-green-100 px-2 py-0.5 rounded">Proof</span>
            </div>
            <div class="text-green-400 transform transition-transform duration-200" :class="{ 'rotate-180': isExpanded }">
                <i class="i-lucide-chevron-down w-4 h-4"></i>
            </div>
        </button>
        
        <div v-show="isExpanded" class="px-4 pb-4 pt-2">
            <div class="prose prose-sm max-w-none font-serif text-gray-700" v-html="renderedContent"></div>
            <div class="text-right text-gray-400 mt-2 font-serif select-none">
                {{ block.payload.qcd_symbol || '■' }}
            </div>
        </div>
    </div>
</template>
