import { type Component, defineAsyncComponent } from 'vue';

// Default Layout
const StandardReadLayout = defineAsyncComponent(() => import('@/components/layouts/StandardReadLayout.vue'));
const MathArchiveLayout = defineAsyncComponent(() => import('@/components/layouts/MathArchiveLayout.vue'));
const RendererErrorLayout = defineAsyncComponent(() => import('@/components/layouts/RendererErrorLayout.vue'));
const MathManuscriptLayout = defineAsyncComponent(() => import('@/components/layouts/MathManuscriptLayout.vue'));

const registry = new Map<string, Component>();

// Initialize with default
registry.set('default', StandardReadLayout);
registry.set('math_v1', MathArchiveLayout);
registry.set('math_v3', MathManuscriptLayout);

const EnglishArticleAnalyzer = defineAsyncComponent(() => import('@/components/english/EnglishArticleAnalyzer.vue'));
registry.set('english_v1', EnglishArticleAnalyzer);

export const registerLayout = (id: string, component: Component) => {
    registry.set(id, component);
    console.log(`[ReadLayoutRegistry] Registered: ${id}`);
};

export const getLayout = (id: string | null | undefined): Component => {
    if (!id || id === 'default') {
        return StandardReadLayout;
    }

    const layout = registry.get(id);
    if (!layout) {
        console.warn(`[ReadLayoutRegistry] Layout '${id}' not found.`);
        return RendererErrorLayout;
    }
    return layout;
};

// -- Dashboard Registry --
const dashboardRegistry = new Map<string, Component>();

// We will register components lazily or manually
// dashboardRegistry.set('math_v1', MathDashboard);

export const registerDashboard = (id: string, component: Component) => {
    dashboardRegistry.set(id, component);
};

export const getDashboard = (id: string | null | undefined): Component | null => {
    if (!id) return null; // Default handling provided by view
    return dashboardRegistry.get(id) || null;
};
