<template>
  <nav class="h-16 w-full flex-shrink-0 flex items-center justify-between px-6 bg-paper/95 backdrop-blur z-50 sticky top-0 transition-all duration-200">
    <!-- Left: Page Identity / Title -->
    <div class="flex items-center gap-4 flex-1 justify-start min-w-0">
      <GlobalBeacon />
      <slot name="left"></slot>
    </div>

    <!-- Center: Navigation / Context Controls -->
    <div class="flex items-center justify-center gap-6 flex-[2] min-w-0 px-4">
      <slot name="center"></slot>
    </div>

    <!-- Right: Functional Actions -->
    <div class="flex items-center gap-4 flex-1 justify-end min-w-0">
      <slot name="right"></slot>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import GlobalBeacon from '@/components/navigation/GlobalBeacon.vue';

// TopNavBar Component
// Designed with 3 zones: Identity (Left), Context/Nav (Center), Actions (Right)

const router = useRouter();
</script>

<style scoped>
/* No visible border as requested, relying on spacing and background transparency */
</style>
