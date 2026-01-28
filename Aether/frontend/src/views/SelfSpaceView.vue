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
const isLoadingDock = ref(false);

const updateDock = async () => {
    // We fetch all KBs and filter by pinned IDs
    // Optimization: In real app, we should have a 'get_many' endpoint or just list.
    // List is lightweight enough.
    isLoadingDock.value = true;
    try {
        const all = await knowledgeApi.list();
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

// Compute Dock Items
const dockItems = computed(() => {
    // 1. The "Library" (All Apps) - Fixed
    const libraryPlugin = pluginStore.getPlugin('knowledge') || {
        id: 'library',
        dock: { label: 'Library', icon: 'ri-apps-2-line', order: 0 },
        component: {} as any,
        capabilities: []
    } as any; // Cast to avoid strict type check for now
    
    // 2. Pinned Items Grouping
    const groups: Record<string, any[]> = {};
    const singles: any[] = [];
    
    // First, map all to standardized objects
    pinnedKbs.value.forEach(kb => {
        let icon = 'ri-book-2-line';
        let typeLabel = 'Knowedge';
        let groupId = kb.renderer_id || 'default';

        if (kb.renderer_id === 'memo') { icon = 'ri-sticky-note-line'; typeLabel = 'Memos'; }
        if (kb.renderer_id === 'english') { icon = 'ri-translate-2'; typeLabel = 'English'; }
        if (kb.renderer_id && kb.renderer_id.startsWith('math')) { icon = 'ri-functions'; typeLabel = 'Math'; }

        const dockItem = {
            id: kb.id,
            dock: {
                label: kb.title,
                icon: icon,
                order: 10
            },
            _renderer_id: kb.renderer_id || 'default', 
            _kb_id: kb.id,
            component: {} as any,
            capabilities: []
        };

        if (!groups[groupId]) groups[groupId] = [];
        groups[groupId].push(dockItem);
    });

    const processedItems: any[] = [];

    // Process Groups
    Object.keys(groups).forEach(key => {
        const items = groups[key];
        if (items.length > 1) {
             // Create a Group Item if > 2 items of same type
             // (Constraint: User said "multiple instances... group multiple instances")
             // Let's use > 1 to be aggressive with grouping 
             const first = items[0];
             processedItems.push({
                 id: `group_${key}`,
                 dock: {
                     label: key.charAt(0).toUpperCase() + key.slice(1) + 's', // e.g. Memos
                     icon: first.dock.icon, // Use same icon
                     order: 10
                 },
                 children: items
             });
        } else {
            // Add as singles
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
    const item = currentDockItem.value;
    if (!item) return null;

    // A. It's a real plugin (Library)
    if (pluginStore.getPlugin(item.id)) {
        return pluginStore.getPlugin(item.id)?.component;
    }

    // B. It's a Pinned KB
    // Resolve Component based on renderer_id
    // We look up the 'base plugin' for that renderer
    const renderer = (item as any)._renderer_id;
    
    if (renderer === 'memo') return pluginStore.getPlugin('memos')?.component;
    // For English/Vocab, we map to 'vocabulary' plugin
    if (renderer === 'english' || renderer === 'vocabulary') return pluginStore.getPlugin('vocabulary')?.component;
    
    // Default fallback is KnowledgeModule (Library) but in Detail Mode
    return pluginStore.getPlugin('knowledge')?.component;
});

const currentKbId = computed(() => {
    const item = currentDockItem.value;
    if (item && (item as any)._kb_id) return (item as any)._kb_id;
    return undefined;
});

const currentModuleLabel = computed(() => currentDockItem.value?.dock.label || 'Self Space');
const currentHeaderIcon = computed(() => currentDockItem.value?.dock.icon);

const CurrentHeaderActions = computed(() => {
    const item = currentDockItem.value;
    if (!item) return null;

    // A. It's a real plugin (Library)
    if (pluginStore.getPlugin(item.id)) {
        return pluginStore.getPlugin(item.id)?.header?.actions;
    }

    // B. It's a Pinned KB - Loopup Base
    const renderer = (item as any)._renderer_id;
    if (renderer === 'memo') return pluginStore.getPlugin('memos')?.header?.actions;
    if (renderer === 'english' || renderer === 'vocabulary') return pluginStore.getPlugin('vocabulary')?.header?.actions;
    
    return null;
});

// -- Refactored Switch --
const switchModule = (id: string) => {
    currentModuleId.value = id;
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
                <!-- handled by global beacon -->
            </template>

            <template #center>
                <!-- DYNAMIC CENTER PORTAL -->
                <div id="nav-center-portal" class="contents"></div>

                <!-- Default Center Content -->
                <div v-show="!navStore.hasCustomCenter" class="flex items-center gap-3">
                    <i v-if="currentHeaderIcon" :class="[currentHeaderIcon, 'text-ink/40 text-lg']"></i>
                    <span class="text-[10px] font-black uppercase tracking-[0.3em] text-ink/40">
                        Self Space / {{ currentModuleLabel }}
                    </span>
                </div>
            </template>

            <template #right>
                <!-- DYNAMIC RIGHT PORTAL -->
                <div id="nav-right-portal" class="contents"></div>

                <!-- Default Right Content -->
                <div v-show="!navStore.hasCustomRight" class="flex items-center gap-2">
                    <!-- Plugin Default Actions -->
                    <component :is="CurrentHeaderActions" v-if="CurrentHeaderActions" />

                     <!-- Default Global Actions (e.g. Settings, Profile) -->
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
        <main class="flex-1 relative z-10 w-full h-full flex flex-col pt-16">
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
        <ModuleSwitcher :active-module="currentModuleId" :modules="dockItems" @switch="switchModule" />
    </div>
</template>
