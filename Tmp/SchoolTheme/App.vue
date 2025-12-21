<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';

onMounted(() => {
  // Switch to Light Mode for the "School Homework" aesthetic
  document.documentElement.setAttribute('theme-mode', 'light');
});
</script>

<template>
  <RouterView />
</template>

<style>
/*
  SCHOOL HOMEWORK THEME
  Clean, academic, paper-like aesthetic.
*/
:root {
  /* Brand Colors - Academic Blue & Teacher's Red */
  --td-brand-color: #2563eb;
  --td-brand-color-hover: #1d4ed8;
  --td-brand-color-focus: #1e40af;
  --td-brand-color-active: #1e3a8a;

  /* Backgrounds - Paper White & Off-White */
  --td-bg-color-page: #F8F9FA;
  --td-bg-color-container: #FFFFFF;
  --td-bg-color-secondarycontainer: #F1F3F5;

  /* Typography - Serious but legible */
  --td-font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --td-text-color-primary: #1F2937;
  --td-text-color-secondary: #4B5563;

  /* Borders - Pencil Thin */
  --td-component-border: #E5E7EB;
  --td-radius-medium: 8px;

  /* Custom Homework Variables */
  --paper-shadow: 0 1px 3px rgba(0,0,0,0.1), 0 1px 2px rgba(0,0,0,0.06);
  --paper-line-color: #E5E7EB;
  --accent-highlight: #FEF3C7; /* Post-it Yellow */
}

body {
  margin: 0;
  padding: 0;
  background-color: var(--td-bg-color-page);
  color: var(--td-text-color-primary);
  line-height: 1.6;
  /* Ruled paper effect hint */
  background-image: linear-gradient(#E5E7EB 1px, transparent 1px);
  background-size: 100% 40px;
}

/* Custom Scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: #F1F3F5;
}

::-webkit-scrollbar-thumb {
  background: #CBD5E1;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #94A3B8;
}

* {
  box-sizing: border-box;
}

/* Component Overrides for "Worksheet" feel */
.t-button {
  border-radius: 8px !important;
  font-weight: 600 !important;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}

.t-card {
  border: 1px solid var(--td-component-border) !important;
  box-shadow: var(--paper-shadow) !important;
  border-radius: 12px !important;
}

.t-tag {
  border-radius: 12px !important;
  font-weight: 500 !important;
}
</style>
