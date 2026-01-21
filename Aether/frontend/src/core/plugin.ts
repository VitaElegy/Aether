import type { Component } from 'vue';

export interface SelfSpacePlugin {
    id: string; // Unique ID (e.g., 'vrkb', 'math')

    // Bottom Dock Registration
    dock: {
        label: string;
        icon: string; // Remix Icon class name (e.g., 'ri-book-line')
        order?: number;
    };

    // Top Navigation Registration (Optional defaults)
    // If provided, SelfSpaceView can render these defaults if no portal content is active.
    header?: {
        title?: string;
        icon?: string;
        // Actions component to render in the right portal by default
        actions?: Component;
    };

    // Capability Flags
    capabilities: {
        // parsing engine capable of handling standard Aether article format
        articleParser?: boolean;
        // Does the plugin support the Reactive Context Protocol?
        reactiveContext?: boolean;
    };

    // The Main View Component
    component: Component;
}
