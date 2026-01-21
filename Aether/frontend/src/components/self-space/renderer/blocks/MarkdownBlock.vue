<template>
    <div class="prose dark:prose-invert max-w-none mb-4">
        <!-- In a real app, use a markdown parser like 'marked' here -->
        <p class="text-ink text-lg leading-relaxed">{{ payload.content }}</p>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
    block: any;
}>();

const payload = computed(() => props.block.payload || {});
</script>
