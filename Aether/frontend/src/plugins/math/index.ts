import type { SelfSpacePlugin } from '../../core/plugin';
import MathDashboard from '../../components/dashboard/MathDashboard.vue';

export const MathPlugin: SelfSpacePlugin = {
    id: 'math',
    dock: {
        label: 'Math',
        icon: 'ri-function-line',
        order: 30
    },
    header: {
        title: 'Math Knowledge Base',
        icon: 'ri-function-line'
    },
    capabilities: {
        articleParser: true
    },
    component: MathDashboard,
};
