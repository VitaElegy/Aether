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
        <Transition name="fade">
            <div v-if="showCreateModal" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-[4px]" @click.self="showCreateModal = false">
                <Transition name="slide-up">
                    <div class="bg-paper shadow-2xl w-full max-w-md border border-ash/20 rounded-xl overflow-hidden relative" v-if="showCreateModal">
                         <!-- Close Button -->
                        <button @click="showCreateModal = false" class="absolute top-4 right-4 p-2 text-ink/40 hover:text-ink transition-colors z-10 rounded-full hover:bg-black/5">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                        </button>

                        <div class="p-8">
                            <h3 class="text-2xl font-black font-serif text-ink tracking-tight mb-6">Create Project</h3>
                            
                            <div class="space-y-6">
                                <div>
                                    <label class="block text-[10px] font-black uppercase tracking-widest text-ink/40 mb-2">Project Name</label>
                                    <input v-model="newProjectName" type="text" class="w-full bg-surface border border-ash/20 rounded-lg px-4 py-3 text-ink font-serif text-lg focus:outline-none focus:border-accent transition-colors placeholder-ink/20" placeholder="e.g. Linux Kernel Analysis" />
                                </div>
                                <div>
                                    <label class="block text-[10px] font-black uppercase tracking-widest text-ink/40 mb-2">Repository URL <span class="text-ink/20">(Optional)</span></label>
                                    <input v-model="newProjectRepo" type="text" class="w-full bg-surface border border-ash/20 rounded-lg px-4 py-3 text-ink font-mono text-sm focus:outline-none focus:border-accent transition-colors placeholder-ink/20" placeholder="https://github.com/..." />
                                </div>
                            </div>

                            <div class="flex justify-end gap-3 mt-10 pt-6 border-t border-ash/10">
                                <button @click="showCreateModal = false" class="text-ink/60 hover:text-ink px-4 py-2 hover:bg-surface rounded-lg transition-colors text-xs font-bold uppercase tracking-wider">Cancel</button>
                                <button 
                                    @click="handleCreate" 
                                    :disabled="!newProjectName || creating"
                                    class="bg-accent text-white px-6 py-2 rounded-lg shadow-lg shadow-accent/20 hover:shadow-accent/40 hover:-translate-y-0.5 transition-all font-bold text-xs uppercase tracking-wider disabled:opacity-50 disabled:translate-y-0"
                                >
                                    {{ creating ? 'Creating...' : 'Create Project' }}
                                </button>
                            </div>
                        </div>
                    </div>
                </Transition>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, inject } from 'vue';
import { useVrkbStore } from '@/stores/vrkb';
import { DateTime } from 'luxon';

const store = useVrkbStore();
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
    try {
        await store.selectProject(id);
        // Navigation is handled by parent (VrkbModule) reacting to store.currentProject
    } catch (e: any) {
        console.error(e);
        alert("Failed to open project: " + (e.response?.data?.message || e.message));
    }
};

const handleCreate = async () => {
    if (!newProjectName.value) return;
    creating.value = true;
    try {
        await store.createProject(newProjectName.value, newProjectRepo.value || undefined);
        showCreateModal.value = false;
        newProjectName.value = '';
        newProjectRepo.value = '';
    } catch (e: any) {
        console.error(e);
        alert("Failed to create project: " + (e.response?.data?.message || e.message));
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
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.98);
}
</style>
