import type { DefineComponent } from 'vue';

export interface SkbPluginManifest {
    sys_id: string;          // Unique Renderer ID (e.g., 'vrkb', 'math_v3')

    // 1. Identity (Dock & Library)
    identity: {
        icon: string;          // RemixIcon class (e.g., 'ri-shield-line')
        label_strategy: 'static' | 'dynamic'; // Does it use the specific KB title?
        color_theme: string;   // Tailwind class for branding
    };

    // 2. View Injection (The Component)
    view: {
        component: any; // DefineComponent (using any to avoid complex type issues for now)
        default_route_params?: Record<string, any>; // Reset state params
    };

    // 3. Settings Schema (System-Rendered)
    // AI MUST define settings here, NOT in a custom component.
    settings: {
        key: string;
        type: 'toggle' | 'text' | 'select' | 'range';
        label: string;
        default_value: any;
        options?: { label: string; value: any }[];
    }[];

    // 4. Capabilities (Feature Flags)
    capabilities: {
        has_global_search: boolean;
        has_export: boolean;
        supports_auto_grouping: boolean; // MUST be true for Templates
    };
}
