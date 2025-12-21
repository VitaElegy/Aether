<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';

interface Props {
  content: string;
}

const props = defineProps<Props>();

const renderedHtml = computed(() => {
  if (!props.content) return '';
  return marked(props.content);
});
</script>

<template>
  <div class="prose prose-neutral prose-lg max-w-none" v-html="renderedHtml"></div>
</template>
