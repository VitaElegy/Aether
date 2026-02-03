<script setup lang="ts">
/**
 * SelfSpaceView - The Desktop Shell
 * 
 * REFACTORED: Now uses useSelfSpaceOrchestrator for all state management.
 * This component is now a thin presentation layer.
 */
import { onMounted, provide, onErrorCaptured, watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import TopNavBar from '../components/TopNavBar.vue';
import ModuleSwitcher from '../components/self-space/ModuleSwitcher.vue';
import { useNavigationStore } from '../stores/navigation';
import { usePluginStore } from '../stores/plugins';
import { useSelfSpaceOrchestrator } from '../composables/useSelfSpaceOrchestrator';
import { useAppStateStore } from '../stores/read_app_state';
import { MessagePlugin } from 'tdesign-vue-next';
import { osKey } from '../composables/useOS';

const router = useRouter();
const navStore = useNavigationStore();
const pluginStore = usePluginStore();
const appStore = useAppStateStore();

// ============================================================
// ORCHESTRATOR (Single Source of Truth)
// ============================================================
const orchestrator = useSelfSpaceOrchestrator();

// Initialize on mount
onMounted(async () => {
    await orchestrator.initialize();
});

// ============================================================
// ACTIONS (Delegated to Orchestrator)
// ============================================================
// ============================================================
// ACTIONS (Delegated to Orchestrator + Router)
// ============================================================
const launchApp = async (kbId: string) => {
    // 1. Update URL (Source of Truth)
    // using replace to prevent history spam, unless it's a distinct navigation context?
    // User requested "silky smooth", replace is usually better for tab-switching feel.
    router.replace({ name: 'space', params: { kbId } });
};

const closeApp = (kbId: string) => {
    orchestrator.closeKb(kbId);
    if (appStore.activeKbId === kbId) {
        router.replace({ name: 'space', params: { kbId: 'library' } });
    }
};

const goHome = () => {
    router.replace({ name: 'space', params: { kbId: 'library' } });
};

// ============================================================
// ROUTE SYNC (The Listener)
// ============================================================
watch(() => router.currentRoute.value.params.kbId, async (newId) => {
    const target = Array.isArray(newId) ? newId[0] : newId;
    if (target) {
        await orchestrator.switchToKb(target);
    } else {
        // If /space (empty), default to library
        await orchestrator.switchToKb('library');
    }
}, { immediate: true });

// ============================================================
// PROVIDE OS CONTEXT TO CHILDREN
// ============================================================
// ============================================================
// PROVIDE OS CONTEXT TO CHILDREN
// ============================================================
// OS Service Injection (Standard v2)
const osContext = {
    launchApp,
    closeApp,
    toast: (content: string, theme: 'success' | 'warning' | 'error' = 'success') => MessagePlugin[theme]({ content }),
};
provide(osKey, osContext);
// Legacy string inject for old components (if any)
provide('os', osContext);

// ============================================================
// DERIVED STATE (From Orchestrator)
// ============================================================
const currentModuleLabel = () => orchestrator.currentDockItem.value?.dock.label || 'Self Space';
const currentHeaderIcon = () => orchestrator.currentDockItem.value?.dock.icon;
const currentKbId = () => orchestrator.currentDockItem.value?.id;

// Header Actions Resolution
const CurrentHeaderActions = () => {
    const item = orchestrator.currentDockItem.value;
    if (!item) return null;

    // Library has no special header
    if (item.id === 'library') return null;

    // Try to resolve plugin header
    const resolved = pluginStore.resolvePlugin(item._renderer_id);
    if (resolved?.header?.actions) return resolved.header.actions;

    return null;
};

// ============================================================
// ERROR BOUNDARY (Layer 2: KB-Level)
// ============================================================
onErrorCaptured((err, instance, info) => {
    console.error('[SelfSpace] Captured Component Error:', err, info);

    const activeId = appStore.activeKbId;
    if (activeId !== 'library') {
        // Crash to Desktop - use timeout to allow error to propagate
        setTimeout(() => {
            // TODO: Replace alert with Toast (per spec)
            console.error(`[SelfSpace] Application Crashed: ${activeId}`);
            orchestrator.crashKb(activeId, err instanceof Error ? err : new Error(String(err)));
        }, 100);
        return false; // Prevent bubbling
    }
    return true; // Bubble if shell itself
});

// ============================================================
// DEBUG WATCHERS (Remove in production)
// ============================================================
watch(() => appStore.activeKbId, (val) => {
    console.log('[SelfSpace] Active KB Changed:', val);
});
const activeKb = computed(() => {
    const id = currentKbId();
    if (!id || id === 'library') return undefined; // Library doesn't use generic KB object usually
    return orchestrator.allKbs.value.find(k => k.id === id);
});
</script>

<template>
    <div class="h-screen bg-paper text-ink selection:bg-accent/20 flex flex-col relative overflow-hidden">
        <!-- Ambient Background Elements -->
        <div class="absolute top-0 left-0 w-full h-96 bg-gradient-to-b from-ash/10 to-transparent pointer-events-none">
        </div>

        <!-- Top Navigation Bar -->
        <TopNavBar>
            <template #left>
                <!-- SYSTEM HEADER (Traffic Lights) -->
                <div class="flex items-center gap-2 pl-2">
                    <!-- Minimal Home Button -->
                    <button 
                        v-if="appStore.activeKbId !== 'library'"
                        @click="goHome"
                        class="w-6 h-6 rounded-md hover:bg-ink/5 text-ink/40 hover:text-ink transition-colors flex items-center justify-center"
                        title="Back to Dashboard"
                    >
                         <i class="ri-layout-grid-fill text-sm"></i>
                    </button>

                    <!-- App Title Separator -->
                    <div class="h-4 w-[1px] bg-ink/10 mx-2"></div>
                    
                    <!-- Current App Title -->
                    <div v-if="appStore.activeKbId !== 'library'" class="text-xs font-bold font-serif text-ink select-none flex items-center gap-2">
                         <i v-if="currentHeaderIcon()" :class="currentHeaderIcon()" class="text-ink/60"></i>
                         {{ currentModuleLabel() }}
                    </div>
                </div>
            </template>

            <template #center>
                <!-- DYNAMIC CENTER PORTAL -->
                <div id="nav-center-portal" class="contents"></div>
            </template>

            <template #right>
                <!-- DYNAMIC RIGHT PORTAL -->
                <div id="nav-right-portal" class="contents"></div>

                <!-- Default Right Content -->
                <div v-show="!navStore.hasCustomRight" class="flex items-center gap-2">
                    <!-- Plugin Default Actions -->
                    <component :is="CurrentHeaderActions()" v-if="CurrentHeaderActions()" />

                     <!-- Default Global Actions -->
                     <button 
                        @click="router.push('/settings')"
                        class="text-ink/30 hover:text-ink transition-colors px-2 py-1"
                        title="Settings"
                    >
                        <i class="ri-settings-4-line text-lg"></i>
                    </button>
                </div>
            </template>
        </TopNavBar>

        <!-- Loading State (Phase 3) -->
        <div v-if="orchestrator.isLoading.value && orchestrator.loadingKbId.value" 
             class="flex-1 flex items-center justify-center">
            <div class="text-center">
                <i class="ri-loader-4-line animate-spin text-2xl text-ink/30 mb-2"></i>
                <p class="text-sm text-ink/50">Loading...</p>
            </div>
        </div>

        <!-- Main Content Area -->
        <main v-else class="flex-1 relative z-10 w-full h-full flex flex-col pt-14">
            <Transition mode="out-in" enter-active-class="transition duration-300 ease-out"
                enter-from-class="opacity-0 translate-y-4" enter-to-class="opacity-100 translate-y-0"
                leave-active-class="transition duration-200 ease-in" leave-from-class="opacity-100 translate-y-0"
                leave-to-class="opacity-0 -translate-y-4">
                <KeepAlive :max="10">
                    <component 
                        :is="orchestrator.currentComponent.value" 
                        :kbId="currentKbId()" 
                        :kb="activeKb"
                        v-bind="orchestrator.errorState.value.get(appStore.activeKbId) ? {
                            errorMessage: orchestrator.errorState.value.get(appStore.activeKbId)?.message,
                            errorStack: orchestrator.errorState.value.get(appStore.activeKbId)?.stack,
                            rendererId: orchestrator.currentDockItem.value?._renderer_id
                        } : {}"
                        class="flex-1 h-full" 
                        :headless="true" 
                    />
                </KeepAlive>
            </Transition>
        </main>

        <!-- Dock Navigation -->
        <ModuleSwitcher 
            :active-module="appStore.activeKbId" 
            :modules="orchestrator.dockItems.value" 
            @switch="launchApp" 
            @close="closeApp" 
        />
    </div>
</template>
