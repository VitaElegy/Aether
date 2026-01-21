import type { SelfSpacePlugin } from '../../core/plugin';
import MemosModule from '../../components/self-space/modules/MemosModule.vue';

export const MemosPlugin: SelfSpacePlugin = {
    id: 'memos',
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
    component: MemosModule,
};
