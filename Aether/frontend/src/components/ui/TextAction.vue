<template>
  <button
    class="font-serif italic transition-colors duration-200 focus:outline-none"
    :class="[
        sizeClasses[size],
        active
          ? 'text-gray-900 underline decoration-1 underline-offset-4'
          : 'text-gray-500 hover:text-gray-800'
    ]"
    @click="$emit('click', $event)"
  >
    <slot></slot>
  </button>
</template>

<script setup lang="ts">
const props = withDefaults(defineProps<{
    active?: boolean;
    size?: 'sm' | 'md' | 'lg';
}>(), {
    active: false,
    size: 'md'
});

defineEmits(['click']);

const sizeClasses = {
    sm: 'text-sm',
    md: 'text-lg',
    lg: 'text-xl'
};
</script>
