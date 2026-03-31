import { setActivePinia, createPinia } from 'pinia';
import { defineComponent } from 'vue';
import { usePluginStore } from './plugins';
import type { SelfSpacePlugin } from '@/core/plugin';

function makePlugin(id: string, order: number): SelfSpacePlugin {
  return {
    id,
    dock: {
      label: id,
      icon: 'ri-test-tube-line',
      order,
    },
    capabilities: {
      articleParser: false,
    },
    component: defineComponent({ name: `${id}-plugin`, template: '<div />' }),
  };
}

describe('usePluginStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('sorts plugins by dock order and ignores duplicates', () => {
    const store = usePluginStore();
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    store.registerPlugin(makePlugin('later', 20));
    store.registerPlugin(makePlugin('earlier', 10));
    store.registerPlugin(makePlugin('later', 20));

    expect(store.plugins.map((plugin) => plugin.id)).toEqual(['earlier', 'later']);
    expect(warnSpy).toHaveBeenCalledTimes(1);
  });

  it('resolves plugin ids strictly but normalizes whitespace and casing', () => {
    const store = usePluginStore();
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});

    store.registerPlugin(makePlugin('vrkb', 10));
    store.registerPlugin(makePlugin('knowledge', 0));
    store.registerPlugin(makePlugin('article-analysis', 20));

    expect(store.resolvePlugin('  VRKB  ')?.id).toBe('vrkb');
    expect(store.resolvePlugin('vrkb_std')?.id).toBe('vrkb');
    expect(store.resolvePlugin('default')?.id).toBe('knowledge');
    expect(store.resolvePlugin('English Analysis')?.id).toBe('article-analysis');
    expect(store.resolvePlugin('missing-plugin')).toBeUndefined();
    expect(infoSpy).toHaveBeenCalledTimes(2);
    expect(errorSpy).toHaveBeenCalledTimes(1);
    expect(warnSpy).toHaveBeenCalledTimes(1);
  });
});
