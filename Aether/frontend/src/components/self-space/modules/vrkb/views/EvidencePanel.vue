<template>
  <div class="evidence-panel">
    <div class="evidence-header">
      <h4>Evidence ({{ evidenceList.length }})</h4>
      <button class="add-evidence-btn" @click="showAddForm = !showAddForm">+ Add Evidence</button>
    </div>

    <div class="add-evidence-form" v-if="showAddForm">
      <select v-model="newEvidence.evidence_type">
        <option value="screenshot">📷 Screenshot</option>
        <option value="request_response">🔄 Request/Response</option>
        <option value="log_extract">📋 Log Extract</option>
        <option value="poc_file">🔧 PoC File</option>
        <option value="external_reference">🔗 External Reference</option>
      </select>
      <input v-model="newEvidence.title" placeholder="Evidence title..." />
      <textarea v-model="newEvidence.description" placeholder="Description (optional)..." rows="2"></textarea>
      <textarea v-model="contentText" placeholder="Content (URL, raw text, etc.)..." rows="3"></textarea>
      <div class="form-actions">
        <button @click="addEvidence" :disabled="!newEvidence.title.trim()">Save</button>
        <button @click="showAddForm = false" class="cancel">Cancel</button>
      </div>
    </div>

    <div class="evidence-list">
      <div v-for="ev in evidenceList" :key="ev.id" class="evidence-card">
        <div class="evidence-type-icon">
          {{ getTypeIcon(ev.evidence_type) }}
        </div>
        <div class="evidence-body">
          <h5>{{ ev.title }}</h5>
          <p v-if="ev.description" class="ev-desc">{{ ev.description }}</p>
          <span class="ev-type">{{ ev.evidence_type }}</span>
          <span class="ev-date">{{ new Date(ev.created_at).toLocaleDateString() }}</span>
        </div>
        <button class="delete-btn" @click="deleteEvidence(ev.id)" title="Delete">🗑</button>
      </div>
    </div>

    <div class="empty-evidence" v-if="evidenceList.length === 0 && !showAddForm">
      <p>No evidence attached. Click "Add Evidence" to get started.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { vrkbApi } from '@/api/vrkb';

const props = defineProps<{
  projectId: string;
  attachedToType: string;  // "finding", "doc", "asset"
  attachedToId: string;
}>();

const evidenceList = ref<any[]>([]);
const showAddForm = ref(false);
const contentText = ref('');
const newEvidence = ref({
  evidence_type: 'screenshot',
  title: '',
  description: '',
});

const getTypeIcon = (type: string) => {
  const icons: Record<string, string> = {
    screenshot: '📷', request_response: '🔄', log_extract: '📋', poc_file: '🔧', external_reference: '🔗'
  };
  return icons[type] || '📎';
};

const loadEvidence = async () => {
  try {
    evidenceList.value = await vrkbApi.getEvidence(props.projectId, props.attachedToType, props.attachedToId);
  } catch (e) {
    console.error('Failed to load evidence', e);
  }
};

const addEvidence = async () => {
  try {
    await vrkbApi.createEvidence(props.projectId, {
      ...newEvidence.value,
      content: { text: contentText.value },
      attached_to_type: props.attachedToType,
      attached_to_id: props.attachedToId,
    });
    newEvidence.value = { evidence_type: 'screenshot', title: '', description: '' };
    contentText.value = '';
    showAddForm.value = false;
    await loadEvidence();
  } catch (e) {
    console.error('Failed to add evidence', e);
  }
};

const deleteEvidence = async (id: string) => {
  try {
    await vrkbApi.deleteEvidence(props.projectId, id);
    await loadEvidence();
  } catch (e) {
    console.error('Failed to delete evidence', e);
  }
};

onMounted(loadEvidence);
watch(() => [props.projectId, props.attachedToId], loadEvidence);
</script>

<style scoped>
.evidence-panel { padding: 1rem; }
.evidence-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
.evidence-header h4 { margin: 0; color: var(--text-primary, #fff); }
.add-evidence-btn { padding: 0.3rem 0.6rem; background: rgba(100, 181, 246, 0.15); border: 1px solid rgba(100, 181, 246, 0.3); color: #64b5f6; border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
.add-evidence-form { padding: 1rem; background: rgba(255,255,255,0.03); border-radius: 8px; border: 1px solid rgba(255,255,255,0.06); margin-bottom: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
.add-evidence-form select, .add-evidence-form input, .add-evidence-form textarea { padding: 0.4rem 0.6rem; background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: var(--text-primary, #fff); font-size: 0.85rem; }
.form-actions { display: flex; gap: 0.5rem; }
.form-actions button { padding: 0.4rem 0.8rem; border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
.form-actions button:first-child { background: rgba(76, 175, 80, 0.2); color: #4caf50; border: 1px solid rgba(76, 175, 80, 0.3); }
.form-actions .cancel { background: transparent; color: var(--text-secondary, #888); border: 1px solid rgba(255,255,255,0.1); }
.evidence-card { display: flex; align-items: flex-start; gap: 0.75rem; padding: 0.75rem; border-radius: 8px; background: rgba(255,255,255,0.02); border: 1px solid rgba(255,255,255,0.04); margin-bottom: 0.5rem; }
.evidence-type-icon { font-size: 1.5rem; line-height: 1; }
.evidence-body { flex: 1; }
.evidence-body h5 { margin: 0 0 0.25rem 0; color: var(--text-primary, #fff); font-size: 0.85rem; }
.ev-desc { margin: 0 0 0.25rem 0; font-size: 0.75rem; color: var(--text-secondary, #888); }
.ev-type, .ev-date { font-size: 0.7rem; color: var(--text-secondary, #666); margin-right: 0.5rem; }
.delete-btn { background: transparent; border: none; cursor: pointer; opacity: 0.4; font-size: 1rem; }
.delete-btn:hover { opacity: 1; }
.empty-evidence { text-align: center; padding: 2rem; color: var(--text-secondary, #888); font-size: 0.85rem; }
</style>
