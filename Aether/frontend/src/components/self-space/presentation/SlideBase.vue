<template>
  <div class="slide-base">
    <div class="slide-content">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  pageNumber?: number | string
  totalPages?: number | string
}>()
</script>

<style scoped>
.slide-base {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  background-color: var(--td-bg-color-container); /* Using TDesign token or fallback */
  color: var(--td-text-color-primary);
  overflow: hidden;
  position: relative;
  /* Glassmorphism background effect */
  background: radial-gradient(circle at 50% 50%, #2c3e50 0%, #000000 100%);
}

.slide-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: 4rem;
  box-sizing: border-box;
  z-index: 1;
}

.slide-footer {
  position: absolute;
  bottom: 2rem;
  right: 2rem;
  z-index: 2;
  font-family: 'Outfit', sans-serif;
  opacity: 0.5;
}
</style>
