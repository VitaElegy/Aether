<template>
  <div class="p-6 mx-auto" :class="maxWidthClass">
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
    maxWidth?: '3xl' | '4xl' | '5xl' | '6xl' | 'full';
}>(), {
    maxWidth: '5xl'
});

const maxWidthClass = computed(() => `max-w-${props.maxWidth}`);
</script>
