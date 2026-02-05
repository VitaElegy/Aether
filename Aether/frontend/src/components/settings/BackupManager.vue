<template>
  <div class="space-y-6">
    <!-- Portability Section (New) -->
    <div class="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-lg p-6 shadow-sm border border-blue-100 dark:border-blue-800">
      <h2 class="text-lg font-medium mb-2 text-blue-900 dark:text-blue-100">Smart Portability</h2>
      <p class="text-sm text-blue-700 dark:text-blue-300 mb-4">
        Export your Knowledge Base with domain-specific formatting (e.g., CSV for Vocabulary, Markdown for Content). 
        Includes preview and progress tracking.
      </p>
      
      <div class="flex gap-4 items-end">
        <div class="flex-1">
          <label class="block text-sm font-medium mb-1 text-blue-800 dark:text-blue-200">Select Knowledge Base</label>
          <select 
            v-model="selectedKbForPortability" 
            class="w-full px-3 py-2 bg-white dark:bg-gray-900 border border-blue-200 dark:border-blue-700 rounded-md"
          >
            <option v-for="kb in kbs" :key="kb.id" :value="kb.id">
              {{ kb.title }} ({{ kb.renderer_id || 'Default' }})
            </option>
          </select>
        </div>
        <button 
          @click="showExportModal = true" 
          :disabled="!selectedKbForPortability"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 shadow-sm"
        >
          Start Smart Export
        </button>
      </div>
    </div>

    <!-- Restore Section -->
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm border border-gray-200 dark:border-gray-700">
      <h2 class="text-lg font-medium mb-4">System Restore</h2>
      <p class="text-xs text-gray-500 mb-4">
        Upload a .akb file to restore a Knowledge Base. This will create a NEW Knowledge Base copy and will not overwrite existing data.
      </p>
      
      <div class="flex gap-4 items-center">
        <input 
          type="file" 
          ref="fileInput" 
          class="hidden" 
          accept=".akb,.zip"
          @change="handleRestoreUpload" 
        />
        <button 
          @click="triggerRestore" 
          :disabled="restoring"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 flex items-center gap-2"
        >
          <span v-if="restoring">Restoring...</span>
          <span v-else>📤 Upload Backup File</span>
        </button>
      </div>
    </div>

    <!-- Legacy Backup Section -->
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm border border-gray-200 dark:border-gray-700 opacity-75">
      <h2 class="text-lg font-medium mb-4">Legacy System Backup</h2>
      <div class="flex gap-4 items-end">
        <div class="flex-1">
          <label class="block text-sm font-medium mb-1">Select Knowledge Base</label>
          <select 
            v-model="selectedKb" 
            class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-700 rounded-md"
          >
            <option v-for="kb in kbs" :key="kb.id" :value="kb.id">
              {{ kb.title }} ({{ kb.id }})
            </option>
          </select>
        </div>
        <button 
          @click="createBackup" 
          :disabled="!selectedKb || creating"
          class="px-4 py-2 bg-gray-800 dark:bg-gray-200 text-white dark:text-black rounded-md hover:opacity-80 disabled:opacity-50"
        >
          {{ creating ? 'Backing up...' : 'Create Snapshot' }}
        </button>
      </div>
    </div>

    <!-- List Section -->
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm border border-gray-200 dark:border-gray-700">
      <h2 class="text-lg font-medium mb-4">Server Archives</h2>
      <div v-if="loading" class="text-center py-4 text-gray-500">Loading...</div>
      <div v-else-if="backups.length === 0" class="text-center py-4 text-gray-500">No backups found</div>
      <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
        <li v-for="file in backups" :key="file" class="py-3 flex justify-between items-center">
          <div class="flex items-center gap-3">
            <span class="text-xl">📦</span>
            <span class="font-mono text-sm">{{ file }}</span>
          </div>
          <a 
            :href="backupApi.getDownloadUrl(file)" 
            class="text-blue-600 hover:underline text-sm"
          >
            Download
          </a>
        </li>
      </ul>
    </div>

    <!-- Modals -->
    <ExportModal 
      v-if="showExportModal" 
      :isOpen="showExportModal" 
      :kbId="selectedKbForPortability" 
      @close="showExportModal = false" 
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { backupApi } from '../../api/backup';
import { knowledgeApi } from '../../api/knowledge';
import ExportModal from '../portability/ExportModal.vue';

const kbs = ref<any[]>([]);
const backups = ref<string[]>([]);
const selectedKb = ref('');
const selectedKbForPortability = ref('');
const loading = ref(true);
const creating = ref(false);
const restoring = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);
const showExportModal = ref(false);

const loadData = async () => {
  loading.value = true;
  try {
    const [kbList, backupList] = await Promise.all([
      knowledgeApi.list(),
      backupApi.list()
    ]);
    kbs.value = kbList;
    backups.value = backupList;
    
    // Auto-select first KB
    if (kbList.length > 0) {
      selectedKb.value = kbList[0].id;
      selectedKbForPortability.value = kbList[0].id;
    }
  } catch (e) {
    console.error("Failed to load backup data", e);
  } finally {
    loading.value = false;
  }
};

const createBackup = async () => {
  if (!selectedKb.value) return;
  creating.value = true;
  try {
    await backupApi.create(selectedKb.value);
    await loadData(); // Refresh list
    alert('Backup created successfully');
  } catch (e) {
    console.error("Backup failed", e);
    alert('Backup failed');
  } finally {
    creating.value = false;
  }
};

const triggerRestore = () => {
  fileInput.value?.click();
};

const handleRestoreUpload = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files?.length) return;
  
  const file = target.files[0];
  if (!confirm(`Restore backup from ${file.name}? This will create a NEW Knowledge Base.`)) {
      target.value = '';
      return;
  }

  restoring.value = true;
  try {
    await backupApi.restore(file);
    alert('Restoration Complete! The restored KB is now available in your Library.');
    // Ideally redirect or reload KBs
    window.location.reload(); 
  } catch (e) {
    console.error("Restore failed", e);
    alert('Restore failed. Check console for details.');
  } finally {
    restoring.value = false;
    target.value = '';
  }
};

onMounted(loadData);
</script>
