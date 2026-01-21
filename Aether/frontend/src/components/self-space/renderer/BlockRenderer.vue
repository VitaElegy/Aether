<template>
    <div class="w-full max-w-4xl mx-auto py-8">
        <template v-for="block in blocks" :key="block.id">
            <component 
                :is="resolveComponent(block.type)" 
                :block="block"
            />
        </template>
    </div>
</template>

<script setup lang="ts">
import MarkdownBlock from './blocks/MarkdownBlock.vue';
import MathBlock from './blocks/MathBlock.vue';
import { defineAsyncComponent } from 'vue';

const props = defineProps<{
    blocks: any[];
}>();

// Map string types (from backend) to Vue Components
const resolveComponent = (type: string) => {
    switch (type) {
        case 'markdown': return MarkdownBlock;
        case 'math_block': return MathBlock;
        default: return MarkdownBlock; // Fallback
    }
};
</script>
