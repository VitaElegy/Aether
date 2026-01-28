import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { sessionService } from '../services/session';

/**
 * Navigation Stack Store
 * Implements the "Multi-Stack" pattern where each Module (tab) retains its own history.
 * Persists to IndexedDB via SessionService.
 */
export const useNavigationStackStore = defineStore('navigation-stack', () => {
    // Map<ModuleID, LastRoute>
    const stacks = ref<Record<string, string>>({});
    const isLoaded = ref(false);

    // Initialize: Load from persistence
    async function init() {
        if (isLoaded.value) return;

        try {
            const saved = await sessionService.restoreState('global', 'stack_history');
            if (saved && saved.formData) {
                stacks.value = saved.formData;
            }
        } catch (e) {
            console.error("Failed to load navigation stack", e);
        } finally {
            isLoaded.value = true;
        }
    }

    /**
     * Save the current route for a specific module
     */
    function saveRoute(moduleId: string, fullPath: string) {
        stacks.value[moduleId] = fullPath;
        // Persistence handled by watcher
    }

    /**
     * Get the last saved route for a module.
     * Returns undefined if never visited.
     */
    function getLastRoute(moduleId: string): string | undefined {
        return stacks.value[moduleId];
    }

    /**
     * Clear stack for a module (e.g. on double click to reset)
     */
    function clearStack(moduleId: string) {
        delete stacks.value[moduleId];
    }

    // Persist changes
    watch(stacks, (newVal) => {
        sessionService.saveState('global', 'stack_history', {
            scrollX: 0,
            scrollY: 0,
            timestamp: Date.now(),
            formData: newVal // Storing the map as formData
        });
    }, { deep: true });

    return {
        stacks,
        isLoaded,
        init,
        saveRoute,
        getLastRoute,
        clearStack
    };
});
