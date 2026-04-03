<template>
  <div class="checklist-panel">
    <div class="checklist-header">
      <h4>Checklist</h4>
      <div class="progress-bar" v-if="summary">
        <div class="progress-fill" :style="{ width: summary.completion_percent + '%' }"></div>
        <span class="progress-text">{{ summary.completed }}/{{ summary.total }} ({{ Math.round(summary.completion_percent) }}%)</span>
      </div>
    </div>

    <div class="blocker-alert" v-if="summary && summary.blockers > 0">
      ⚠️ {{ summary.blockers }} blocker(s) remaining
    </div>

    <div class="checklist-items">
      <div v-for="item in items" :key="item.id" 
        :class="['checklist-item', { completed: item.is_completed, blocker: item.is_blocker }]">
        <label class="checkbox-label">
          <input type="checkbox" :checked="item.is_completed" @change="toggleItem(item)" />
          <span class="item-title">{{ item.title }}</span>
          <span class="blocker-tag" v-if="item.is_blocker">BLOCKER</span>
        </label>
        <p class="item-desc" v-if="item.description">{{ item.description }}</p>
      </div>
    </div>

    <div class="add-item" v-if="showAddForm">
      <input v-model="newItemTitle" placeholder="New checklist item..." @keyup.enter="addItem" />
      <label><input type="checkbox" v-model="newItemBlocker" /> Blocker</label>
      <button @click="addItem" :disabled="!newItemTitle.trim()">Add</button>
    </div>
    <button v-else class="add-btn" @click="showAddForm = true">+ Add Item</button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { vrkbApi } from '@/api/vrkb';

const props = defineProps<{ sectionId: string }>();

const items = ref<any[]>([]);
const summary = ref<any>(null);
const showAddForm = ref(false);
const newItemTitle = ref('');
const newItemBlocker = ref(false);

const loadChecklist = async () => {
  try {
    items.value = await vrkbApi.getChecklist(props.sectionId);
    summary.value = await vrkbApi.getChecklistSummary(props.sectionId);
  } catch (e) {
    console.error('Failed to load checklist', e);
  }
};

const toggleItem = async (item: any) => {
  try {
    await vrkbApi.updateChecklistItem(props.sectionId, item.id, { is_completed: !item.is_completed });
    await loadChecklist();
  } catch (e) {
    console.error('Failed to toggle item', e);
  }
};

const addItem = async () => {
  if (!newItemTitle.value.trim()) return;
  try {
    await vrkbApi.createChecklistItem(props.sectionId, {
      title: newItemTitle.value,
      is_blocker: newItemBlocker.value,
    });
    newItemTitle.value = '';
    newItemBlocker.value = false;
    showAddForm.value = false;
    await loadChecklist();
  } catch (e) {
    console.error('Failed to add item', e);
  }
};

onMounted(loadChecklist);
watch(() => props.sectionId, loadChecklist);
</script>

<style scoped>
.checklist-panel { padding: 1rem; }
.checklist-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
.checklist-header h4 { margin: 0; color: var(--text-primary, #fff); }
.progress-bar { position: relative; width: 200px; height: 20px; background: rgba(255,255,255,0.05); border-radius: 10px; overflow: hidden; }
.progress-fill { height: 100%; background: linear-gradient(90deg, #4caf50, #8bc34a); border-radius: 10px; transition: width 0.3s ease; }
.progress-text { position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); font-size: 0.7rem; color: #fff; text-shadow: 0 0 4px rgba(0,0,0,0.5); }
.blocker-alert { padding: 0.5rem; background: rgba(255, 152, 0, 0.1); border: 1px solid rgba(255, 152, 0, 0.3); border-radius: 6px; color: #ff9800; font-size: 0.8rem; margin-bottom: 0.75rem; }
.checklist-item { padding: 0.5rem; border-radius: 4px; margin-bottom: 0.25rem; }
.checklist-item.completed { opacity: 0.6; }
.checklist-item.blocker { border-left: 3px solid #ff9800; }
.checkbox-label { display: flex; align-items: center; gap: 0.5rem; cursor: pointer; color: var(--text-primary, #fff); }
.item-title { font-size: 0.85rem; }
.blocker-tag { font-size: 0.6rem; padding: 0.1rem 0.3rem; background: rgba(255, 152, 0, 0.2); color: #ff9800; border-radius: 3px; font-weight: 600; }
.item-desc { margin: 0.25rem 0 0 1.5rem; font-size: 0.75rem; color: var(--text-secondary, #888); }
.add-item { display: flex; gap: 0.5rem; align-items: center; margin-top: 0.5rem; }
.add-item input[type="text"], .add-item input:not([type]) { flex: 1; padding: 0.4rem 0.6rem; background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: var(--text-primary, #fff); }
.add-item button { padding: 0.4rem 0.8rem; background: rgba(100, 181, 246, 0.2); color: #64b5f6; border: 1px solid rgba(100, 181, 246, 0.3); border-radius: 4px; cursor: pointer; }
.add-btn { margin-top: 0.5rem; padding: 0.4rem 0.8rem; background: transparent; border: 1px dashed rgba(255,255,255,0.1); color: var(--text-secondary, #888); border-radius: 4px; cursor: pointer; }
</style>
