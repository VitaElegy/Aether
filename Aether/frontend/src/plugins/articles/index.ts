import type { SelfSpacePlugin } from '../../core/plugin';
import ArticlesModule from '../../components/self-space/modules/ArticlesModule.vue';

export const ArticlesPlugin: SelfSpacePlugin = {
    id: 'articles',
    label: 'Articles',
    icon: 'ri-article-line',
    component: ArticlesModule,
    order: 10,
};
