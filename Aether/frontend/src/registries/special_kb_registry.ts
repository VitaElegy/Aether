export interface SpecialKbRegistryEntry {
    canonicalRendererId: string;
    pluginId: string;
    layoutId?: string;
    dashboardId?: string;
    portabilityProviderId?: string;
    singleton: boolean;
    legacyRendererIds?: string[];
}

export interface SpecialKbRendererResolution {
    requestedId: string;
    canonicalRendererId: string;
    pluginId: string;
    layoutId?: string;
    dashboardId?: string;
    portabilityProviderId?: string;
    singleton: boolean;
    migrated: boolean;
}

const SPECIAL_KB_REGISTRY: SpecialKbRegistryEntry[] = [
    {
        canonicalRendererId: 'default',
        pluginId: 'knowledge',
        layoutId: 'default',
        portabilityProviderId: 'default',
        singleton: false,
    },
    {
        canonicalRendererId: 'memo',
        pluginId: 'memo',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['memo_std', 'memo_v1'],
    },
    {
        canonicalRendererId: 'vocabulary',
        pluginId: 'vocabulary',
        portabilityProviderId: 'english_v1',
        singleton: true,
        legacyRendererIds: ['vocabulary_std'],
    },
    {
        canonicalRendererId: 'english_v1',
        pluginId: 'vocabulary',
        layoutId: 'english_v1',
        portabilityProviderId: 'english_v1',
        singleton: true,
        legacyRendererIds: ['english', 'english_v1_std'],
    },
    {
        canonicalRendererId: 'article-analysis',
        pluginId: 'article-analysis',
        layoutId: 'english_v1',
        portabilityProviderId: 'english_v1',
        singleton: false,
        legacyRendererIds: ['article_analysis', 'english_analysis', 'english analysis'],
    },
    {
        canonicalRendererId: 'math_v3',
        pluginId: 'math',
        layoutId: 'math_v3',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['math', 'math_std'],
    },
    {
        canonicalRendererId: 'math_v1',
        pluginId: 'math',
        layoutId: 'math_v1',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['math_v1_std'],
    },
    {
        canonicalRendererId: 'vrkb',
        pluginId: 'vrkb',
        layoutId: 'vulnerability_research',
        dashboardId: 'vulnerability_research',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['vrkb_std', 'vulnerability_research'],
    },
    {
        canonicalRendererId: 'prkb',
        pluginId: 'prkb',
        portabilityProviderId: 'default',
        singleton: true,
    },
    {
        canonicalRendererId: 'assets_v1',
        pluginId: 'assets_v1',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['assets'],
    },
    {
        canonicalRendererId: 'admin_system',
        pluginId: 'admin_system',
        dashboardId: 'admin_system',
        singleton: true,
        legacyRendererIds: ['admin', 'system'],
    },
];

const lookup = new Map<string, SpecialKbRegistryEntry>();

function registerEntry(entry: SpecialKbRegistryEntry) {
    const keys = [entry.canonicalRendererId, ...(entry.legacyRendererIds ?? [])];
    for (const rawKey of keys) {
        const key = normalizeRendererId(rawKey);
        if (!key) {
            continue;
        }
        if (lookup.has(key)) {
            throw new Error(`[SpecialKbRegistry] Duplicate renderer mapping for '${key}'.`);
        }
        lookup.set(key, entry);
    }
}

SPECIAL_KB_REGISTRY.forEach(registerEntry);

export function normalizeRendererId(rendererId: string | null | undefined): string | undefined {
    if (!rendererId) {
        return undefined;
    }

    const normalized = rendererId.trim().toLowerCase().replace(/\s+/g, ' ');
    return normalized.length > 0 ? normalized : undefined;
}

export function resolveSpecialKbRenderer(rendererId: string | null | undefined): SpecialKbRendererResolution | undefined {
    const normalized = normalizeRendererId(rendererId);
    if (!normalized) {
        return undefined;
    }

    const entry = lookup.get(normalized);
    if (!entry) {
        return undefined;
    }

    return {
        requestedId: normalized,
        canonicalRendererId: entry.canonicalRendererId,
        pluginId: entry.pluginId,
        layoutId: entry.layoutId,
        dashboardId: entry.dashboardId,
        portabilityProviderId: entry.portabilityProviderId,
        singleton: entry.singleton,
        migrated: normalized !== entry.canonicalRendererId,
    };
}

export function resolvePluginIdForRenderer(rendererId: string | null | undefined): string | undefined {
    return resolveSpecialKbRenderer(rendererId)?.pluginId;
}

export function resolveLayoutIdForRenderer(rendererId: string | null | undefined): string | undefined {
    const resolution = resolveSpecialKbRenderer(rendererId);
    if (!resolution) {
        return normalizeRendererId(rendererId);
    }
    return resolution.layoutId ?? resolution.canonicalRendererId;
}

export function resolveDashboardIdForRenderer(rendererId: string | null | undefined): string | undefined {
    const resolution = resolveSpecialKbRenderer(rendererId);
    if (!resolution) {
        return normalizeRendererId(rendererId);
    }
    return resolution.dashboardId ?? resolution.canonicalRendererId;
}

export function resolvePortabilityProviderIdForRenderer(rendererId: string | null | undefined): string | undefined {
    const resolution = resolveSpecialKbRenderer(rendererId);
    if (!resolution) {
        return normalizeRendererId(rendererId);
    }
    return resolution.portabilityProviderId ?? resolution.canonicalRendererId;
}

export const SINGLETON_SPECIAL_KB_RENDERERS = new Set(
    SPECIAL_KB_REGISTRY
        .filter((entry) => entry.singleton)
        .map((entry) => entry.canonicalRendererId),
);

export function isSingletonSpecialKbRenderer(rendererId: string | null | undefined): boolean {
    const resolution = resolveSpecialKbRenderer(rendererId);
    return resolution?.singleton ?? false;
}
