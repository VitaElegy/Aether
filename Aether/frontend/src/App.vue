<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router';
import { usePreferencesStore } from './stores/preferences';
import { computed, onErrorCaptured, ref } from 'vue';

// Initialize store to apply theme immediately
usePreferencesStore();

const route = useRoute();

// Determines the transition effect based on route meta
const transitionName = computed(() => {
    return (route.meta.transition as string) || 'fade';
});

// Root Error Boundary
const fatalError = ref<Error | null>(null);

onErrorCaptured((err) => {
    console.error('[App] Fatal Error Captured:', err);
    fatalError.value = err instanceof Error ? err : new Error(String(err));
    return false; // Prevent bubble
});

const reloadApp = () => {
    window.location.reload();
};
</script>

<template>
    <div class="min-h-screen w-full flex flex-col relative bg-paper transition-colors duration-500">
        <div v-if="fatalError" class="fixed inset-0 z-[9999] bg-white flex flex-col items-center justify-center p-8 text-center text-red-600">
            <h1 class="text-4xl font-bold mb-4">Fatal Application Error</h1>
             <p class="text-xl text-gray-800 mb-6 font-mono bg-gray-100 p-4 rounded">{{ fatalError.message }}</p>
             <pre v-if="fatalError.stack" class="text-left text-xs text-gray-500 overflow-auto max-w-2xl max-h-64 mb-6 bg-gray-50 p-4 border border-gray-200 block whitespace-pre-wrap">{{ fatalError.stack }}</pre>
             <button @click="fatalError = null; $router.push('/')" class="px-6 py-3 bg-red-600 text-white font-bold rounded hover:bg-red-700">Attempt Recovery (Go Home)</button>
             <button @click="reloadApp" class="ml-4 px-6 py-3 bg-gray-200 text-gray-800 font-bold rounded hover:bg-gray-300">Reload Page</button>
        </div>

        <RouterView v-else v-slot="{ Component, route }">
            <Transition :name="transitionName" mode="out-in">
                <component :is="Component" :key="route.fullPath" />
            </Transition>
        </RouterView>
    </div>
</template>
