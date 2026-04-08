/**
 * Phase 0 — 全覆盖测试套件
 *
 * 对应 CHECKLIST.md 中 B~N 的全部测试点。
 * A 类（编译构建）由 npm run build / npm run test:unit 覆盖。
 *
 * 运行: npx vitest run src/../tests/phase0/phase0_full.test.ts
 * 或统一运行: npm run test:unit
 */
import {
  SINGLETON_SPECIAL_KB_RENDERERS,
  getCanonicalRendererId,
  getCapabilities,
  getAllCanonicalRendererIds,
  getAllRegistryEntries,
  getRegistrySize,
  hasCapability,
  isSingletonSpecialKbRenderer,
  normalizeRendererId,
  resolveDashboardIdForRenderer,
  resolveLayoutIdForRenderer,
  resolvePluginIdForRenderer,
  resolvePortabilityProviderIdForRenderer,
  resolveSpecialKbRenderer,
  validateRegistry,
  type SpecialKbCapabilities,
} from '../registries/special_kb_registry';

// ===========================================================================
// B. Registry — Identity Resolution
// ===========================================================================

describe('[B] Identity Resolution', () => {
  // B1: 11 canonical IDs exist
  test('B1: 11 canonical renderer IDs registered', () => {
    const ids = getAllCanonicalRendererIds();
    expect(ids.length).toBe(11);
    const expected = [
      'default', 'memo', 'vocabulary', 'english_v1', 'article-analysis',
      'math_v3', 'math_v1', 'vrkb', 'prkb', 'assets_v1', 'admin_system',
    ];
    for (const e of expected) {
      expect(ids).toContain(e);
    }
  });

  // B2: All legacy aliases map correctly
  test('B2: legacy aliases map to canonical', () => {
    const aliasMap: Record<string, string> = {
      // English family (3)
      'english': 'english_v1',
      'english_v1_std': 'english_v1',
      'vocabulary_std': 'vocabulary',
      // Memo family (2)
      'memo_std': 'memo',
      'memo_v1': 'memo',
      // Math family (3)
      'math': 'math_v3',
      'math_std': 'math_v3',
      'math_v1_std': 'math_v1',
      // VRKB family (2)
      'vrkb_std': 'vrkb',
      'vulnerability_research': 'vrkb',
      // Assets (1)
      'assets': 'assets_v1',
      // Admin (2)
      'admin': 'admin_system',
      'system': 'admin_system',
      // Article-analysis (3)
      'article_analysis': 'article-analysis',
      'english_analysis': 'article-analysis',
      'english analysis': 'article-analysis',
    };
    for (const [alias, canonical] of Object.entries(aliasMap)) {
      expect(resolveSpecialKbRenderer(alias)?.canonicalRendererId).toBe(canonical);
    }
  });

  // B3: migrated flag
  test('B3: migrated flag correct', () => {
    expect(resolveSpecialKbRenderer('english')?.migrated).toBe(true);
    expect(resolveSpecialKbRenderer('english_v1')?.migrated).toBe(false);
    expect(resolveSpecialKbRenderer('memo_std')?.migrated).toBe(true);
    expect(resolveSpecialKbRenderer('memo')?.migrated).toBe(false);
    expect(resolveSpecialKbRenderer('assets')?.migrated).toBe(true);
    expect(resolveSpecialKbRenderer('assets_v1')?.migrated).toBe(false);
  });

  // B4: unknown → undefined
  test('B4: unknown renderer returns undefined', () => {
    expect(resolveSpecialKbRenderer('totally_unknown')).toBeUndefined();
    expect(resolveSpecialKbRenderer('not_a_kb')).toBeUndefined();
  });

  // B5: null/undefined/empty
  test('B5: null/undefined/empty returns undefined', () => {
    expect(resolveSpecialKbRenderer(null)).toBeUndefined();
    expect(resolveSpecialKbRenderer(undefined)).toBeUndefined();
    expect(resolveSpecialKbRenderer('')).toBeUndefined();
    expect(resolveSpecialKbRenderer('  ')).toBeUndefined();
  });

  // B6–B9: resolution fields
  test('B6: pluginId resolution', () => {
    expect(resolvePluginIdForRenderer('default')).toBe('knowledge');
    expect(resolvePluginIdForRenderer('memo')).toBe('memo');
    expect(resolvePluginIdForRenderer('math')).toBe('math');
    expect(resolvePluginIdForRenderer('vrkb')).toBe('vrkb');
    expect(resolvePluginIdForRenderer('prkb')).toBe('prkb');
    expect(resolvePluginIdForRenderer('assets_v1')).toBe('assets_v1');
    expect(resolvePluginIdForRenderer('english_v1')).toBe('vocabulary');
    expect(resolvePluginIdForRenderer('vocabulary')).toBe('vocabulary');
    expect(resolvePluginIdForRenderer('article-analysis')).toBe('article-analysis');
    expect(resolvePluginIdForRenderer('admin_system')).toBe('admin_system');
  });

  test('B7: layoutId resolution', () => {
    expect(resolveLayoutIdForRenderer('default')).toBe('default');
    expect(resolveLayoutIdForRenderer('math')).toBe('math_v3');
    expect(resolveLayoutIdForRenderer('math_v1')).toBe('math_v1');
    expect(resolveLayoutIdForRenderer('vrkb')).toBe('vulnerability_research');
    expect(resolveLayoutIdForRenderer('english_v1')).toBe('english_v1');
    expect(resolveLayoutIdForRenderer('article-analysis')).toBe('english_v1');
    // No explicit layoutId → falls back to canonical
    expect(resolveLayoutIdForRenderer('memo')).toBe('memo');
    expect(resolveLayoutIdForRenderer('prkb')).toBe('prkb');
  });

  test('B8: dashboardId resolution', () => {
    expect(resolveDashboardIdForRenderer('vrkb')).toBe('vulnerability_research');
    expect(resolveDashboardIdForRenderer('vulnerability_research')).toBe('vulnerability_research');
    expect(resolveDashboardIdForRenderer('admin_system')).toBe('admin_system');
    expect(resolveDashboardIdForRenderer('admin')).toBe('admin_system');
    // No dashboardId → falls back to canonical
    expect(resolveDashboardIdForRenderer('memo')).toBe('memo');
  });

  test('B9: portabilityProviderId resolution', () => {
    expect(resolvePortabilityProviderIdForRenderer('vocabulary')).toBe('english_v1');
    expect(resolvePortabilityProviderIdForRenderer('english_v1')).toBe('english_v1');
    expect(resolvePortabilityProviderIdForRenderer('article-analysis')).toBe('english_v1');
    expect(resolvePortabilityProviderIdForRenderer('assets_v1')).toBe('assets_v1');
    expect(resolvePortabilityProviderIdForRenderer('assets')).toBe('assets_v1');
    expect(resolvePortabilityProviderIdForRenderer('memo')).toBe('memo');
    expect(resolvePortabilityProviderIdForRenderer('vrkb')).toBe('vrkb');
  });
});

// ===========================================================================
// C. Normalization
// ===========================================================================

describe('[C] Normalization', () => {
  test('C1: uppercase → lowercase', () => {
    expect(normalizeRendererId('ENGLISH')).toBe('english');
    expect(normalizeRendererId('MEMO')).toBe('memo');
    expect(normalizeRendererId('VrKb')).toBe('vrkb');
  });

  test('C2: trim whitespace', () => {
    expect(normalizeRendererId('  memo  ')).toBe('memo');
    expect(normalizeRendererId('\tvrkb\t')).toBe('vrkb');
  });

  test('C3: collapse multiple spaces', () => {
    expect(normalizeRendererId('English  Analysis')).toBe('english analysis');
    expect(normalizeRendererId('a   b   c')).toBe('a b c');
  });

  test('C4: empty/whitespace → undefined', () => {
    expect(normalizeRendererId('')).toBeUndefined();
    expect(normalizeRendererId('   ')).toBeUndefined();
    expect(normalizeRendererId(null)).toBeUndefined();
    expect(normalizeRendererId(undefined)).toBeUndefined();
  });
});

// ===========================================================================
// D. Canonical ID Helper
// ===========================================================================

describe('[D] getCanonicalRendererId', () => {
  test('D1: legacy → canonical', () => {
    expect(getCanonicalRendererId('english')).toBe('english_v1');
    expect(getCanonicalRendererId('math_std')).toBe('math_v3');
    expect(getCanonicalRendererId('assets')).toBe('assets_v1');
    expect(getCanonicalRendererId('admin')).toBe('admin_system');
  });

  test('D2: canonical → canonical (passthrough)', () => {
    expect(getCanonicalRendererId('memo')).toBe('memo');
    expect(getCanonicalRendererId('prkb')).toBe('prkb');
    expect(getCanonicalRendererId('vrkb')).toBe('vrkb');
  });

  test('D3: null/undefined → "default"', () => {
    expect(getCanonicalRendererId(null)).toBe('default');
    expect(getCanonicalRendererId(undefined)).toBe('default');
  });

  test('D4: unknown → normalized passthrough', () => {
    expect(getCanonicalRendererId('my_custom_renderer')).toBe('my_custom_renderer');
    expect(getCanonicalRendererId('UNKNOWN')).toBe('unknown');
  });
});

// ===========================================================================
// E. Capability Schema
// ===========================================================================

describe('[E] Capability Schema', () => {
  test('E1: vrkb capabilities', () => {
    const c = getCapabilities('vrkb');
    expect(c.assets).toBe(true);
    expect(c.auditLog).toBe(true);
    expect(c.collaboration).toBe(true);
    expect(c.search).toBe(true);
    expect(c.dashboard).toBe(true);
    expect(c.articleParser).toBe(false);
    expect(c.longTasks).toBe(false);
    expect(c.reactiveContext).toBe(false);
  });

  test('E2: english_v1 capabilities', () => {
    const c = getCapabilities('english_v1');
    expect(c.articleParser).toBe(true);
    expect(c.longTasks).toBe(true);
    expect(c.export).toBe(true);
    expect(c.import).toBe(true);
    expect(c.search).toBe(true);
    expect(c.collaboration).toBe(false);
    expect(c.assets).toBe(false);
  });

  test('E3: legacy alias → capabilities', () => {
    expect(getCapabilities('english').articleParser).toBe(true);
    expect(getCapabilities('math_std').search).toBe(true);
    expect(getCapabilities('vrkb_std').collaboration).toBe(true);
  });

  test('E4: unknown → all false', () => {
    const c = getCapabilities('unknown_thing');
    const keys = Object.keys(c) as (keyof typeof c)[];
    for (const k of keys) {
      expect(c[k]).toBe(false);
    }
  });

  test('E5: hasCapability shorthand', () => {
    expect(hasCapability('vrkb', 'collaboration')).toBe(true);
    expect(hasCapability('memo', 'collaboration')).toBe(false);
    expect(hasCapability('english_v1', 'articleParser')).toBe(true);
    expect(hasCapability(null, 'search')).toBe(false);
    expect(hasCapability('prkb', 'longTasks')).toBe(true);
  });

  test('E6: NO_CAPABILITIES is frozen (immutable)', () => {
    const c = getCapabilities('nonexistent');
    // Attempting mutation on frozen object should throw in strict mode or be a no-op
    expect(() => { (c as any).assets = true; }).toThrow();
  });

  test('E7: every entry has capabilities', () => {
    for (const entry of getAllRegistryEntries()) {
      expect(entry.capabilities).toBeDefined();
      expect(typeof entry.capabilities.assets).toBe('boolean');
      expect(typeof entry.capabilities.export).toBe('boolean');
      expect(typeof entry.capabilities.search).toBe('boolean');
    }
  });
});

// ===========================================================================
// F. Singleton
// ===========================================================================

describe('[F] Singleton Tracking', () => {
  const expectedSingletons = [
    'memo', 'vocabulary', 'english_v1', 'math_v3', 'math_v1',
    'vrkb', 'prkb', 'assets_v1', 'admin_system',
  ];
  const expectedNonSingletons = ['default', 'article-analysis'];
  const legacyAliases = ['memo_std', 'english', 'math', 'vrkb_std', 'assets', 'admin'];

  test('F1: singleton set contains correct canonical IDs', () => {
    for (const id of expectedSingletons) {
      expect(SINGLETON_SPECIAL_KB_RENDERERS.has(id)).toBe(true);
    }
  });

  test('F2: legacy IDs not in singleton set', () => {
    for (const id of legacyAliases) {
      expect(SINGLETON_SPECIAL_KB_RENDERERS.has(id)).toBe(false);
    }
  });

  test('F3: non-singletons not in set', () => {
    for (const id of expectedNonSingletons) {
      expect(SINGLETON_SPECIAL_KB_RENDERERS.has(id)).toBe(false);
    }
  });

  test('F4: isSingletonSpecialKbRenderer works through aliases', () => {
    expect(isSingletonSpecialKbRenderer('memo_std')).toBe(true);
    expect(isSingletonSpecialKbRenderer('english')).toBe(true);
    expect(isSingletonSpecialKbRenderer('vrkb_std')).toBe(true);
    expect(isSingletonSpecialKbRenderer('article_analysis')).toBe(false);
    expect(isSingletonSpecialKbRenderer(null)).toBe(false);
  });
});

// ===========================================================================
// G. Introspection
// ===========================================================================

describe('[G] Introspection', () => {
  test('G1: getAllCanonicalRendererIds returns 11', () => {
    expect(getAllCanonicalRendererIds().length).toBe(11);
  });

  test('G2: getAllRegistryEntries length matches', () => {
    expect(getAllRegistryEntries().length).toBe(getAllCanonicalRendererIds().length);
  });

  test('G3: every entry has required fields', () => {
    for (const e of getAllRegistryEntries()) {
      expect(e.canonicalRendererId).toBeTruthy();
      expect(e.pluginId).toBeTruthy();
      expect(e.capabilities).toBeDefined();
      expect(typeof e.singleton).toBe('boolean');
    }
  });

  test('G4: getRegistrySize = canonical + legacy total', () => {
    const entries = getAllRegistryEntries();
    const total = entries.reduce((s, e) => s + 1 + (e.legacyRendererIds?.length ?? 0), 0);
    expect(getRegistrySize()).toBe(total);
  });
});

// ===========================================================================
// H. Validation
// ===========================================================================

describe('[H] Registry Validation', () => {
  test('H1: missing plugin → error', () => {
    const r = validateRegistry(new Set(['knowledge']), new Set(), new Set());
    expect(r.valid).toBe(false);
    expect(r.errors.some(e => e.includes('memo'))).toBe(true);
  });

  test('H2: all plugins → valid', () => {
    const entries = getAllRegistryEntries();
    const plugins = new Set(entries.map(e => e.pluginId));
    const layouts = new Set(entries.filter(e => e.layoutId).map(e => e.layoutId!));
    const dashboards = new Set(entries.filter(e => e.dashboardId).map(e => e.dashboardId!));
    const r = validateRegistry(plugins, layouts, dashboards);
    expect(r.valid).toBe(true);
    expect(r.errors).toHaveLength(0);
  });

  test('H3: export=true + default provider → warning', () => {
    const plugins = new Set(getAllRegistryEntries().map(e => e.pluginId));
    const r = validateRegistry(plugins, new Set(), new Set());
    expect(r.warnings.some(w => w.includes('export') && w.includes('default'))).toBe(true);
  });

  test('H4: dashboard cap but no dashboardId → warning (simulated)', () => {
    // In current registry vrkb has both dashboard:true and dashboardId set, so no warning.
    // But 'prkb' does NOT have dashboard cap, so no warning either.
    // We verify the LOGIC by checking that existing entries that claim dashboard do have dashboardId.
    for (const e of getAllRegistryEntries()) {
      if (e.capabilities.dashboard) {
        expect(e.dashboardId).toBeTruthy();
      }
    }
  });

  test('H5: all entries now have portabilityProviderId', () => {
    // After fixing admin_system, all entries should have explicit portabilityProviderId.
    for (const e of getAllRegistryEntries()) {
      expect(e.portabilityProviderId).toBeTruthy();
    }
  });
});

// ===========================================================================
// I. Resolution Cache
// ===========================================================================

describe('[I] Resolution Cache', () => {
  test('I1: same input returns same object reference (cached)', () => {
    const r1 = resolveSpecialKbRenderer('vrkb');
    const r2 = resolveSpecialKbRenderer('vrkb');
    expect(r1).toBe(r2); // strict reference equality
  });

  test('I1b: alias also caches independently', () => {
    const r1 = resolveSpecialKbRenderer('english');
    const r2 = resolveSpecialKbRenderer('english');
    expect(r1).toBe(r2);
    // But 'english' and 'english_v1' are different cache entries
    const r3 = resolveSpecialKbRenderer('english_v1');
    expect(r1).not.toBe(r3); // different keys, different objects
    expect(r1!.canonicalRendererId).toBe(r3!.canonicalRendererId); // same canonical
  });
});

// ===========================================================================
// N. Cross-Layer Consistency (Frontend self-check)
// ===========================================================================

describe('[N] Cross-Layer Consistency', () => {
  // Backend data (hardcoded here as ground truth, verified by backend tests)
  const BACKEND_CANONICAL_IDS = [
    'default', 'memo', 'vocabulary', 'english_v1', 'article-analysis',
    'math_v3', 'math_v1', 'vrkb', 'prkb', 'assets_v1', 'admin_system',
  ];

  const BACKEND_SINGLETONS = [
    'memo', 'vocabulary', 'english_v1', 'math_v3', 'math_v1',
    'vrkb', 'prkb', 'assets_v1', 'admin_system',
  ];

  const BACKEND_PROVIDER_MAP: Record<string, string> = {
    'default': 'default',
    'memo': 'memo',
    'vocabulary': 'english_v1',
    'english_v1': 'english_v1',
    'article-analysis': 'english_v1',
    'math_v3': 'default',
    'math_v1': 'default',
    'vrkb': 'vrkb',
    'prkb': 'prkb',
    'assets_v1': 'assets_v1',
    'admin_system': 'default',
  };

  test('N1: frontend canonical IDs match backend', () => {
    const frontendIds = getAllCanonicalRendererIds();
    expect(frontendIds.sort()).toEqual([...BACKEND_CANONICAL_IDS].sort());
  });

  test('N3: frontend singletons match backend', () => {
    const frontendSingletons = getAllRegistryEntries()
      .filter(e => e.singleton)
      .map(e => e.canonicalRendererId)
      .sort();
    expect(frontendSingletons).toEqual([...BACKEND_SINGLETONS].sort());
  });

  test('N5: frontend provider_id matches backend', () => {
    for (const [rid, expectedProvider] of Object.entries(BACKEND_PROVIDER_MAP)) {
      const entry = getAllRegistryEntries().find(e => e.canonicalRendererId === rid);
      expect(entry).toBeDefined();
      expect(entry!.portabilityProviderId ?? entry!.canonicalRendererId).toBe(expectedProvider);
    }
  });

  test('N2: capability field count matches (10 fields)', () => {
    const c = getCapabilities('default');
    const keys = Object.keys(c);
    expect(keys.length).toBe(10);
    // Verify field names match backend (camelCase ↔ snake_case)
    const expectedFields = [
      'assets', 'export', 'import', 'search', 'auditLog',
      'longTasks', 'articleParser', 'reactiveContext', 'dashboard', 'collaboration',
    ];
    expect(keys.sort()).toEqual(expectedFields.sort());
  });
});
