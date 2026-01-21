import type { SelfSpacePlugin } from '../../core/plugin';
import ArticleAnalysisModule from '../../components/self-space/modules/ArticleAnalysisModule.vue';

export const ArticleAnalysisPlugin: SelfSpacePlugin = {
    id: 'article-analysis',
    dock: {
        label: 'Article Analysis',
        icon: 'ri-article-line',
        order: 45
    },
    header: {
        title: 'Article Analysis',
        icon: 'ri-article-line'
    },
    capabilities: {
        articleParser: true
    },
    component: ArticleAnalysisModule,
};
