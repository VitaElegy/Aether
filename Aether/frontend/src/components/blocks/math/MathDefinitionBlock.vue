<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';

const props = defineProps<{
    block: {
        payload: {
            term: string;
            content: string;
        }
    }
}>();

const renderedContent = computed(() => {
    return marked.parse(props.block.payload.content || '');
});
</script>

<template>
    <div class="math-block definition border-l-4 border-blue-500 bg-blue-50/10 p-4 my-4 rounded-r-md">
        <div class="flex items-center gap-2 mb-2 text-blue-700 font-serif font-bold tracking-wide">
            <span class="uppercase text-xs tracking-widest bg-blue-100 px-2 py-0.5 rounded">Definition</span>
            <span>{{ block.payload.term }}</span>
        </div>
        <div class="prose prose-sm max-w-none font-serif text-gray-800" v-html="renderedContent"></div>
    </div>
</template>
