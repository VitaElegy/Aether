import type { SelfSpacePlugin } from '../../core/plugin';
import ArticleAnalysisModule from '../../components/self-space/modules/ArticleAnalysisModule.vue';

export const ArticleAnalysisPlugin: SelfSpacePlugin = {
    id: 'article-analysis',
    label: 'Article Analysis',
    icon: 'ri-article-line', // RemixIcon
    component: ArticleAnalysisModule,
    order: 45, // After Vocabulary
};
