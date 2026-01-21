import type { SelfSpacePlugin } from '../../core/plugin';
import VocabularyModule from '../../components/self-space/modules/VocabularyModule.vue';

export const VocabularyPlugin: SelfSpacePlugin = {
    id: 'vocabulary',
    dock: {
        label: 'Vocabulary',
        icon: 'ri-book-read-line',
        order: 40
    },
    header: {
        title: 'Vocabulary',
        icon: 'ri-book-read-line'
    },
    capabilities: {
        articleParser: false
    },
    component: VocabularyModule,
};
