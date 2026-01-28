import type { SkbPluginManifest } from '@/types/plugin_manifest';
import { defineAsyncComponent } from 'vue';

export const VrkbManifest: SkbPluginManifest = {
    sys_id: 'vrkb',

    identity: {
        icon: 'ri-shield-keyhole-line',
        label_strategy: 'static',
        color_theme: 'text-red-500' // Example
    },

    view: {
        component: defineAsyncComponent(() => import('./VrkbModule.vue')),
        default_route_params: { view: 'lifecycle' }
    },

    settings: [
        {
            key: 'scan_interval',
            type: 'select',
            label: 'Auto-Scan Interval',
            default_value: 'daily',
            options: [
                { label: 'Daily', value: 'daily' },
                { label: 'Weekly', value: 'weekly' },
                { label: 'Manual Only', value: 'manual' }
            ]
        },
        {
            key: 'severity_threshold',
            type: 'select',
            label: 'Severity Threshold',
            default_value: 'medium',
            options: [
                { label: 'Low+', value: 'low' },
                { label: 'Medium+', value: 'medium' },
                { label: 'High+', value: 'high' }
            ]
        }
    ],

    capabilities: {
        has_global_search: true,
        has_export: true,
        supports_auto_grouping: true
    }
};
