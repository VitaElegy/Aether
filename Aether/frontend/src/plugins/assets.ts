import { defineAsyncComponent } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const AssetsPlugin: SelfSpacePlugin = {
    id: 'assets_v1',
    dock: {
        label: 'My Assets',
        icon: 'ri-folder-3-line',
        order: 60
    },
    header: {
        title: 'My Assets',
        icon: 'ri-folder-3-line'
    },
    capabilities: {
        articleParser: false
    },
    component: defineAsyncComponent(() => import('../views/apps/MyAssets.vue'))
};
