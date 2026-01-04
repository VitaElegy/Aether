import type { SelfSpacePlugin } from '../../core/plugin';
import MemosModule from '../../components/self-space/modules/MemosModule.vue';

export const MemosPlugin: SelfSpacePlugin = {
    id: 'memos',
    label: 'Memos',
    icon: 'ri-sticky-note-line',
    component: MemosModule,
    order: 30,
};
