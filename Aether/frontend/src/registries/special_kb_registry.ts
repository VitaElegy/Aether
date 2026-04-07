// ──────────────────────────────────────────────
// Capability Schema
// ──────────────────────────────────────────────

export interface SpecialKbCapabilities {
    assets: boolean;
    export: boolean;
    import: boolean;
    search: boolean;
    auditLog: boolean;
    longTasks: boolean;
    articleParser: boolean;
    reactiveContext: boolean;
    dashboard: boolean;
    collaboration: boolean;
}

const NO_CAPABILITIES: Readonly<SpecialKbCapabilities> = Object.freeze({
    assets: false,
    export: false,
    import: false,
    search: false,
    auditLog: false,
    longTasks: false,
    articleParser: false,
    reactiveContext: false,
    dashboard: false,
    collaboration: false,
});

// ──────────────────────────────────────────────
// Registry Entry
// ──────────────────────────────────────────────

export interface SpecialKbRegistryEntry {
    canonicalRendererId: string;
    pluginId: string;
    layoutId?: string;
    dashboardId?: string;
    portabilityProviderId?: string;
    singleton: boolean;
    legacyRendererIds?: string[];
    capabilities: SpecialKbCapabilities;
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

// ──────────────────────────────────────────────
// Registry Data
// ──────────────────────────────────────────────

const SPECIAL_KB_REGISTRY: SpecialKbRegistryEntry[] = [
    {
        canonicalRendererId: 'default',
        pluginId: 'knowledge',
        layoutId: 'default',
        portabilityProviderId: 'default',
        singleton: false,
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true },
    },
    {
        canonicalRendererId: 'memo',
        pluginId: 'memo',
        portabilityProviderId: 'memo',
        singleton: true,
        legacyRendererIds: ['memo_std', 'memo_v1'],
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true },
    },
    {
        canonicalRendererId: 'vocabulary',
        pluginId: 'vocabulary',
        portabilityProviderId: 'english_v1',
        singleton: true,
        legacyRendererIds: ['vocabulary_std'],
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true, longTasks: true, articleParser: true },
    },
    {
        canonicalRendererId: 'english_v1',
        pluginId: 'vocabulary',
        layoutId: 'english_v1',
        portabilityProviderId: 'english_v1',
        singleton: true,
        legacyRendererIds: ['english', 'english_v1_std'],
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true, longTasks: true, articleParser: true },
    },
    {
        canonicalRendererId: 'article-analysis',
        pluginId: 'article-analysis',
        layoutId: 'english_v1',
        portabilityProviderId: 'english_v1',
        singleton: false,
        legacyRendererIds: ['article_analysis', 'english_analysis', 'english analysis'],
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true, longTasks: true, articleParser: true },
    },
    {
        canonicalRendererId: 'math_v3',
        pluginId: 'math',
        layoutId: 'math_v3',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['math', 'math_std'],
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true },
    },
    {
        canonicalRendererId: 'math_v1',
        pluginId: 'math',
        layoutId: 'math_v1',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['math_v1_std'],
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true },
    },
    {
        canonicalRendererId: 'vrkb',
        pluginId: 'vrkb',
        layoutId: 'vulnerability_research',
        dashboardId: 'vulnerability_research',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['vrkb_std', 'vulnerability_research'],
        capabilities: { ...NO_CAPABILITIES, assets: true, auditLog: true, collaboration: true, search: true, dashboard: true, export: true, import: true },
    },
    {
        canonicalRendererId: 'prkb',
        pluginId: 'prkb',
        portabilityProviderId: 'prkb',
        singleton: true,
        capabilities: { ...NO_CAPABILITIES, export: true, import: true, search: true, longTasks: true },
    },
    {
        canonicalRendererId: 'assets_v1',
        pluginId: 'assets_v1',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['assets'],
        capabilities: { ...NO_CAPABILITIES, assets: true, export: true, import: true, search: true },
    },
    {
        canonicalRendererId: 'admin_system',
        pluginId: 'admin_system',
        dashboardId: 'admin_system',
        portabilityProviderId: 'default',
        singleton: true,
        legacyRendererIds: ['admin', 'system'],
        capabilities: { ...NO_CAPABILITIES, dashboard: true, auditLog: true },
    },
];

// ──────────────────────────────────────────────
// Lookup Table
// ──────────────────────────────────────────────

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

// Resolution cache (same input → same object reference)
const resolutionCache = new Map<string, SpecialKbRendererResolution>();

// ──────────────────────────────────────────────
// Normalization
// ──────────────────────────────────────────────

export function normalizeRendererId(rendererId: string | null | undefined): string | undefined {
    if (!rendererId) {
        return undefined;
    }

    const normalized = rendererId.trim().toLowerCase().replace(/\s+/g, ' ');
    return normalized.length > 0 ? normalized : undefined;
}

// ──────────────────────────────────────────────
// Resolution
// ──────────────────────────────────────────────

export function resolveSpecialKbRenderer(rendererId: string | null | undefined): SpecialKbRendererResolution | undefined {
    const normalized = normalizeRendererId(rendererId);
    if (!normalized) {
        return undefined;
    }

    if (resolutionCache.has(normalized)) {
        return resolutionCache.get(normalized)!;
    }

    const entry = lookup.get(normalized);
    if (!entry) {
        return undefined;
    }

    const resolution: SpecialKbRendererResolution = {
        requestedId: normalized,
        canonicalRendererId: entry.canonicalRendererId,
        pluginId: entry.pluginId,
        layoutId: entry.layoutId,
        dashboardId: entry.dashboardId,
        portabilityProviderId: entry.portabilityProviderId,
        singleton: entry.singleton,
        migrated: normalized !== entry.canonicalRendererId,
    };

    resolutionCache.set(normalized, resolution);
    return resolution;
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

// ──────────────────────────────────────────────
// Singleton Tracking
// ──────────────────────────────────────────────

export const SINGLETON_SPECIAL_KB_RENDERERS = new Set(
    SPECIAL_KB_REGISTRY
        .filter((entry) => entry.singleton)
        .map((entry) => entry.canonicalRendererId),
);

export function isSingletonSpecialKbRenderer(rendererId: string | null | undefined): boolean {
    const resolution = resolveSpecialKbRenderer(rendererId);
    return resolution?.singleton ?? false;
}

// ──────────────────────────────────────────────
// Canonical ID Helper
// ──────────────────────────────────────────────

export function getCanonicalRendererId(rendererId: string | null | undefined): string {
    const resolution = resolveSpecialKbRenderer(rendererId);
    if (resolution) {
        return resolution.canonicalRendererId;
    }
    // null/undefined → 'default'; unknown → normalized passthrough
    const normalized = normalizeRendererId(rendererId);
    return normalized ?? 'default';
}

// ──────────────────────────────────────────────
// Capability Helpers
// ──────────────────────────────────────────────

export function getCapabilities(rendererId: string | null | undefined): Readonly<SpecialKbCapabilities> {
    const normalized = normalizeRendererId(rendererId);
    if (!normalized) {
        return NO_CAPABILITIES;
    }
    const entry = lookup.get(normalized);
    if (!entry) {
        return NO_CAPABILITIES;
    }
    return Object.freeze({ ...entry.capabilities });
}

export function hasCapability(rendererId: string | null | undefined, capability: keyof SpecialKbCapabilities): boolean {
    return getCapabilities(rendererId)[capability];
}

// ──────────────────────────────────────────────
// Introspection
// ──────────────────────────────────────────────

export function getAllCanonicalRendererIds(): string[] {
    return SPECIAL_KB_REGISTRY.map((entry) => entry.canonicalRendererId);
}

export function getAllRegistryEntries(): readonly SpecialKbRegistryEntry[] {
    return SPECIAL_KB_REGISTRY;
}

export function getRegistrySize(): number {
    return lookup.size;
}

// ──────────────────────────────────────────────
// Validation
// ──────────────────────────────────────────────

export interface RegistryValidationResult {
    valid: boolean;
    errors: string[];
    warnings: string[];
}

export function validateRegistry(
    availablePlugins: Set<string>,
    availableLayouts: Set<string>,
    availableDashboards: Set<string>,
): RegistryValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    for (const entry of SPECIAL_KB_REGISTRY) {
        // Plugin must exist
        if (!availablePlugins.has(entry.pluginId)) {
            errors.push(`[${entry.canonicalRendererId}] references missing plugin '${entry.pluginId}'`);
        }

        // Layout must exist if specified
        if (entry.layoutId && !availableLayouts.has(entry.layoutId)) {
            warnings.push(`[${entry.canonicalRendererId}] references layout '${entry.layoutId}' not in available set`);
        }

        // Dashboard must exist if specified
        if (entry.dashboardId && !availableDashboards.has(entry.dashboardId)) {
            warnings.push(`[${entry.canonicalRendererId}] references dashboard '${entry.dashboardId}' not in available set`);
        }

        // Warn if export capability but using default portability provider
        if (entry.capabilities.export && entry.portabilityProviderId === 'default' && entry.canonicalRendererId !== 'default') {
            warnings.push(`[${entry.canonicalRendererId}] has export capability but uses default portability provider`);
        }

        // Warn if dashboard capability but no dashboardId
        if (entry.capabilities.dashboard && !entry.dashboardId) {
            warnings.push(`[${entry.canonicalRendererId}] has dashboard capability but no dashboardId`);
        }
    }

    return {
        valid: errors.length === 0,
        errors,
        warnings,
    };
}
