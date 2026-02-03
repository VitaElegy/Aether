/**
 * Core Orchestrator for Self Space.
 * 
 * DESIGN: EXPLICIT DOCK QUEUE + SINGLETON RESOLUTION + REGISTRY DRIVEN
 * 
 * Responsibilities:
 * 1. KB Switching (with plugin loading validation)
 * 2. Dock Items Management (Managed Queue)
 * 3. component resolution
 * 4. Loading state management
 */

import { ref, computed, type Component, defineAsyncComponent, watch } from 'vue';
import { useAppStateStore } from '../stores/read_app_state';
import { usePluginStore } from '../stores/plugins';
import { usePreferencesStore } from '../stores/preferences';
import { knowledgeApi, type KnowledgeBase } from '../api/knowledge';
import { eventBus } from '../utils/eventBus';

// Types
export interface DockItem {
    id: string;
    dock: {
        label: string;
        icon: string;
        order: number;
    };
    _renderer_id: string;
    pinned: boolean; // Is it in the Pinned Section?
    isRunning: boolean; // Is it currently open/backgrounded?
    children?: DockItem[];
}

// Special Renderers that should only appear ONCE in the dock
const SINGLETON_RENDERERS = new Set(['math', 'system', 'admin_system', 'admin', 'vocabulary', 'memo', 'vrkb']);

export function useSelfSpaceOrchestrator() {
    const appStore = useAppStateStore();
    const pluginStore = usePluginStore();
    const prefStore = usePreferencesStore();

    // Internal State
    const allKbs = ref<KnowledgeBase[]>([]);
    const isLoading = ref(false);
    const loadingKbId = ref<string | null>(null);
    const lastError = ref<Error | null>(null);
    const isInitialized = ref(false);

    // ============================================================
    // THE DOCK QUEUE (Explicit State)
    // ============================================================
    const dockQueue = ref<DockItem[]>([]);

    // ============================================================
    // CORE METHODS
    // ============================================================

    async function initialize(): Promise<void> {
        if (isInitialized.value) return;

        isLoading.value = true;
        try {
            await refreshKbList();
            syncDock();
            isInitialized.value = true;
        } catch (e) {
            console.error('[Orchestrator] Init failed:', e);
            lastError.value = e as Error;
        } finally {
            isLoading.value = false;
        }
    }

    async function refreshKbList(): Promise<void> {
        try {
            const kbs = await knowledgeApi.list();
            allKbs.value = kbs;
            appStore.registerKbs(kbs);
        } catch (e) {
            console.error('[Orchestrator] Failed to fetch KB list:', e);
            throw e;
        }
    }

    function createDockItem(kb: KnowledgeBase): DockItem {
        const renderer = kb.renderer_id || 'default';
        const plugin = pluginStore.resolvePlugin(renderer);

        // PURE REGISTRY LOGIC: Ask the plugin for the icon.
        // If the plugin is not found, resolvePlugin returns fallback, or we use default.
        // Visual Error if Plugin Missing
        const icon = plugin?.dock?.icon || (plugin ? 'ri-book-2-line' : 'ri-error-warning-fill');
        const label = kb.title; // Or plugin?.dock?.label if we wanted to override title

        return {
            id: kb.id,
            dock: {
                label,
                icon,
                order: plugin?.dock?.order || 10
            },
            _renderer_id: renderer,
            pinned: prefStore.isPinned(kb.id),
            isRunning: appStore.runningKbIds.has(kb.id)
        };
    }

    /**
     * CORE ACTION: Sync Dock
     * Reconciles Pinned Preferences + Running Apps into the Queue.
     * Uses strict Singleton Deduplication.
     */
    function syncDock() {
        // Queue builder
        const nextQueue = new Map<string, DockItem>();
        const processedRenderers = new Set<string>();

        // internal helper to add safely
        const attemptAdd = (item: DockItem) => {
            if (nextQueue.has(item.id)) return; // Already by ID

            // Singleton Check
            if (SINGLETON_RENDERERS.has(item._renderer_id)) {
                if (processedRenderers.has(item._renderer_id)) {
                    // Update existing if needed
                    for (const [key, existing] of nextQueue.entries()) {
                        if (existing._renderer_id === item._renderer_id) {
                            if (item.isRunning) existing.isRunning = true;
                            if (item.pinned) existing.pinned = true;
                            return;
                        }
                    }
                }
                processedRenderers.add(item._renderer_id);
            }

            nextQueue.set(item.id, item);
        };

        // 1. Library (Always First - The Anchor)
        const libraryPlugin = pluginStore.resolvePlugin('knowledge');
        attemptAdd({
            id: 'library',
            dock: {
                label: 'Library',
                icon: libraryPlugin?.dock.icon || 'ri-apps-2-line',
                order: 0
            },
            _renderer_id: 'knowledge',
            pinned: true,
            isRunning: true
        });

        // 2. Add Pinned Items (Order matters)
        for (const pid of prefStore.pinnedKbIds) {
            const kb = allKbs.value.find(k => k.id === pid);
            if (kb) {
                attemptAdd(createDockItem(kb));
            } else {
                // Legacy ID support
                const altKb = allKbs.value.find(k => k.renderer_id === pid);
                if (altKb) {
                    const item = createDockItem(altKb);
                    item.pinned = true;
                    attemptAdd(item);
                }
            }
        }

        // 3. System KB (Explicitly Pinned via Registry Match)
        // We scan for KBs that resolve to 'admin_system' which we know is the critical system app.
        // This is the only "hardcoded" assumption: that there is a 'admin_system' renderer that implies pinned status.
        const systemKb = allKbs.value.find(k => k.renderer_id === 'admin_system');
        if (systemKb) {
            const item = createDockItem(systemKb);
            item.pinned = true;
            attemptAdd(item);
        }

        // 4. Running Apps
        // A. From Existing Queue (Keep state)
        for (const existing of dockQueue.value) {
            if (existing.isRunning) {
                attemptAdd(existing);
            }
        }
        // B. From AppStore Running List
        for (const rid of appStore.runningKbIds) {
            const kb = allKbs.value.find(k => k.id === rid);
            if (kb) {
                const item = createDockItem(kb);
                item.isRunning = true;
                attemptAdd(item);
            }
        }

        dockQueue.value = Array.from(nextQueue.values());
    }

    async function switchToKb(kbId: string): Promise<void> {
        if (isLoading.value) return;
        if (kbId === 'library') {
            appStore.switchKB('library');
            return;
        }
        if (kbId === appStore.activeKbId) return;

        await requestOpen(kbId);
    }

    async function requestOpen(kbId: string) {
        isLoading.value = true;
        try {
            // Smart ID Resolution
            // [Fix] Clear any previous error state so we can retry!
            errorState.value.delete(kbId);

            let targetKb = allKbs.value.find(k => k.id === kbId);

            if (!targetKb) {
                await refreshKbList();
                targetKb = allKbs.value.find(k => k.id === kbId);
            }
            if (!targetKb) {
                targetKb = allKbs.value.find(k => k.renderer_id === kbId);
                if (targetKb) {
                    console.log(`[Orchestrator] Resolved legacy ID '${kbId}' to UUID '${targetKb.id}'`);
                    kbId = targetKb.id;
                }
            }

            if (!targetKb) throw new Error(`KB Not Found: ${kbId}`);

            // 3. Activate
            const renderer = targetKb.renderer_id || 'knowledge';
            pluginStore.resolvePlugin(renderer);
            appStore.switchKB(targetKb.id);

            // 4. Sync Dock
            syncDock();

        } catch (e) {
            console.error('[Orchestrator] Open Failed:', e);
        } finally {
            isLoading.value = false;
        }
    }

    function closeKb(kbId: string): void {
        appStore.closeKB(kbId);

        // Optimistic update
        const item = dockQueue.value.find(i => i.id === kbId);
        if (item) {
            item.isRunning = false;
            if (!item.pinned) {
                dockQueue.value = dockQueue.value.filter(i => i.id !== kbId);
            }
        }
        syncDock();
    }

    // Error State Management
    const errorState = ref<Map<string, { message: string, stack?: string }>>(new Map());

    // ... (existing code)

    function crashKb(kbId: string, error?: Error) {
        // Log the crash
        console.error(`[Orchestrator] Crashing KB: ${kbId}`, error);

        // Store error details
        errorState.value.set(kbId, {
            message: error?.message || 'Unknown Application Error',
            stack: error?.stack
        });

        // DO NOT CLOSE immediately, let the UI show the BrokenState
        // appStore.crashApp(kbId); // Optional: Mark as crashed in AppStore if needed
    }

    // Components
    const currentDockItem = computed((): DockItem | null => {
        return dockQueue.value.find(i => i.id === appStore.activeKbId) || null;
    });

    const currentComponent = computed((): Component => {
        // 1. Check for Forced Error State
        const activeId = appStore.activeKbId;
        if (errorState.value.has(activeId)) {
            const err = errorState.value.get(activeId)!;
            // Return BrokenState with injected props (via provide or props shifting?)
            // Since we can't easily pass props to a dynamic component definition here without a wrapper,
            // we will rely on the View Layer to read the error from Orchestrator or
            // we define a wrapping component on the fly (less performant).
            // A better approach: The View reads `orchestrator.errorState` and decides what to render.
            // BUT, to keep strict encapsulation, let's return BrokenState.
            return defineAsyncComponent(() => import('../components/self-space/BrokenState.vue'));
        }

        const item = currentDockItem.value;
        const renderer = item ? item._renderer_id : 'knowledge';

        if (activeId === 'library') {
            const lib = pluginStore.resolvePlugin('knowledge');
            return lib?.component || defineAsyncComponent(() => import('../components/self-space/BrokenState.vue'));
        }

        const resolved = pluginStore.resolvePlugin(renderer);
        return resolved?.component || defineAsyncComponent(() => import('../components/self-space/BrokenState.vue'));
    });

    return {
        isLoading,
        loadingKbId,
        lastError,
        isInitialized,
        allKbs,
        errorState, // Exported for View to read details

        initialize,
        refreshKbList,
        switchToKb,
        closeKb,
        crashKb,

        dockItems: computed(() => dockQueue.value),
        pinnedDockItems: computed(() => dockQueue.value.filter(i => i.pinned)),
        runningDockItems: computed(() => dockQueue.value.filter(i => !i.pinned)),

        currentDockItem,
        currentComponent
    };
}
