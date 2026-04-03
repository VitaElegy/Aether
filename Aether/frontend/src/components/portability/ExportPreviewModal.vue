<template>
  <div class="export-preview-modal" v-if="visible">
    <div class="modal-backdrop" @click="$emit('close')"></div>
    <div class="modal-content">
      <h3>Export Preview</h3>
      <div class="loading" v-if="loading">Analyzing export...</div>
      <div class="summary" v-else-if="summary">
        <div class="stat-row">
          <span>Total Items</span>
          <strong>{{ summary.total_items }}</strong>
        </div>
        <div class="stat-row">
          <span>Estimated Size</span>
          <strong>{{ summary.estimated_size }}</strong>
        </div>
        <div class="sections">
          <div v-for="s in summary.sections" :key="s.name" class="section-row">
            <span>{{ s.name }}</span>
            <span>{{ s.count }} items</span>
            <small>{{ s.details }}</small>
          </div>
        </div>
        <div class="modal-actions">
          <button class="export-btn" @click="$emit('startExport')">Start Export</button>
          <button class="cancel-btn" @click="$emit('close')">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ExportSummary } from '@/api/portability';

defineProps<{
  visible: boolean;
  loading: boolean;
  summary: ExportSummary | null;
}>();
defineEmits(['close', 'startExport']);
</script>

<style scoped>
.export-preview-modal { position: fixed; inset: 0; z-index: 1000; display: flex; align-items: center; justify-content: center; }
.modal-backdrop { position: absolute; inset: 0; background: rgba(0,0,0,0.6); }
.modal-content { position: relative; background: var(--bg-secondary, #1a1a2e); border-radius: 12px; padding: 1.5rem; width: 480px; max-height: 80vh; overflow-y: auto; border: 1px solid rgba(255,255,255,0.08); }
.modal-content h3 { margin: 0 0 1rem 0; color: var(--text-primary, #fff); }
.stat-row { display: flex; justify-content: space-between; padding: 0.5rem 0; border-bottom: 1px solid rgba(255,255,255,0.04); color: var(--text-primary, #ddd); }
.section-row { padding: 0.5rem; background: rgba(255,255,255,0.02); border-radius: 6px; margin-top: 0.5rem; }
.section-row small { display: block; color: var(--text-secondary, #888); font-size: 0.75rem; }
.modal-actions { display: flex; gap: 0.5rem; margin-top: 1rem; }
.export-btn { flex: 1; padding: 0.6rem; background: rgba(76, 175, 80, 0.2); color: #4caf50; border: 1px solid rgba(76, 175, 80, 0.3); border-radius: 6px; cursor: pointer; }
.cancel-btn { flex: 1; padding: 0.6rem; background: transparent; color: var(--text-secondary, #888); border: 1px solid rgba(255,255,255,0.1); border-radius: 6px; cursor: pointer; }
.loading { text-align: center; padding: 2rem; color: var(--text-secondary, #888); }
</style>
