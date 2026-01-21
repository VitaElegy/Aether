import type { SelfSpacePlugin } from '../../core/plugin';
import KnowledgeModule from '../../components/self-space/modules/KnowledgeModule.vue';

export const KnowledgePlugin: SelfSpacePlugin = {
    id: 'knowledge',
    dock: {
        label: 'Knowledge',
        icon: 'ri-brain-line',
        order: 20
    },
    header: {
        title: 'Knowledge Base',
        icon: 'ri-brain-line'
    },
    capabilities: {
        articleParser: false
    },
    component: KnowledgeModule,
};
