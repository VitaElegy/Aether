import { defineStore } from 'pinia';
import { ref, watch, computed } from 'vue';
import { sessionService } from '../services/session';

/**
 * Navigation Stack Store (Smart Trace V1)
 * Implements "Scoped History Stacks" for each module to decouple app navigation from browser history.
 * 
 * Logic:
 * - Each Module (e.g., 'math_kb', 'library') has its own stack of visited routes.
 * - 'Push' adds to the current module's stack.
 * - 'Pop' removes the top and returns the previous route (for Smart Back).
 * - Persisted via IndexedDB.
 */
export const useNavigationStackStore = defineStore('navigation-stack', () => {
    // Map<ModuleID, HistoryStack[]>
    const stacks = ref<Record<string, string[]>>({});
    const activeModule = ref<string>('library'); // Default, should be updated by Router
    const isLoaded = ref(false);

    // Initialize: Load from persistence
    async function init() {
        if (isLoaded.value) return;

        try {
            const saved = await sessionService.restoreState('global', 'stack_history_v1');
            if (saved && saved.formData) {
                // Migration check: If old format (string), reset or migrate
                const data = saved.formData;
                if (Object.values(data).some(v => typeof v === 'string')) {
                    console.warn("Detected legacy stack format. Resetting.");
                    stacks.value = {};
                } else {
                    stacks.value = data;
                }
            }
        } catch (e) {
            console.error("Failed to load navigation stack", e);
        } finally {
            isLoaded.value = true;
        }
    }

    /**
     * Push a route onto the active module's stack.
     * Prevents duplicate adjacent entries.
     */
    function pushRoute(moduleId: string, fullPath: string) {
        if (!stacks.value[moduleId]) {
            stacks.value[moduleId] = [];
        }

        const stack = stacks.value[moduleId];
        const last = stack[stack.length - 1];

        // Dedup: Don't push if same as current top
        if (last !== fullPath) {
            stack.push(fullPath);
        }
    }

    /**
     * Pop the current route and return the *previous* one.
     * Used by Global Beacon for "Smart Back".
     */
    function popRoute(moduleId: string): string | undefined {
        const stack = stacks.value[moduleId];
        if (!stack || stack.length <= 1) return undefined;

        stack.pop(); // Remove current
        return stack[stack.length - 1]; // Return new top
    }

    /**
     * Get the current (top) route for a module.
     * Used for "Resume" functionality when switching tabs.
     */
    function getLastRoute(moduleId: string): string | undefined {
        const stack = stacks.value[moduleId];
        if (!stack || stack.length === 0) return undefined;
        return stack[stack.length - 1];
    }

    /**
     * Clear stack for a module (Reset to root)
     */
    function clearStack(moduleId: string) {
        stacks.value[moduleId] = [];
    }

    function setActiveModule(moduleId: string) {
        activeModule.value = moduleId;
    }

    // Persist changes
    watch(stacks, (newVal) => {
        sessionService.saveState('global', 'stack_history_v1', {
            scrollX: 0,
            scrollY: 0,
            timestamp: Date.now(),
            formData: newVal
        });
    }, { deep: true });

    return {
        stacks,
        activeModule,
        isLoaded,
        init,
        pushRoute,
        popRoute,
        getLastRoute,
        clearStack,
        setActiveModule
    };
});
