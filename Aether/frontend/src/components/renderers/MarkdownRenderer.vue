<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';

interface Props {
    content: string;
}

const props = defineProps<Props>();

const renderedHtml = computed(() => {
    if (!props.content) return '';

    const renderer = new marked.Renderer();
    renderer.heading = function ({ text, depth }: { text: string; depth: number }) {
        const id = text.toLowerCase().replace(/[^\w]+/g, '-');
        return `<h${depth} id="${id}">${text}</h${depth}>`;
    };

    return marked(props.content, { renderer });
});
</script>

<template>
    <div class="prose prose-lg max-w-none" v-html="renderedHtml"></div>
</template>
