<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';

const props = defineProps<{
    block: {
        payload: {
            label?: string;
            content: string;
        }
    }
}>();

const renderedContent = computed(() => {
    return marked.parse(props.block.payload.content || '');
});
</script>

<template>
    <div class="math-block theorem border-l-4 border-purple-500 bg-purple-50/10 p-4 my-4 rounded-r-md shadow-sm">
        <div class="flex items-center gap-2 mb-2 text-purple-700 font-serif font-bold tracking-wide">
            <span class="uppercase text-xs tracking-widest bg-purple-100 px-2 py-0.5 rounded">Theorem</span>
            <span v-if="block.payload.label">{{ block.payload.label }}</span>
        </div>
        <div class="prose prose-sm max-w-none font-serif text-gray-800 italic" v-html="renderedContent"></div>
    </div>
</template>
