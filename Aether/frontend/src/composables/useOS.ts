
import { inject, type InjectionKey } from 'vue';

// Define the Shape of OS Services
export interface OSContext {
    // Navigation
    launchApp: (id: string) => void;
    closeApp: (id: string) => void;

    // Feedback
    toast: (content: string, theme?: 'success' | 'warning' | 'error') => void;

    // Future expansion:
    // modal: { open: ... }
    // clipboard: { write: ... }
}

// Symbol for Injection
export const osKey: InjectionKey<OSContext> = Symbol('os');

// Helper hook for plugins to use
export function useOS() {
    const os = inject(osKey);
    if (!os) {
        throw new Error('OS Context Not Found! Plugins must be used within Self Space.');
    }
    return os;
}
