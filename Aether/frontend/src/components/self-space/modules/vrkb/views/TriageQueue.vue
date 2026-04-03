<template>
  <div class="triage-queue">
    <div class="triage-header">
      <h3>Triage Queue</h3>
      <div class="triage-stats" v-if="stats">
        <span class="stat-badge unreviewed">{{ stats.unreviewed }} unreviewed</span>
        <span class="stat-badge stale">{{ stats.stale }} stale</span>
        <span class="stat-badge missing">{{ stats.missing_evidence }} missing evidence</span>
      </div>
    </div>

    <div class="filter-bar">
      <button v-for="f in filters" :key="f.value" 
        :class="['filter-btn', { active: currentFilter === f.value }]"
        @click="currentFilter = f.value; loadQueue()">
        {{ f.label }}
      </button>
    </div>

    <div class="findings-list" v-if="findings.length > 0">
      <div v-for="finding in findings" :key="finding.id" class="finding-card triage">
        <div class="finding-info">
          <span :class="['severity-badge', finding.severity?.toLowerCase()]">{{ finding.severity }}</span>
          <h4>{{ finding.title }}</h4>
          <span class="status-tag">{{ finding.status }}</span>
        </div>
        <div class="triage-actions">
          <button class="action-btn accept" @click="$emit('accept', finding)" title="Accept">✓</button>
          <button class="action-btn reject" @click="$emit('reject', finding)" title="Reject">✕</button>
          <button class="action-btn request" @click="$emit('requestEvidence', finding)" title="Request Evidence">📎</button>
        </div>
      </div>
    </div>

    <div class="empty-state" v-else>
      <p>No findings matching this filter.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { vrkbApi } from '@/api/vrkb';

const props = defineProps<{ projectId: string }>();
const emit = defineEmits(['accept', 'reject', 'requestEvidence']);

const findings = ref<any[]>([]);
const stats = ref<any>(null);
const currentFilter = ref('unreviewed');
const filters = [
  { label: 'Unreviewed', value: 'unreviewed' },
  { label: 'Stale', value: 'stale' },
  { label: 'Missing Evidence', value: 'missing_evidence' },
  { label: 'All', value: 'all' },
];

const loadQueue = async () => {
  try {
    findings.value = await vrkbApi.getTriageQueue(props.projectId, currentFilter.value);
    stats.value = await vrkbApi.getTriageStats(props.projectId);
  } catch (e) {
    console.error('Failed to load triage queue', e);
  }
};

onMounted(loadQueue);
watch(() => props.projectId, loadQueue);
</script>

<style scoped>
.triage-queue { padding: 1rem; }
.triage-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.triage-header h3 { margin: 0; color: var(--text-primary, #fff); }
.triage-stats { display: flex; gap: 0.5rem; }
.stat-badge { padding: 0.25rem 0.5rem; border-radius: 12px; font-size: 0.75rem; }
.stat-badge.unreviewed { background: rgba(255, 193, 7, 0.2); color: #ffc107; }
.stat-badge.stale { background: rgba(255, 87, 34, 0.2); color: #ff5722; }
.stat-badge.missing { background: rgba(156, 39, 176, 0.2); color: #9c27b0; }
.filter-bar { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
.filter-btn { padding: 0.4rem 0.8rem; border: 1px solid rgba(255,255,255,0.1); background: transparent; color: var(--text-secondary, #aaa); border-radius: 4px; cursor: pointer; }
.filter-btn.active { background: rgba(100, 181, 246, 0.2); color: #64b5f6; border-color: #64b5f6; }
.finding-card { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem; margin-bottom: 0.5rem; border-radius: 8px; background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); }
.finding-info { display: flex; align-items: center; gap: 0.75rem; }
.finding-info h4 { margin: 0; color: var(--text-primary, #fff); font-size: 0.9rem; }
.severity-badge { padding: 0.15rem 0.5rem; border-radius: 4px; font-size: 0.7rem; font-weight: 600; text-transform: uppercase; }
.severity-badge.critical { background: rgba(244, 67, 54, 0.2); color: #f44336; }
.severity-badge.high { background: rgba(255, 152, 0, 0.2); color: #ff9800; }
.severity-badge.medium { background: rgba(255, 193, 7, 0.2); color: #ffc107; }
.severity-badge.low { background: rgba(76, 175, 80, 0.2); color: #4caf50; }
.status-tag { font-size: 0.7rem; color: var(--text-secondary, #aaa); background: rgba(255,255,255,0.05); padding: 0.1rem 0.4rem; border-radius: 4px; }
.triage-actions { display: flex; gap: 0.3rem; }
.action-btn { width: 32px; height: 32px; border-radius: 50%; border: 1px solid rgba(255,255,255,0.1); background: transparent; cursor: pointer; display: flex; align-items: center; justify-content: center; }
.action-btn.accept:hover { background: rgba(76, 175, 80, 0.2); }
.action-btn.reject:hover { background: rgba(244, 67, 54, 0.2); }
.action-btn.request:hover { background: rgba(33, 150, 243, 0.2); }
.empty-state { text-align: center; padding: 2rem; color: var(--text-secondary, #888); }
</style>
