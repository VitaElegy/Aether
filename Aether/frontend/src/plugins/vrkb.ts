import { defineAsyncComponent } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const VrkbPlugin: SelfSpacePlugin = {
    id: 'vrkb',
    dock: {
        label: 'Vulnerability Research',
        icon: 'ri-shield-keyhole-line',
        order: 50
    },
    header: {
        title: 'Vulnerability Research',
        icon: 'ri-shield-keyhole-line'
    },
    capabilities: {
        articleParser: false
    },
    component: defineAsyncComponent(() => import('../components/self-space/modules/vrkb/VrkbModule.vue'))
};
