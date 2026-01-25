import { defineAsyncComponent } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const MemosPlugin: SelfSpacePlugin = {
    id: 'memos',
    dock: {
        label: 'Memos',
        icon: 'ri-sticky-note-line',
        order: 30
    },
    header: {
        title: 'Memos & Quick Capture',
        icon: 'ri-sticky-note-line',
        actions: defineAsyncComponent(() => import('../components/self-space/modules/memos/MemosHeaderActions.vue'))
    },
    capabilities: {
        articleParser: false
    },
    component: defineAsyncComponent(() => import('../components/self-space/modules/memos/MemosModule.vue'))
};
