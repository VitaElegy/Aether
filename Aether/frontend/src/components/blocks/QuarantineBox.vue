<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
    error: unknown;
    blockType: string;
    payload: any;
}>();

const showDetails = ref(false);
</script>

<template>
    <div class="quarantine-box p-4 my-2 border border-red-200 bg-red-50 rounded-lg text-sm">
        <div class="flex items-center justify-between text-red-800 mb-2">
            <div class="font-medium flex items-center gap-2">
                <i class="i-lucide-alert-triangle w-4 h-4"></i>
                <span>Failed to render {{ blockType }}</span>
            </div>
            <button 
                @click="showDetails = !showDetails"
                class="text-xs underline hover:text-red-900"
            >
                {{ showDetails ? 'Hide Details' : 'View Source' }}
            </button>
        </div>
        
        <p v-if="!showDetails" class="text-red-600 opacity-80">
            This block has been isolated to prevent page crash.
        </p>

        <div v-else class="mt-2 space-y-2">
            <div class="p-2 bg-white rounded border border-red-100 font-mono text-xs text-red-600 break-words">
                {{ error }}
            </div>
            <pre class="p-2 bg-gray-50 rounded border border-gray-200 font-mono text-xs overflow-x-auto text-gray-600">{{ JSON.stringify(payload, null, 2) }}</pre>
        </div>
    </div>
</template>

<style scoped>
.quarantine-box {
    contain: content; /* CSS Containment for extra safety */
}
</style>
