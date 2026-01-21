import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { RouteLocationNormalized } from 'vue-router';

/**
 * Navigation Stack Store
 * Implements the "Multi-Stack" pattern where each Module (tab) retains its own history.
 */
export const useNavigationStackStore = defineStore('navigation-stack', () => {
    // Map<ModuleID, LastRoute>
    const stacks = ref<Record<string, string>>({});

    /**
     * Save the current route for a specific module
     */
    function saveRoute(moduleId: string, fullPath: string) {
        stacks.value[moduleId] = fullPath;
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

    return {
        stacks,
        saveRoute,
        getLastRoute,
        clearStack
    };
});
