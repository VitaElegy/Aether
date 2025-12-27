<template>
  <h1
    class="font-serif text-gray-900 tracking-wide italic"
    :class="{
        'text-3xl': size === 'lg',
        'text-2xl': size === 'md',
        'text-xl': size === 'sm'
    }"
  >
    <slot></slot>
  </h1>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
    size?: 'sm' | 'md' | 'lg';
}>(), {
    size: 'lg'
});
</script>
