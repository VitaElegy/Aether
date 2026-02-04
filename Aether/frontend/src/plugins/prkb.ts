import { defineAsyncComponent } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const PrkbPlugin: SelfSpacePlugin = {
    id: 'prkb',
    dock: {
        label: 'Paper Research',
        icon: 'ri-article-line',
        order: 60
    },
    header: {
        title: 'Paper Research',
        icon: 'ri-article-line'
    },
    capabilities: {
        articleParser: false
    },
    component: defineAsyncComponent(() => import('@/views/prkb/ResearchSpace.vue'))
};
