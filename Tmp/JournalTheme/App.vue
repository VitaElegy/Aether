<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';

onMounted(() => {
  // Use light mode but with warmer, richer tones
  document.documentElement.setAttribute('theme-mode', 'light');
});
</script>

<template>
  <RouterView />
</template>

<style>
/*
  AETHER JOURNAL THEME
  High-end "Bullet Journal" / "Traveler's Notebook" aesthetic.
  Warm, tactile, personal.
*/
:root {
  /* Brand Colors - Leather, Ink, and Cream */
  --td-brand-color: #8B4513; /* Saddle Brown */
  --td-brand-color-hover: #A0522D;
  --td-brand-color-focus: #5D4037;
  --td-brand-color-active: #3E2723;

  /* Backgrounds - textured paper colors */
  --td-bg-color-page: #F5F1E6; /* Warm Cream */
  --td-bg-color-container: #FFFAF0; /* Floral White */
  --td-bg-color-secondarycontainer: #EAE5D9;

  /* Typography */
  --td-font-family: 'Crimson Pro', 'Georgia', serif; /* Elegant Serif */
  --td-text-color-primary: #2C241B; /* Dark Coffee */
  --td-text-color-secondary: #5D5448;

  /* Borders - Hand-drawn feel */
  --td-component-border: #D7CCC8;
  --td-radius-medium: 4px;

  /* Custom Journal Variables */
  --journal-shadow: 2px 4px 12px rgba(44, 36, 27, 0.15);
  --paper-texture: url('https://www.transparenttextures.com/patterns/cream-paper.png');
  --leather-texture: #5D4037;
}

body {
  margin: 0;
  padding: 0;
  background-color: #2C241B; /* Desk background */
  color: var(--td-text-color-primary);
  line-height: 1.6;
  font-family: var(--td-font-family);
  background-image: url('https://www.transparenttextures.com/patterns/wood-pattern.png'); /* Desk wood texture */
}

/* Custom Scrollbar - Hidden or minimal */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(139, 69, 19, 0.3);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(139, 69, 19, 0.6);
}

* {
  box-sizing: border-box;
}

/* TDesign Overrides */
.t-button {
  font-family: 'Inter', sans-serif; /* Buttons stay readable sans-serif */
  letter-spacing: 0.05em;
  text-transform: uppercase;
  font-size: 12px !important;
  font-weight: 600 !important;
}

.t-card {
  border: none !important;
  box-shadow: var(--journal-shadow) !important;
  background-color: var(--td-bg-color-container) !important;
  background-image: var(--paper-texture);
}

.t-tag {
  font-family: 'Caveat', cursive; /* Handwritten tags */
  font-size: 14px !important;
  background: transparent !important;
  border: 1px solid var(--td-brand-color) !important;
  color: var(--td-brand-color) !important;
}

/* Import Fonts */
@import url('https://fonts.googleapis.com/css2?family=Crimson+Pro:ital,wght@0,300;0,400;0,600;1,400&family=Caveat:wght@400;700&family=Inter:wght@400;500;600&display=swap');
</style>
