import { defineAsyncComponent } from 'vue';
import type { SelfSpacePlugin } from '../../core/plugin';

export const MemosPlugin: SelfSpacePlugin = {
    id: 'memo',
    dock: {
        label: 'Memos',
        icon: 'ri-sticky-note-line',
        order: 30
    },
    header: {
        title: 'Memos',
        icon: 'ri-sticky-note-line'
    },
    capabilities: {
        articleParser: false
    },
    component: defineAsyncComponent(() => import('../../components/self-space/modules/memos/MemosModule.vue')),
};
