import type { SelfSpacePlugin } from '../../core/plugin';
import VocabularyModule from '../../components/self-space/modules/VocabularyModule.vue';

export const VocabularyPlugin: SelfSpacePlugin = {
    id: 'vocabulary',
    label: 'Vocabulary',
    icon: 'ri-book-read-line',
    component: VocabularyModule,
    order: 40,
};
