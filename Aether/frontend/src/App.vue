<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router';
import { usePreferencesStore } from './stores/preferences';
import { computed } from 'vue';

// Initialize store to apply theme immediately
usePreferencesStore();

const route = useRoute();

// Determines the transition effect based on route meta
const transitionName = computed(() => {
    return (route.meta.transition as string) || 'fade';
});
</script>

<template>
    <div class="min-h-screen w-full flex flex-col relative bg-paper transition-colors duration-500">
        <RouterView v-slot="{ Component, route }">
            <Transition :name="transitionName" mode="out-in">
                <component :is="Component" :key="route.fullPath" />
            </Transition>
        </RouterView>
    </div>
</template>
