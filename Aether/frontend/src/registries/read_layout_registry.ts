import { type Component, defineAsyncComponent } from 'vue';

// Default Layout
const StandardReadLayout = defineAsyncComponent(() => import('@/components/layouts/StandardReadLayout.vue'));
const MathArchiveLayout = defineAsyncComponent(() => import('@/components/layouts/MathArchiveLayout.vue'));
const RendererErrorLayout = defineAsyncComponent(() => import('@/components/layouts/RendererErrorLayout.vue'));

const registry = new Map<string, Component>();

// Initialize with default
registry.set('default', StandardReadLayout);
registry.set('math_v1', MathArchiveLayout);

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
