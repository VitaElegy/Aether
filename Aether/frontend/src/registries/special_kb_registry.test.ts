import {
  SINGLETON_SPECIAL_KB_RENDERERS,
  resolveDashboardIdForRenderer,
  resolveLayoutIdForRenderer,
  resolvePluginIdForRenderer,
  resolvePortabilityProviderIdForRenderer,
  resolveSpecialKbRenderer,
} from './special_kb_registry';

describe('special_kb_registry', () => {
  it('maps legacy renderer ids to canonical app identities', () => {
    expect(resolveSpecialKbRenderer('english')?.canonicalRendererId).toBe('english_v1');
    expect(resolveSpecialKbRenderer('English Analysis')?.canonicalRendererId).toBe('article-analysis');
    expect(resolveSpecialKbRenderer('assets')?.canonicalRendererId).toBe('assets_v1');
    expect(resolveSpecialKbRenderer('system')?.canonicalRendererId).toBe('admin_system');
  });

  it('resolves plugin, layout, dashboard, and provider ids consistently', () => {
    expect(resolvePluginIdForRenderer('default')).toBe('knowledge');
    expect(resolvePluginIdForRenderer('math')).toBe('math');
    expect(resolveLayoutIdForRenderer('math')).toBe('math_v3');
    expect(resolveLayoutIdForRenderer('vrkb')).toBe('vulnerability_research');
    expect(resolveDashboardIdForRenderer('vulnerability_research')).toBe('vulnerability_research');
    expect(resolvePortabilityProviderIdForRenderer('vocabulary')).toBe('english_v1');
    expect(resolvePortabilityProviderIdForRenderer('assets')).toBe('assets_v1');
  });

  it('tracks singleton renderers using canonical ids only', () => {
    expect(SINGLETON_SPECIAL_KB_RENDERERS.has('memo')).toBe(true);
    expect(SINGLETON_SPECIAL_KB_RENDERERS.has('assets_v1')).toBe(true);
    expect(SINGLETON_SPECIAL_KB_RENDERERS.has('english')).toBe(false);
  });
});
