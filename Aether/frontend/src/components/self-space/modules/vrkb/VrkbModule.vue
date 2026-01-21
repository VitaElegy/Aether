<template>
    <div class="h-full flex flex-col bg-paper">
        <!-- Top Toolbar / Navigation -->
        <div class="flex items-center justify-between px-6 py-3 border-b border-ink/10 bg-ash/30 backdrop-blur-sm sticky top-0 z-10">
            <div class="flex items-center gap-4">
                <div class="flex items-center gap-2 text-ink/60 hover:text-ink cursor-pointer transition-colors">
                    <i class="ri-shield-keyhole-line text-lg"></i>
                    <h1 class="font-serif font-bold text-lg tracking-tight">Vulnerability Research</h1>
                </div>

                <div class="h-4 w-px bg-ink/10 mx-2"></div>

                <!-- Tab Switcher -->
                <div class="flex bg-ash/50 rounded-lg p-1 gap-1">
                    <button 
                        v-for="tab in tabs" 
                        :key="tab.id"
                        @click="currentTab = tab.id"
                        class="px-3 py-1 rounded-md text-sm font-medium transition-all"
                        :class="currentTab === tab.id ? 'bg-white shadow-sm text-ink' : 'text-ink-muted hover:text-ink hover:bg-black/5'"
                    >
                        {{ tab.label }}
                    </button>
                </div>
            </div>

            <div class="flex items-center gap-3">
                 <button class="text-ink/60 hover:text-ink transition-colors" title="Settings">
                    <i class="ri-settings-3-line text-lg"></i>
                </button>
            </div>
        </div>

        <!-- Main Content Area -->
        <div class="flex-1 overflow-hidden relative">
            <Transition name="fade" mode="out-in">
                <component :is="currentTabComponent" />
            </Transition>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import ProjectList from './ProjectList.vue';
import VulnerabilityKanban from './VulnerabilityKanban.vue';
import VirtualFinder from './VirtualFinder.vue';

const tabs = [
    { id: 'projects', label: 'Projects' },
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'finder', label: 'Assets' }
];

const currentTab = ref('projects');

const switchTab = (tab: string) => { currentTab.value = tab; };
import { provide } from 'vue';
provide('switchTab', switchTab);

const currentTabComponent = computed(() => {
    switch (currentTab.value) {
        case 'projects': return ProjectList;
        case 'dashboard': return VulnerabilityKanban;
        case 'finder': return VirtualFinder;
        default: return ProjectList;
    }
});
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
</style>
