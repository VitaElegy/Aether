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
            <!-- Global Nav Injection -->
            <Teleport to="#nav-right-portal">
                <div class="flex items-center gap-4 h-full">
                     <!-- Back Button (Only when subview) -->
                    <button v-if="currentView !== 'dashboard'" @click="backToDashboard" class="px-3 py-1.5 bg-ink/5 hover:bg-ink/10 text-ink rounded-lg text-xs font-bold flex items-center gap-2 transition-colors">
                        <i class="ri-arrow-left-line"></i>
                        Back
                    </button>
                    <!-- Settings Button (Shortcut) -->
                    <button v-if="currentView === 'dashboard'" @click="currentView = 'settings'" class="w-8 h-8 rounded-full bg-ink/5 hover:bg-ink/10 text-ink/70 flex items-center justify-center transition-colors">
                        <i class="ri-settings-4-line"></i>
                    </button>

                    <div class="hidden md:flex px-3 py-1.5 bg-green-500/10 text-green-600 rounded-full text-[10px] font-bold border border-green-500/20 items-center gap-2">
                        <span class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></span>
                        System Online
                    </div>
                </div>
            </Teleport>

            <Teleport to="#nav-center-portal">
                 <div class="flex items-center justify-center h-full">
                    <div class="flex items-center gap-2 text-ink/80 text-sm font-bold font-serif">
                        <span :class="currentView === 'dashboard' ? 'text-ink' : 'text-ink/40 hover:text-ink cursor-pointer transition-colors'" @click="backToDashboard">Admin Control Center</span>
                        
                        <template v-if="currentView !== 'dashboard'">
                            <span class="text-ink/30">/</span>
                            <span>
                                {{ currentView === 'templates' ? 'Templates' : 
                                   currentView === 'users' ? 'User Management' : 
                                   currentView === 'logs' ? 'System Logs' : 
                                   currentView === 'settings' ? 'Global Settings' : '' }}
                            </span>
                        </template>
                    </div>
                 </div>
            </Teleport>

            <!-- Spacing Adjustments for Banner Removal -->
            <div class="p-8 pb-0">
                <h1 v-if="currentView === 'dashboard'" class="text-3xl font-bold font-serif text-ink mb-4">Welcome, Administrator.</h1>
                <p class="text-ink/60 mb-8 max-w-lg">Manage system resources, users, and templates from this centralized dashboard.</p>
            </div>

            <!-- VIEW: Template Manager -->
            <div v-if="currentView === 'templates'" class="animate-fade-in">
                 <TemplateManager />
            </div>

            <!-- VIEW: User Management -->
            <div v-else-if="currentView === 'users'" class="animate-fade-in">
                 <UserManagement />
            </div>

            <!-- VIEW: System Logs -->
            <div v-else-if="currentView === 'logs'" class="animate-fade-in">
                 <SystemLogs />
            </div>

            <!-- VIEW: Settings -->
            <div v-else-if="currentView === 'settings'" class="animate-fade-in">
                 <SystemSettings />
            </div>

            <!-- VIEW: Dashboard Grid -->
            <div v-else class="p-8 max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-fade-in">
                
                <!-- Card: Templates -->
                <div @click="currentView = 'templates'"
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
                <div @click="currentView = 'users'"
                    class="group relative bg-surface p-6 rounded-xl border border-ink/5 shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all cursor-pointer overflow-hidden">
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
                <div @click="currentView = 'logs'"
                    class="group relative bg-surface p-6 rounded-xl border border-ink/5 shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all cursor-pointer overflow-hidden">
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
import { ref, onMounted, onBeforeUnmount, onActivated, defineAsyncComponent, shallowRef } from 'vue';
import { useNavigationStore } from '@/stores/navigation';

// Internal Views
import TemplateManager from '../admin/TemplateManager.vue';
import UserManagement from '../admin/UserManagement.vue';
import SystemLogs from '../admin/SystemLogs.vue';
import SystemSettings from '../admin/SystemSettings.vue';

const router = useRouter();
const navStore = useNavigationStore();
const isReady = ref(false);

// [FIX] Internal Navigation State (No Jailbreak)
const currentView = ref<'dashboard' | 'templates' | 'users' | 'logs' | 'settings'>('dashboard');

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
    navStore.setCustomCenter(true);
    navStore.setCustomRight(true);
});

onBeforeUnmount(() => {
    navStore.reset();
});

onActivated(() => {
    console.log('[Admin] Dashboard Reactivated');
    // Ensure flags are set if re-entering from KeepAlive
    navStore.setCustomCenter(true);
    navStore.setCustomRight(true);
});
</script>
