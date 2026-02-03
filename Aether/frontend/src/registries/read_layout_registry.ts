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
registry.set('vulnerability_research', defineAsyncComponent(() => import('@/components/layouts/VRLayout.vue')));

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
dashboardRegistry.set('vulnerability_research', defineAsyncComponent(() => import('@/views/VRProjectDashboard.vue')));
dashboardRegistry.set('admin_system', defineAsyncComponent(() => import('@/views/dashboard/AdminDashboard.vue')));

export const registerDashboard = (id: string, component: Component) => {
    dashboardRegistry.set(id, component);
};

export const getDashboard = (id: string | null | undefined): Component | null => {
    if (!id) return null; // Default handling provided by view
    return dashboardRegistry.get(id) || null;
};

// -- Metadata Registry --
import { ref } from 'vue';
import { templateApi, type LayoutTemplate } from '@/api/template';

// We map 'renderer_id' (from DB/Template) -> Component (Code)
// This remains static/hardcoded because components must exist in the bundle.
const RENDERER_MAP: Record<string, string> = {
    'default': 'default',
    'math_v1': 'math_v1',
    'math_v3': 'math_v3',
    'english_v1': 'english_v1',
    'vrkb': 'vulnerability_research',
    'admin_system': 'admin_system',
};

export const LAYOUTS = ref<LayoutTemplate[]>([]);

// Fallback defaults if API fails or for initial boot
const DEFAULTS: LayoutTemplate[] = [
    {
        id: 'default_std',
        renderer_id: 'default',
        title: 'Standard Blog',
        description: 'A clean, readable layout optimized for general writing.',
        thumbnail: 'bg-gradient-to-br from-gray-100 to-gray-200',
        tags: ['General'],
        created_at: new Date().toISOString()
    },
    {
        id: 'math_v1_std',
        renderer_id: 'math_v1',
        title: 'Math Archive (Graph)',
        description: 'Graph-centric visualization for mathematical axioms and theorems.',
        thumbnail: 'bg-gradient-to-br from-blue-900 to-slate-900',
        tags: ['Mathematics', 'Graph'],
        created_at: new Date().toISOString()
    },
    {
        id: 'math_v3_std',
        renderer_id: 'math_v3',
        title: 'Math Manuscript',
        description: 'Book-like rendering for dense mathematical proofs.',
        thumbnail: 'bg-[#FDF6E3]',
        tags: ['Academic', 'Math'],
        created_at: new Date().toISOString()
    },
    {
        id: 'english_v1_std',
        renderer_id: 'english_v1',
        title: 'English Analysis',
        description: 'Immersive philological environment with sentence analysis.',
        thumbnail: 'bg-stone-200',
        tags: ['Language', 'Learning'],
        created_at: new Date().toISOString()
    },
    {
        id: 'memo_std',
        renderer_id: 'memo',
        title: 'Memos',
        description: 'Sticky note interface for transient thoughts.',
        thumbnail: 'bg-yellow-100',
        tags: ['Productivity', 'Notes'],
        created_at: new Date().toISOString()
    },
    {
        id: 'vocabulary_std',
        renderer_id: 'vocabulary',
        title: 'Vocabulary',
        description: 'Spaced repetition card system for building lexicons.',
        thumbnail: 'bg-indigo-50',
        tags: ['Language', 'Cards'],
        created_at: new Date().toISOString()
    },
    {
        id: 'vrkb_std',
        renderer_id: 'vrkb',
        title: 'Vulnerability Research',
        description: 'Kanban-style project management board.',
        thumbnail: 'bg-slate-800 text-green-500',
        tags: ['Security'],
        created_at: new Date().toISOString()
    }
];

export const fetchLayouts = async () => {
    try {
        const { data } = await templateApi.list();
        if (data && data.length > 0) {
            LAYOUTS.value = data;
        } else {
            LAYOUTS.value = DEFAULTS;
        }
    } catch (e) {
        console.warn("Failed to fetch dynamic layouts, using defaults.", e);
        LAYOUTS.value = DEFAULTS;
    }
};

// Auto-fetch on import (mostly safe for client-side)
fetchLayouts();
