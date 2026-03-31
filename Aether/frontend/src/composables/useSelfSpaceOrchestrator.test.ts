import { createPinia, setActivePinia } from 'pinia';
import { defineComponent } from 'vue';

vi.mock('vue-router', () => ({
  useRouter: () => ({
    replace: vi.fn(),
    push: vi.fn(),
  }),
}));

vi.mock('@/api/knowledge', () => ({
  knowledgeApi: {
    list: vi.fn(),
  },
}));

import { useSelfSpaceOrchestrator } from './useSelfSpaceOrchestrator';
import { usePluginStore } from '@/stores/plugins';
import { usePreferencesStore } from '@/stores/preferences';
import { useAppStateStore } from '@/stores/read_app_state';
import { knowledgeApi } from '@/api/knowledge';
import type { SelfSpacePlugin } from '@/core/plugin';

const knowledgeApiMock = vi.mocked(knowledgeApi, true);

function makePlugin(id: string, order: number): SelfSpacePlugin {
  return {
    id,
    dock: {
      label: id,
      icon: `ri-${id}-line`,
      order,
    },
    capabilities: {
      articleParser: false,
    },
    component: defineComponent({ name: `${id}-component`, template: '<div />' }),
  };
}

describe('useSelfSpaceOrchestrator', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
    knowledgeApiMock.list.mockReset();
  });

  it('initializes dock items and deduplicates singleton renderer apps', async () => {
    knowledgeApiMock.list.mockResolvedValueOnce([
      { id: 'sys-1', title: 'System', renderer_id: 'admin_system' },
      { id: 'memo-1', title: 'Memo A', renderer_id: 'memo' },
      { id: 'memo-2', title: 'Memo B', renderer_id: 'memo' },
    ] as any);

    const pluginStore = usePluginStore();
    pluginStore.registerPlugin(makePlugin('knowledge', 0));
    pluginStore.registerPlugin(makePlugin('admin_system', 1));
    pluginStore.registerPlugin(makePlugin('memo', 2));

    const prefStore = usePreferencesStore();
    prefStore.pinnedKbIds = ['memo-1', 'memo-2'] as any;

    const appStore = useAppStateStore();
    appStore.runningKbIds = new Set(['memo-2']) as any;

    const orchestrator = useSelfSpaceOrchestrator();
    await orchestrator.initialize();

    const memoItems = orchestrator.dockItems.value.filter((item) => item._renderer_id === 'memo');
    expect(memoItems).toHaveLength(1);
    expect(orchestrator.dockItems.value.map((item) => item.id)).toEqual(['library', 'memo-1', 'sys-1']);
  });

  it('opens apps by renderer alias and switches to the concrete kb id', async () => {
    knowledgeApiMock.list.mockResolvedValue([
      { id: 'asset-kb-1', title: 'My Assets', renderer_id: 'assets_v1' },
    ] as any);

    const pluginStore = usePluginStore();
    pluginStore.registerPlugin(makePlugin('knowledge', 0));
    pluginStore.registerPlugin(makePlugin('assets_v1', 1));

    const orchestrator = useSelfSpaceOrchestrator();
    const appStore = useAppStateStore();

    await orchestrator.initialize();
    await orchestrator.switchToKb('assets');

    expect(appStore.activeKbId).toBe('asset-kb-1');
    expect(appStore.runningKbIds.has('asset-kb-1')).toBe(true);
  });
});
