<template>
    <div class="h-full w-full bg-ash/5 overflow-y-auto">
        <!-- Skeleton -->
        <div v-if="!isReady" class="p-8 animate-pulse">
             <div class="w-full h-32 bg-ash/10 rounded-xl mb-8"></div>
             <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                 <div class="h-48 bg-ash/10 rounded-xl"></div>
                 <div class="h-48 bg-ash/10 rounded-xl"></div>
                 <div class="h-48 bg-ash/10 rounded-xl"></div>
             </div>
        </div>

        <!-- Real Content -->
        <div v-else>
            <!-- Banner -->
            <div class="w-full bg-surface border-b border-ink/5 p-8 flex items-end justify-between">
                <div>
                    <div class="flex items-center gap-2 mb-2 text-ink/40 text-xs font-bold uppercase tracking-widest">
                        <i class="ri-shield-star-line text-accent"></i>
                        <span>System Administrator</span>
                    </div>
                    <!-- Breadcrumbs / Title -->
                    <div class="flex items-center gap-2">
                        <h1 @click="backToDashboard" :class="currentView !== 'dashboard' ? 'cursor-pointer hover:text-accent transition-colors' : ''" class="text-4xl font-bold font-serif text-ink">Admin Control Center</h1>
                        <span v-if="currentView === 'templates'" class="text-2xl text-ink/30">/</span>
                        <h2 v-if="currentView === 'templates'" class="text-3xl font-serif text-ink/80">Templates</h2>
                    </div>
                    
                    <p class="text-ink/60 mt-2 max-w-lg">Manage system resources, users, and templates from this centralized dashboard.</p>
                </div>
                
                <div class="flex items-center gap-4">
                     <!-- Back Button (Only when subview) -->
                    <button v-if="currentView !== 'dashboard'" @click="backToDashboard" class="px-4 py-2 bg-ink text-white rounded-lg text-xs font-bold flex items-center gap-2 hover:bg-ink/80 transition-colors">
                        <i class="ri-arrow-left-line"></i>
                        Back
                    </button>

                    <div class="px-4 py-2 bg-green-500/10 text-green-600 rounded-full text-xs font-bold border border-green-500/20 flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
                        System Online
                    </div>
                </div>
            </div>

            <!-- VIEW: Template Manager -->
            <div v-if="currentView === 'templates'" class="animate-fade-in">
                 <TemplateManager />
            </div>

            <!-- VIEW: Dashboard Grid -->
            <div v-else class="p-8 max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-fade-in">
                
                <!-- Card: Templates -->
                <div @click="openTemplates"
                    class="group relative bg-surface p-6 rounded-xl border border-ink/5 shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all cursor-pointer overflow-hidden">
                    <div class="absolute top-0 right-0 p-3 opacity-10 group-hover:opacity-20 transition-opacity">
                        <i class="ri-layout-masonry-line text-8xl"></i>
                    </div>
                    
                    <div class="relative z-10">
                        <div class="w-12 h-12 rounded-lg bg-accent/10 flex items-center justify-center text-accent mb-4 group-hover:bg-accent group-hover:text-white transition-colors">
                            <i class="ri-layout-2-line text-2xl"></i>
                        </div>
                        <h3 class="text-xl font-bold text-ink mb-1 group-hover:text-accent transition-colors">Template Manager</h3>
                        <p class="text-sm text-ink/50 leading-relaxed">Create, edit, and assign layout templates for knowledge bases.</p>
                    </div>
                </div>

                <!-- Card: User Management -->
                <div class="group relative bg-surface p-6 rounded-xl border border-ink/5 shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all cursor-pointer overflow-hidden">
                    <div class="absolute top-0 right-0 p-3 opacity-10 group-hover:opacity-20 transition-opacity">
                        <i class="ri-group-line text-8xl"></i>
                    </div>
                    
                    <div class="relative z-10">
                        <div class="w-12 h-12 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-500 mb-4 group-hover:bg-blue-500 group-hover:text-white transition-colors">
                            <i class="ri-user-settings-line text-2xl"></i>
                        </div>
                        <h3 class="text-xl font-bold text-ink mb-1 group-hover:text-blue-500 transition-colors">User Management</h3>
                        <p class="text-sm text-ink/50 leading-relaxed">Manage registered users, roles, and permissions.</p>
                    </div>
                </div>

                <!-- Card: System Logs -->
                <div class="group relative bg-surface p-6 rounded-xl border border-ink/5 shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all cursor-pointer overflow-hidden">
                    <div class="absolute top-0 right-0 p-3 opacity-10 group-hover:opacity-20 transition-opacity">
                        <i class="ri-terminal-line text-8xl"></i>
                    </div>
                    
                    <div class="relative z-10">
                        <div class="w-12 h-12 rounded-lg bg-orange-500/10 flex items-center justify-center text-orange-500 mb-4 group-hover:bg-orange-500 group-hover:text-white transition-colors">
                            <i class="ri-file-list-3-line text-2xl"></i>
                        </div>
                        <h3 class="text-xl font-bold text-ink mb-1 group-hover:text-orange-500 transition-colors">System Logs</h3>
                        <p class="text-sm text-ink/50 leading-relaxed">View system output, audit trails, and error reports.</p>
                    </div>
                </div>

            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { ref, onMounted, onActivated, defineAsyncComponent, shallowRef } from 'vue';

// Internal Views
import TemplateManager from '../admin/TemplateManager.vue';

const router = useRouter();
const isReady = ref(false);

// [FIX] Internal Navigation State (No Jailbreak)
const currentView = ref<'dashboard' | 'templates'>('dashboard');

const openTemplates = () => {
    currentView.value = 'templates';
};

const backToDashboard = () => {
    currentView.value = 'dashboard';
};

const loadData = async () => {
    isReady.value = false;
    // Simulate Fetch
    await new Promise(resolve => setTimeout(resolve, 600));
    isReady.value = true;
};

onMounted(() => {
    loadData();
});

onActivated(() => {
    console.log('[Admin] Dashboard Reactivated');
});
</script>
