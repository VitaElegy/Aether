<template>
    <div class="h-full flex flex-col p-6 overflow-y-auto">
        <div class="flex items-center justify-between mb-8">
            <h2 class="text-2xl font-serif text-ink tracking-tight">Projects</h2>
            <button @click="showCreateModal = true" class="bg-accent text-white px-4 py-2 rounded-md hover:bg-accent/90 transition-colors font-medium flex items-center gap-2">
                <i class="ri-add-line text-lg"></i>
                New Project
            </button>
        </div>

        <!-- Project Grid -->
        <div v-if="loading" class="flex items-center justify-center py-12">
            <div class="animate-spin h-8 w-8 border-2 border-accent border-t-transparent rounded-full"></div>
        </div>

        <div v-else-if="projects.length === 0" class="flex flex-col items-center justify-center flex-1 text-ink/60">
            <i class="ri-folder-warning-line text-4xl mb-4 opacity-50"></i>
            <p>No projects found</p>
            <button @click="showCreateModal = true" class="text-accent hover:underline mt-2">Create your first project</button>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div 
                v-for="project in projects" 
                :key="project.id"
                @click="openProject(project.id)"
                class="bg-ash/30 border border-ink/10 rounded-lg p-6 hover:shadow-lg hover:border-accent/50 transition-all cursor-pointer group"
            >
                <div class="flex items-start justify-between mb-4">
                    <div class="p-3 bg-ash/50 rounded-md text-accent group-hover:bg-accent group-hover:text-white transition-colors">
                        <i class="ri-shield-keyhole-line text-xl"></i>
                    </div>
                     <i class="ri-arrow-right-line text-ink/60 opacity-0 group-hover:opacity-100 transition-opacity"></i>
                </div>
                
                <h3 class="text-lg font-bold text-ink mb-1 group-hover:text-accent transition-colors">{{ project.name }}</h3>
                <div class="text-xs text-ink/60 flex items-center gap-2 mb-4">
                    <span v-if="project.repository_url" class="flex items-center gap-1">
                        <i class="ri-github-fill"></i> Linked
                    </span>
                    <span v-else>Local Project</span>
                    <span>•</span>
                    <span>{{ formatDate(project.created_at) }}</span>
                </div>

                <!-- Usage Bar (Mock) -->
                <div class="w-full bg-paper-3 h-1.5 rounded-full overflow-hidden">
                    <div class="bg-accent/50 h-full w-1/12"></div>
                </div>
                <div class="flex justify-between text-[10px] text-ink-muted mt-2">
                    <span>Usage</span>
                    <span>{{ formatBytes(0) }} / {{ formatBytes(project.quota_bytes) }}</span>
                </div>
            </div>
        </div>

        <!-- Create Modal -->
        <div v-if="showCreateModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
            <div class="bg-paper-1 rounded-xl shadow-2xl w-full max-w-md p-6 border border-ink-border">
                <h3 class="text-xl font-bold font-serif mb-4">Create Project</h3>
                
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-ink/70 mb-1">Project Name</label>
                        <input v-model="newProjectName" type="text" class="w-full bg-paper/50 border border-ink/20 rounded-md px-3 py-2 text-ink focus:outline-none focus:border-accent transition-colors" placeholder="e.g. Linux Kernel Analysis" />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-ink/70 mb-1">Repository URL (Optional)</label>
                        <input v-model="newProjectRepo" type="text" class="w-full bg-paper/50 border border-ink/20 rounded-md px-3 py-2 text-ink focus:outline-none focus:border-accent transition-colors" placeholder="https://github.com/..." />
                    </div>
                </div>

                <div class="flex justify-end gap-3 mt-8">
                    <button @click="showCreateModal = false" class="text-ink/60 hover:text-ink px-4 py-2 hover:bg-paper/50 rounded-md transition-colors">Cancel</button>
                    <button 
                        @click="handleCreate" 
                        :disabled="!newProjectName || creating"
                        class="bg-accent text-white px-4 py-2 rounded-md hover:bg-accent/90 transition-colors font-medium"
                    >
                        {{ creating ? 'Creating...' : 'Create Project' }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, inject } from 'vue';
import { useVrkbStore } from '@/stores/vrkb';
import { DateTime } from 'luxon';

const store = useVrkbStore();
const switchTab = inject('switchTab') as (tab: string) => void;
const showCreateModal = ref(false);
const newProjectName = ref('');
const newProjectRepo = ref('');
const creating = ref(false);

const projects = computed(() => store.projects);
const loading = computed(() => store.isLoading);

onMounted(() => {
    store.fetchProjects();
});

const openProject = async (id: string) => {
    await store.selectProject(id);
    switchTab('dashboard');
};

const handleCreate = async () => {
    if (!newProjectName.value) return;
    creating.value = true;
    try {
        await store.createProject(newProjectName.value, newProjectRepo.value || undefined);
        showCreateModal.value = false;
        newProjectName.value = '';
        newProjectRepo.value = '';
    } catch (e) {
        console.error(e);
    } finally {
        creating.value = false;
    }
};

const formatDate = (iso: string) => {
    return DateTime.fromISO(iso).toRelative();
};

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};
</script>

<style scoped>
</style>
