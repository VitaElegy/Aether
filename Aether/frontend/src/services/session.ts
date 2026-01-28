import localforage from 'localforage';

// Initialize localforage instance for Aether Session State
const sessionStore = localforage.createInstance({
    name: "aether_session",
    storeName: "tab_states" // Key-Value store
});

export interface RouteState {
    scrollX: number;
    scrollY: number;
    formData?: Record<string, any>;
    meta?: Record<string, any>; // Extra metadata (cursor position, etc.)
    timestamp: number;
}

export class SessionService {
    /**
     * Generates a unique persistence key for a specific tab and route.
     */
    private getKey(tabId: string, fullPath: string): string {
        return `session:${tabId}:${fullPath}`;
    }

    /**
     * Save state for a route
     */
    async saveState(tabId: string, fullPath: string, state: Partial<RouteState>): Promise<void> {
        const key = this.getKey(tabId, fullPath);
        try {
            await sessionStore.setItem(key, {
                ...state,
                timestamp: Date.now()
            });
            // console.log(`[Session] Saved state for ${key}`);
        } catch (e) {
            console.warn(`[Session] Failed to save state for ${key}`, e);
        }
    }

    /**
     * Restore state for a route
     */
    async restoreState(tabId: string, fullPath: string): Promise<RouteState | null> {
        const key = this.getKey(tabId, fullPath);
        try {
            const state = await sessionStore.getItem<RouteState>(key);
            if (state) {
                // Optional: Expiration logic (e.g., clear after 7 days)
                if (Date.now() - state.timestamp > 7 * 24 * 60 * 60 * 1000) {
                    await sessionStore.removeItem(key);
                    return null;
                }
                // console.log(`[Session] Restored state for ${key}`);
            }
            return state;
        } catch (e) {
            console.warn(`[Session] Failed to restore state for ${key}`, e);
            return null;
        }
    }

    /**
     * Clear all state for a tab (e.g. when closing it)
     */
    async clearTab(tabId: string): Promise<void> {
        // Localforage doesn't support "delete by prefix" efficiently.
        // In a real implementation, we might maintain a set of keys per tab.
        // For now, we assume states are evicted by LRU or expiration, OR we iterate keys.
        // Optimization for V2.1
        console.log(`[Session] Clear tab request for ${tabId} (Not fully implemented)`);
    }
}

export const sessionService = new SessionService();
