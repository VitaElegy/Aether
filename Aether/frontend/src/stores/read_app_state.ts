import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useRouter } from 'vue-router';

export const useAppStateStore = defineStore('app_state', () => {
    const router = useRouter();

    // 1. State
    // The currently focused KB/Module ID
    const activeKbId = ref<string>('library');

    // The set of "Open" applications (Running in background/KeepAlive)
    const runningKbIds = ref<Set<string>>(new Set());

    // [NEW] Smart Registry (Internal Cache for Validation/Normalization)
    // We keep a lightweight map to resolve aliases like 'system' -> UUID
    const kbRegistry = ref<any[]>([]);

    // 2. Actions

    /**
     * The Single Source of Truth for switching KBs.
     * @param id - The KB ID (or 'library') to switch to.
     */
    function switchKB(rawId: string) {
        // 1. Normalize ID (The Gatekeeper)
        const id = _resolveId(rawId);
        console.log(`[Store] SwitchKB: ${rawId} -> ${id}`);

        // A. Update Active State
        activeKbId.value = id;

        // B. Add to Running List if it's not the Library
        if (id !== 'library') {
            // Force reactivity by creating a new Set
            const newSet = new Set(runningKbIds.value);
            newSet.add(id);
            runningKbIds.value = newSet;
        }

        // C. Sync with Router (Side Effect)
        // We replace the query to clean up, but we might want to push a new route in the future
        // For now, retaining the current behavior of 'query clearing'
        // Ideally, this should update the URL path /kb/:id, but SelfSpace uses query parameter logic or internal state.
        // Based on SelfSpaceView, it seems to rely on internal state + clearing query.
    }

    /**
     * Close an app explicitly.
     * @param id - The KB ID to close.
     */
    function closeKB(id: string) {
        const newSet = new Set(runningKbIds.value);
        newSet.delete(id);
        runningKbIds.value = newSet;

        // If closing the active one, go home
        if (activeKbId.value === id) {
            switchKB('library');
        }
    }

    /**
     * "Crash to Desktop" action.
     * Used by Error Boundaries to force-quit a broken app.
     */
    function crashApp(id: string) {
        console.warn(`[AppState] Crashing App: ${id}`);
        closeKB(id);
        // We could also add a 'crashedApps' list here to prevent auto-reopening
    }

    /**
     * Register KBs for normalization.
     * Called by Shell/Dock when data is loaded.
     */
    function registerKbs(kbs: any[]) {
        kbRegistry.value = kbs;
    }

    /**
     * Helper to resolve aliases.
     */
    function _resolveId(id: string): string {
        if (id === 'library') return 'library';

        // Alias Normalization
        if (id === 'system' || id === 'admin') {
            const sys = kbRegistry.value.find(k => k.renderer_id === 'admin' || k.renderer_id === 'system' || k.renderer_id === 'admin_system');
            if (sys) return sys.id;
        }

        // Direct ID check
        // If we have the ID in registry, great.
        // If not, we still allow it (maybe it's a new one not yet synced), but we prefer real IDs.
        return id;
    }

    return {
        activeKbId,
        runningKbIds,
        switchKB,
        closeKB,
        crashApp,
        registerKbs
    };
});
