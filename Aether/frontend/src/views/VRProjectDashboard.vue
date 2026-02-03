<template>
  <div class="h-full flex flex-col bg-bg-base">
    
    <!-- Dashboard Header -->
    <div class="h-16 flex items-center justify-between px-6 border-b border-component-stroke bg-bg-surface shrink-0">
      <div>
        <h1 class="text-xl font-bold text-text-primary">{{ currentProject?.name || 'Loading Project...' }}</h1>
        <div class="text-xs text-text-secondary flex gap-4 mt-1">
           <span>Quota: {{ formatBytes(currentProject?.quota_bytes || 0) }}</span>
           <span v-if="currentProject?.repository_url" class="text-brand-primary cursor-pointer hover:underline">
             {{ currentProject.repository_url }}
           </span>
        </div>
      </div>

      <button 
        @click="openNewFinding"
        class="bg-brand-primary hover:bg-brand-primary-hover text-white px-4 py-2 rounded shadow-sm text-sm font-medium transition-colors"
      >
        + New Finding
      </button>
    </div>

    <!-- Kanban Board -->
    <div class="flex-1 overflow-x-auto overflow-y-hidden p-6" v-if="isReady">
      <div class="flex gap-6 h-full min-w-max">
        
        <!-- Column Component ( Inline for simplicity or extract ) -->
        <div v-for="col in columns" :key="col.id" class="w-80 flex flex-col h-full">
          <!-- Column Header -->
          <div class="flex items-center justify-between mb-4 px-2">
            <div class="flex items-center gap-2">
              <div :class="`w-2 h-2 rounded-full ${col.color}`"></div>
              <h3 class="font-semibold text-text-secondary text-sm uppercase tracking-wide">{{ col.title }}</h3>
            </div>
            <span class="text-xs font-bold text-text-disabled bg-bg-surface px-2 py-0.5 rounded-full border border-component-stroke">
              {{ getFindingsByStatus(col.id).length }}
            </span>
          </div>

          <!-- Cards Container -->
          <div class="flex-1 overflow-y-auto pr-2 space-y-3 custom-scrollbar">
            <div 
              v-for="item in getFindingsByStatus(col.id)" 
              :key="item.id"
              @click="openFinding(item)"
              class="group bg-bg-surface border border-component-stroke rounded-lg p-3 shadow-sm hover:shadow-md hover:border-brand-primary cursor-pointer transition-all active:scale-[0.98]"
            >
              <div class="flex justify-between items-start mb-2">
                 <span :class="`text-[10px] font-bold px-1.5 py-0.5 rounded border ${getSeverityClass(item.severity)}`">
                   {{ item.severity }}
                 </span>
                 <span class="text-xs text-text-disabled">{{ formatDate(item.updated_at) }}</span>
              </div>
              <h4 class="text-sm font-medium text-text-primary leading-snug mb-2 group-hover:text-brand-primary transition-colors">
                {{ item.title }}
              </h4>
              <div class="flex items-center gap-2 text-xs text-text-secondary" v-if="item.section_id">
                 <!-- Mock Section Name, ideally we map ID to Section Title -->
                 <span class="flex items-center gap-1 opacity-70">
                   <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
                   Section
                 </span>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>
    
    <!-- Skeleton Loading -->
    <div class="flex-1 p-6 flex gap-6" v-else>
         <div v-for="i in 4" :key="i" class="w-80 h-full flex flex-col gap-4">
              <div class="h-6 w-32 bg-bg-surface/50 rounded animate-pulse"></div>
              <div class="h-full bg-bg-surface/20 rounded-lg animate-pulse border border-component-stroke/30"></div>
         </div>
    </div>

    <!-- Finding Editor Modal -->
    <VRFindingEditor 
      v-if="editingFinding" 
      :finding="editingFinding" 
      @close="editingFinding = null" 
      @save="handleSaveFinding"
    />

  </div>
</template>



<script setup lang="ts">
import { ref, onMounted, computed, onActivated } from 'vue';
import { useRoute } from 'vue-router';
import { vrkbApi, type VrkbProject, type VrkbFinding } from '@/api/vrkb';
import VRFindingEditor from '@/components/vrkb/VRFindingEditor.vue';
import { DateTime } from 'luxon';

const route = useRoute();
const currentProject = ref<VrkbProject | null>(null);
const findings = ref<VrkbFinding[]>([]);
const editingFinding = ref<VrkbFinding | null>(null);
const isReady = ref(false);

const loadData = async () => {
    isReady.value = false;
    try {
        // 1. Fetch Projects
        const projects = await vrkbApi.listProjects();
        if (projects.length > 0) {
            currentProject.value = projects[0];
            projectId.value = projects[0].id;
            // 2. Fetch Findings
            const f = await vrkbApi.listFindings(projectId.value);
            findings.value = f;
        } else {
             // Create default project if none exists (Auto-init for demo)
             console.log("No projects found, creating default...");
             const newProj = await vrkbApi.createProject({
                 name: 'Main Audit Target',
                 quota_bytes: 1024 * 1024 * 100 // 100MB
             });
             currentProject.value = newProj;
             projectId.value = newProj.id;
        }
    } catch (e) {
        console.error("Failed to load VRKB data", e);
    } finally {
        isReady.value = true;
    }
};

onMounted(() => {
    loadData();
});

onActivated(() => {
    console.log('[VRKB] Re-activated, checking for updates...');
    refreshFindings();
});
const columns = [
  { id: 'Pending', title: 'Pending', color: 'bg-gray-400' },
  { id: 'Triage', title: 'Triage', color: 'bg-blue-500' },
  { id: 'Fixing', title: 'Fixing', color: 'bg-brand-primary' },
  { id: 'Verified', title: 'Verified', color: 'bg-green-500' },
];

const kbId = computed(() => route.params.id as string);

// ... (logic remains same) ...

const projectId = ref<string>('');


const refreshFindings = async () => {
    if (!projectId.value) return;
    findings.value = await vrkbApi.listFindings(projectId.value);
};

// ... (rest remains same) ...

const getFindingsByStatus = (status: string) => {
    return findings.value.filter(f => f.status === status);
};

const openNewFinding = () => {
    vrkbApi.listSections(projectId.value).then(sections => {
        let sectionId = '';
        if (sections.length > 0) {
            sectionId = sections[0].id;
        } else {
             vrkbApi.createSection({ project_id: projectId.value, title: 'General' }).then(s => {
                 sectionId = s.id;
                 startEdit(sectionId);
             });
             return;
        }
        startEdit(sectionId);
    });
};

const startEdit = (sectionId: string) => {
    editingFinding.value = {
        id: '', // New
        section_id: sectionId,
        title: '',
        status: 'Pending',
        severity: 'Medium',
        is_triage: false,
        created_at: '',
        updated_at: ''
    };
};

const openFinding = (item: VrkbFinding) => {
    editingFinding.value = item;
};

const handleSaveFinding = async (item: VrkbFinding) => {
    try {
        if (!item.id) {
            await vrkbApi.createFinding(item);
        } else {
            await vrkbApi.updateFinding(item.id, item);
        }
        await refreshFindings();
        editingFinding.value = null;
    } catch (e) {
        console.error("Failed to save", e);
        alert("Failed to save finding");
    }
};

// Utilities
const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const formatDate = (dateStr?: string) => {
    if (!dateStr) return '';
    return DateTime.fromISO(dateStr).toRelative() || '';
};

const getSeverityClass = (sev: string) => {
    switch(sev) {
        case 'Critical': return 'text-red-600 border-red-200 bg-red-50';
        case 'High': return 'text-orange-600 border-orange-200 bg-orange-50';
        case 'Medium': return 'text-yellow-600 border-yellow-200 bg-yellow-50';
        case 'Low': return 'text-blue-600 border-blue-200 bg-blue-50';
        default: return 'text-gray-600 border-gray-200 bg-gray-50';
    }
};

</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.1);
  border-radius: 4px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.2);
}
</style>
