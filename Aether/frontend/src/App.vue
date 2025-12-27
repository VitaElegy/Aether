<script setup lang="ts">
import { RouterView } from 'vue-router';
import { usePreferencesStore } from './stores/preferences';

// Initialize store to apply theme immediately
usePreferencesStore();
</script>

<template>
    <div class="min-h-screen w-full flex flex-col">
        <RouterView />
    </div>
</template>
