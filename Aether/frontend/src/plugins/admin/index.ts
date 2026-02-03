import type { SelfSpacePlugin } from '../../core/plugin';
import { defineAsyncComponent } from 'vue';

export const AdminPlugin: SelfSpacePlugin = {
    id: 'admin_system', // Exact match for DB renderer_id
    dock: {
        label: 'Admin',
        icon: 'ri-shield-star-line',
        order: 99 // System items often last or very first (pinned handles position)
    },
    header: {
        title: 'System Administrator',
        icon: 'ri-shield-star-line'
    },
    capabilities: {},
    component: defineAsyncComponent(() => import('../../views/dashboard/AdminDashboard.vue')),
};
