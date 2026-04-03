<template>
  <div class="import-analysis-modal" v-if="visible">
    <div class="modal-backdrop" @click="$emit('close')"></div>
    <div class="modal-content">
      <h3>Import Analysis</h3>
      <div class="loading" v-if="loading">Analyzing import file...</div>
      <div class="summary" v-else-if="preview">
        <div class="stat-row">
          <span>Total Items</span>
          <strong>{{ preview.summary.total_items }}</strong>
        </div>
        <div class="sections">
          <div v-for="s in preview.summary.sections" :key="s.name" class="section-row">
            <span>{{ s.name }}</span>
            <span class="action-badge" :class="s.action.toLowerCase()">{{ s.action }}</span>
            <span>{{ s.count }} items</span>
          </div>
        </div>
        <div class="conflicts" v-if="preview.conflicts.length > 0">
          <h4>⚠️ Conflicts ({{ preview.conflicts.length }})</h4>
          <ul>
            <li v-for="c in preview.conflicts" :key="c.item_id">{{ c.item_name }}: {{ c.conflict_type }}</li>
          </ul>
        </div>
        <div class="modal-actions">
          <button class="import-btn" @click="$emit('startImport')">Proceed with Import</button>
          <button class="cancel-btn" @click="$emit('close')">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ImportPreview } from '@/api/portability';

defineProps<{
  visible: boolean;
  loading: boolean;
  preview: ImportPreview | null;
}>();
defineEmits(['close', 'startImport']);
</script>

<style scoped>
.import-analysis-modal { position: fixed; inset: 0; z-index: 1000; display: flex; align-items: center; justify-content: center; }
.modal-backdrop { position: absolute; inset: 0; background: rgba(0,0,0,0.6); }
.modal-content { position: relative; background: var(--bg-secondary, #1a1a2e); border-radius: 12px; padding: 1.5rem; width: 480px; max-height: 80vh; overflow-y: auto; border: 1px solid rgba(255,255,255,0.08); }
.modal-content h3 { margin: 0 0 1rem 0; color: var(--text-primary, #fff); }
.stat-row { display: flex; justify-content: space-between; padding: 0.5rem 0; border-bottom: 1px solid rgba(255,255,255,0.04); color: var(--text-primary, #ddd); }
.section-row { display: flex; gap: 0.5rem; align-items: center; padding: 0.5rem; background: rgba(255,255,255,0.02); border-radius: 6px; margin-top: 0.5rem; }
.action-badge { padding: 0.15rem 0.4rem; border-radius: 3px; font-size: 0.7rem; font-weight: 600; text-transform: uppercase; }
.action-badge.create { background: rgba(76, 175, 80, 0.2); color: #4caf50; }
.action-badge.update { background: rgba(255, 193, 7, 0.2); color: #ffc107; }
.action-badge.skip { background: rgba(158, 158, 158, 0.2); color: #9e9e9e; }
.conflicts { margin-top: 1rem; }
.conflicts h4 { color: #ff9800; margin: 0 0 0.5rem 0; }
.conflicts ul { margin: 0; padding-left: 1.2rem; color: var(--text-secondary, #888); font-size: 0.8rem; }
.modal-actions { display: flex; gap: 0.5rem; margin-top: 1rem; }
.import-btn { flex: 1; padding: 0.6rem; background: rgba(33, 150, 243, 0.2); color: #2196f3; border: 1px solid rgba(33, 150, 243, 0.3); border-radius: 6px; cursor: pointer; }
.cancel-btn { flex: 1; padding: 0.6rem; background: transparent; color: var(--text-secondary, #888); border: 1px solid rgba(255,255,255,0.1); border-radius: 6px; cursor: pointer; }
.loading { text-align: center; padding: 2rem; color: var(--text-secondary, #888); }
</style>
