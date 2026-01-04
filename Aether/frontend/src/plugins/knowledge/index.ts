import type { SelfSpacePlugin } from '../../core/plugin';
import KnowledgeModule from '../../components/self-space/modules/KnowledgeModule.vue';

export const KnowledgePlugin: SelfSpacePlugin = {
    id: 'knowledge',
    label: 'Knowledge',
    icon: 'ri-brain-line',
    component: KnowledgeModule,
    order: 20,
};
