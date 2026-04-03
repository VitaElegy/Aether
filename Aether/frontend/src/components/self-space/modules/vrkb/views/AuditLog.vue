<template>
  <div class="audit-log">
    <div class="audit-header">
      <h4>Activity Log</h4>
      <button class="refresh-btn" @click="loadAuditLog">↻ Refresh</button>
    </div>

    <div class="audit-entries" v-if="loading">
      <p style="text-align: center; color: var(--text-secondary, #888); padding: 1rem;">Loading...</p>
    </div>

    <div class="audit-entries" v-else-if="entries.length > 0">
      <div v-for="entry in entries" :key="entry.id" class="audit-entry">
        <div class="entry-icon">{{ getActionIcon(entry.event_type) }}</div>
        <div class="entry-body">
          <span class="entry-action">{{ formatAction(entry.event_type) }}</span>
          <span class="entry-target">{{ entry.target_type }} {{ entry.target_id?.substring(0, 8) }}</span>
          <span class="entry-time">{{ formatTime(entry.created_at) }}</span>
        </div>
      </div>
    </div>

    <div class="empty-log" v-else>
      <p>No activity recorded yet.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { vrkbApi } from '@/api/vrkb';

const props = defineProps<{ projectId: string }>();

interface AuditEntry {
  id: string;
  event_type: string;
  action?: string;
  target_type: string;
  target_id?: string;
  actor_name?: string;
  created_at: string;
}

const entries = ref<AuditEntry[]>([]);
const loading = ref(false);

const loadAuditLog = async () => {
  loading.value = true;
  try {
    const result = await vrkbApi.listAuditLogs(props.projectId, { limit: 50 });
    // API returns { items: [...], total: N } — extract the items array
    entries.value = (result.items || []).map((e: any) => ({
      ...e,
      action: e.event_type, // normalize field name for template
    }));
  } catch (e) {
    console.error('Failed to load audit log', e);
  } finally {
    loading.value = false;
  }
};

const getActionIcon = (action: string) => {
  const icons: Record<string, string> = {
    finding_created: '🆕', finding_status_changed: '🔄', evidence_added: '📎',
    doc_updated: '📝', member_added: '👤', project_updated: '⚙️',
  };
  return icons[action] || '📋';
};

const formatAction = (action: string) => {
  return action.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
};

const formatTime = (timestamp: string) => {
  if (!timestamp) return '';
  const d = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return d.toLocaleDateString();
};

onMounted(loadAuditLog);
watch(() => props.projectId, loadAuditLog);
</script>

<style scoped>
.audit-log { padding: 1rem; }
.audit-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
.audit-header h4 { margin: 0; color: var(--text-primary, #fff); }
.refresh-btn { padding: 0.3rem 0.6rem; background: transparent; border: 1px solid rgba(255,255,255,0.1); color: var(--text-secondary, #888); border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
.audit-entry { display: flex; gap: 0.75rem; align-items: flex-start; padding: 0.5rem 0; border-bottom: 1px solid rgba(255,255,255,0.04); }
.entry-icon { font-size: 1.1rem; }
.entry-body { display: flex; flex-wrap: wrap; gap: 0.4rem; align-items: baseline; font-size: 0.8rem; }
.entry-action { color: var(--text-primary, #fff); font-weight: 500; }
.entry-target { color: var(--text-secondary, #888); font-family: monospace; font-size: 0.75rem; }
.entry-time { color: var(--text-secondary, #666); font-size: 0.7rem; }
.empty-log { text-align: center; padding: 2rem; color: var(--text-secondary, #888); }
</style>
