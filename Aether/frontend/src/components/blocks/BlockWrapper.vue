<script setup lang="ts">
import { defineAsyncComponent, onErrorCaptured, ref, shallowRef, watch, computed } from 'vue';
import QuarantineBox from './QuarantineBox.vue';

const props = defineProps<{
    block: {
        id: string;
        type: string;
        payload: any;
        revision: number;
    }
}>();

const error = shallowRef<unknown | null>(null);

// Error Boundary Logic
onErrorCaptured((err, instance, info) => {
    console.error(`[BlockWrapper] Error captured in block ${props.block.type}:`, err);
    error.value = err;
    return false; // Stop propagation to prevent app crash
});

// Reset error when block revisions changes (User might have fixed it)
watch(() => props.block.revision, () => {
    error.value = null;
});

// Dynamic Renderer Loader (Registry Pattern)
// Dynamic Renderer Loader (Registry Pattern)
const registry: Record<string, any> = {
    axiom: defineAsyncComponent(() => import('./math/MathAxiomBlock.vue')),
    theorem: defineAsyncComponent(() => import('./math/MathTheoremBlock.vue')),
    definition: defineAsyncComponent(() => import('./math/MathDefinitionBlock.vue')),
    proof: defineAsyncComponent(() => import('./math/MathProofBlock.vue')),
};

const resolvedRenderer = computed(() => {
    const loader = registry[props.block.type];
    if (loader) return loader;
    
    // Fallback or Error
    // In dev, we might want to throw to trigger QuarantineBox
    // throw new Error(`Renderer for ${props.block.type} not found`);
    return null; 
});

// Since defineAsyncComponent handles loading states internally, 
// we can use it directly. However, to trigger QuarantineBox on load error,
// we rely on onErrorCaptured which catches async load errors too.


</script>

<template>
    <div class="block-wrapper relative group" :data-block-id="block.id">
        <QuarantineBox 
            v-if="error" 
            :error="error" 
            :blockType="block.type" 
            :payload="block.payload"
        />
        
        <Suspense v-else>
            <template #default>
                <component 
                    v-if="resolvedRenderer"
                    :is="resolvedRenderer" 
                    :block="block"
                />
                <div v-else class="p-4 border border-dashed text-gray-400">
                    [Block Placeholder: {{ block.type }}]
                </div>
            </template>
            <template #fallback>
                <div class="h-10 bg-gray-50 animate-pulse rounded my-2"></div>
            </template>
        </Suspense>
    </div>
</template>
