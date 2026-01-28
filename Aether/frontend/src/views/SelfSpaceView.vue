<script setup lang="ts">
import { ref, computed, watch } from 'vue'; // [FIXED]
import { useRouter } from 'vue-router';
import TopNavBar from '../components/TopNavBar.vue';
import ModuleSwitcher from '../components/self-space/ModuleSwitcher.vue';
import { usePluginStore } from '../stores/plugins';
import { useVocabularyStore } from '../stores/vocabulary';
import { useNavigationStore } from '../stores/navigation';

const router = useRouter();
const pluginStore = usePluginStore();
const vocabStore = useVocabularyStore();
const navStore = useNavigationStore();
const prefStore = usePreferencesStore(); // [NEW]
import { usePreferencesStore } from '../stores/preferences';
import { knowledgeApi, type KnowledgeBase } from '../api/knowledge';

// Default to 'library' (the manager)
const currentModuleId = ref('library');

// -- Dynamic Dock Logic --
const pinnedKbs = ref<KnowledgeBase[]>([]);
const allKbs = ref<KnowledgeBase[]>([]); // [NEW] Cache
const isLoadingDock = ref(false);

const updateDock = async () => {
    // We fetch all KBs and filter by pinned IDs
    // Optimization: In real app, we should have a 'get_many' endpoint or just list.
    // List is lightweight enough.
    isLoadingDock.value = true;
    try {
        const all = await knowledgeApi.list();
        allKbs.value = all;
        pinnedKbs.value = all.filter(kb => prefStore.isPinned(kb.id));
    } catch (e) {
        console.error("Dock update failed", e);
    } finally {
        isLoadingDock.value = false;
    }
};

// Initial Load & Watch
watch(() => prefStore.pinnedKbIds, updateDock, { deep: true });
// Also fetch on mount
updateDock();

// -- V3 Shell State --
const runningApps = ref<Set<string>>(new Set());

// -- Actions --
const launchApp = async (kbId: string) => {
    // 0. Ensure we have metadata (for new KBs)
    if (!allKbs.value.find(k => k.id === kbId)) {
        await updateDock();
    }

    // 1. Add to running apps (if not pinned)
    if (!prefStore.isPinned(kbId)) {
        runningApps.value.add(kbId);
    }
    // 2. Switch to it
    currentModuleId.value = kbId;
    // 3. Reset URL for pure launch
    router.replace({ query: {} });
};

const closeApp = (kbId: string) => {
    // 1. Remove from running apps
    runningApps.value.delete(kbId);
    
    // 2. If it was active, go home
    if (currentModuleId.value === kbId) {
        currentModuleId.value = 'library';
        router.replace({ query: {} });
    }
    
    // 3. Deep Cleanup (Store Reset)
    // We should ideally call a cleanup hook on the plugin, but for now we rely on onUnmounted
};

const goHome = () => {
   currentModuleId.value = 'library';
   router.replace({ query: {} });
};


// Compute Dock Items (V3: Pinned + Running)
const dockItems = computed(() => {
    // 1. The "Launchpad" (Library) - Always First
    const libraryPlugin = pluginStore.getPlugin('knowledge') || {
        id: 'library',
        dock: { label: 'Launchpad', icon: 'ri-apps-2-line', order: 0 },
        component: {} as any,
        capabilities: []
    } as any;
    libraryPlugin.pinned = true; // [NEW] System Fixed
    
    // 2. Aggregate Items (Pinned U Running)
    const groups: Record<string, any[]> = {};
    
    // Helper to process KB into DockItem
    const processKb = (kb: KnowledgeBase) => {
        const renderer = kb.renderer_id || 'default';
        const manifest = pluginStore.getManifest(renderer);
        
        let icon = 'ri-book-2-line';
        if (manifest) icon = manifest.identity.icon;
        else if (renderer === 'vuln' || renderer === 'vrkb') icon = 'ri-shield-keyhole-line';

        return {
            id: kb.id,
            dock: {
                label: kb.title,
                icon: icon,
                order: 10
            },
            _renderer_id: renderer,
            _kb_id: kb.id,
            _manifest: manifest,
            pinned: false // Default
        };
    };

    // A. Pinned
    const processedIds = new Set<string>();
    
    pinnedKbs.value.forEach(kb => {
        processedIds.add(kb.id);
        const item = processKb(kb);
        item.pinned = true; // [NEW] Explicitly pinned
        const key = item._renderer_id;
        if (!groups[key]) groups[key] = [];
        groups[key].push(item);
    });

    // B. Running (but not Pinned)
    runningApps.value.forEach(kbId => {
        if (!processedIds.has(kbId)) {
            const kb = allKbs.value.find(k => k.id === kbId);
            if (kb) {
                const item = processKb(kb);
                item.pinned = false; // [NEW] Explicitly unpinned
                const key = item._renderer_id;
                if (!groups[key]) groups[key] = [];
                groups[key].push(item);
            }
        }
    });
    
    const processedItems: any[] = [];

    // Process Groups
    Object.keys(groups).forEach(key => {
        const items = groups[key];
        const manifest = pluginStore.getManifest(key);
        
        if (items.length > 1) {
             const first = items[0];
             processedItems.push({
                 id: `group_${key}`,
                 dock: {
                     label: manifest ? (manifest.identity.label_strategy === 'static' ? manifest.sys_id.toUpperCase() : 'Group') : 'Stack',
                     icon: first.dock.icon,
                     order: 10
                 },
                 children: items
             });
        } else {
            processedItems.push(...items);
        }
    });

    return [libraryPlugin, ...processedItems];
});

// Resolution Logic
const currentDockItem = computed(() => {
    // Search in top level
    const top = dockItems.value.find(i => i.id === currentModuleId.value);
    if (top) return top;

    // Search in children
    for (const item of dockItems.value) {
        if ((item as any).children) {
            const child = (item as any).children.find((c: any) => c.id === currentModuleId.value);
            if (child) return child;
        }
    }
    return null;
});

const CurrentComponent = computed(() => {
    const item = currentDockItem.value as any;
    if (!item) return null;

    if (item._manifest) return item._manifest.view.component;
    if (pluginStore.getPlugin(item.id)) return pluginStore.getPlugin(item.id)?.component;
    
    // Legacy mapping (kept for safety)
    const renderer = item._renderer_id;

    if (renderer === 'vuln' || renderer === 'vrkb') return pluginStore.getPlugin('vrkb')?.component;
    
    if (renderer === 'english' || renderer === 'vocabulary' || renderer === 'english_v1') {
        return defineAsyncComponent(() => import('../components/self-space/modules/VocabularyModule.vue'));
    }
    
    if (renderer === 'memo') return pluginStore.getPlugin('memos')?.component;
    
    return pluginStore.getPlugin('knowledge')?.component;
});

const isV2Component = computed(() => {
    const item = currentDockItem.value as any;
    return !!item?._manifest;
});

// Provide Launch context to children
import { provide, defineAsyncComponent } from 'vue';
provide('os', {
    launchApp,
    closeApp
});

const currentKbId = computed(() => {
    const item = currentDockItem.value;
    if (item && (item as any)._kb_id) return (item as any)._kb_id;
    return undefined;
});

const currentModuleLabel = computed(() => currentDockItem.value?.dock.label || 'Self Space');
const currentHeaderIcon = computed(() => currentDockItem.value?.dock.icon);

const CurrentHeaderActions = computed(() => {
    // V2 Components do NOT use Shell Header Actions, they own the header.
    if (isV2Component.value) return null;

    const item = currentDockItem.value;
    if (!item) return null;

    if (pluginStore.getPlugin(item.id)) {
        return pluginStore.getPlugin(item.id)?.header?.actions;
    }

    const renderer = (item as any)._renderer_id;
    if (renderer === 'memo') return pluginStore.getPlugin('memos')?.header?.actions;
    if (renderer === 'english' || renderer === 'vocabulary') return pluginStore.getPlugin('vocabulary')?.header?.actions;
    
    return null;
});

// -- Refactored Switch --
const switchModule = (id: string) => {
    currentModuleId.value = id;
    // V2 Requirement: Deep Linking Reset
    // When switching modules, we must clear the query params to prevent state leakage
    // unless we are supporting background persistence via URL (which is weird).
    router.replace({ query: {} });
};

// -- Navigation --
const goBack = () => {
    if (window.history.state && window.history.state.back) {
        router.back();
    } else {
        router.push('/');
    }
};
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
                        v-if="currentModuleId !== 'library'"
                        @click="goHome"
                        class="w-6 h-6 rounded-md hover:bg-ink/5 text-ink/40 hover:text-ink transition-colors flex items-center justify-center"
                        title="Back to Dashboard"
                    >
                         <i class="ri-layout-grid-fill text-sm"></i>
                    </button>

                    <!-- App Title Separator -->
                    <div class="h-4 w-[1px] bg-ink/10 mx-2"></div>
                    
                    <!-- Current App Title -->
                    <div v-if="currentModuleId !== 'library'" class="text-xs font-bold font-serif text-ink select-none flex items-center gap-2">
                         <i v-if="currentHeaderIcon" :class="currentHeaderIcon" class="text-ink/60"></i>
                         {{ currentModuleLabel }}
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
                    <component :is="CurrentHeaderActions" v-if="CurrentHeaderActions" />

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

        <!-- Main Content Area -->
        <main class="flex-1 relative z-10 w-full h-full flex flex-col pt-14">
            <Transition mode="out-in" enter-active-class="transition duration-300 ease-out"
                enter-from-class="opacity-0 translate-y-4" enter-to-class="opacity-100 translate-y-0"
                leave-active-class="transition duration-200 ease-in" leave-from-class="opacity-100 translate-y-0"
                leave-to-class="opacity-0 -translate-y-4">
                <KeepAlive>
                    <component :is="CurrentComponent" :kbId="currentKbId" :key="currentModuleId" class="flex-1 h-full" :headless="true" />
                </KeepAlive>
            </Transition>
        </main>

        <!-- Dock Navigation -->
        <ModuleSwitcher :active-module="currentModuleId" :modules="dockItems" @switch="switchModule" @close="closeApp" />
    </div>
</template>
