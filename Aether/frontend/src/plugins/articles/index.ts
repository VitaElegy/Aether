import type { SelfSpacePlugin } from '../../core/plugin';
import ArticlesModule from '../../components/self-space/modules/ArticlesModule.vue';

export const ArticlesPlugin: SelfSpacePlugin = {
    id: 'articles',
    dock: {
        label: 'Articles',
        icon: 'ri-article-line',
        order: 10
    },
    header: {
        title: 'Articles',
        icon: 'ri-article-line'
    },
    capabilities: {
        articleParser: true
    },
    component: ArticlesModule,
};
