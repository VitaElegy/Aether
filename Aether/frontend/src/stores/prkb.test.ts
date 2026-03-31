import { createPinia, setActivePinia } from 'pinia';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';
import { usePrkbStore } from './prkb';

vi.mock('axios', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
    patch: vi.fn(),
  },
}));

vi.mock('tdesign-vue-next', () => ({
  MessagePlugin: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

const axiosMock = vi.mocked(axios, true);

describe('usePrkbStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('toggles individual feed selection and supports select all', () => {
    const store = usePrkbStore();
    store.feeds = [
      { id: 'f1', name: 'Feed 1', url: 'a', feed_type: 'rss', last_fetched_at: null },
      { id: 'f2', name: 'Feed 2', url: 'b', feed_type: 'rss', last_fetched_at: null },
    ];

    store.toggleFeedSelection('f1');
    expect(Array.from(store.selectedFeeds)).toEqual(['f1']);

    store.selectAllFeeds();
    expect(Array.from(store.selectedFeeds).sort()).toEqual(['f1', 'f2']);

    store.selectAllFeeds(false);
    expect(Array.from(store.selectedFeeds)).toEqual([]);
  });

  it('refreshes only selected feeds and reports aggregated success', async () => {
    const store = usePrkbStore();
    store.feeds = [
      { id: 'f1', name: 'Arxiv', url: 'a', feed_type: 'arxiv', last_fetched_at: null },
      { id: 'f2', name: 'DBLP', url: 'b', feed_type: 'rss', last_fetched_at: null },
    ];
    store.selectedFeeds = new Set(['f2']) as any;

    axiosMock.post.mockResolvedValueOnce({
      data: {
        total_count: 3,
        details: [{ feed_name: 'DBLP', count: 3, status: 'ok' }],
      },
    });
    axiosMock.get.mockResolvedValue({ data: { items: [], total: 0 } });

    await store.refreshFeeds();

    expect(axiosMock.post).toHaveBeenCalledTimes(1);
    expect(axiosMock.post).toHaveBeenCalledWith('/api/prkb/fetch', { feed_id: 'f2' });
    expect(MessagePlugin.success).toHaveBeenCalledWith('Refresh complete. 3 new items across 1 feeds.');
    expect(store.fetchProgress.active).toBe(false);
    expect(store.loadingFeeds.size).toBe(0);
  });

  it('shows info feedback when there is nothing to refresh', async () => {
    const store = usePrkbStore();

    await store.refreshFeeds();

    expect(MessagePlugin.info).toHaveBeenCalledWith('No feeds to refresh.');
    expect(axiosMock.post).not.toHaveBeenCalled();
  });
});

