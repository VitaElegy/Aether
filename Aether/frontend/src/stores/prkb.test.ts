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
    vi.clearAllMocks();
  });

  // PRKB-01: Feed selection
  it('toggles individual feed selection and supports select all', () => {
    const store = usePrkbStore();
    store.feeds = [
      { id: 'f1', name: 'Feed 1', url: 'a', feed_type: 'rss', enabled: true, last_fetched_at: null, health_status: 'unknown', total_fetched: 0, parse_errors: 0, last_error: null },
      { id: 'f2', name: 'Feed 2', url: 'b', feed_type: 'rss', enabled: true, last_fetched_at: null, health_status: 'unknown', total_fetched: 0, parse_errors: 0, last_error: null },
    ];

    store.toggleFeedSelection('f1');
    expect(Array.from(store.selectedFeeds)).toEqual(['f1']);

    store.selectAllFeeds();
    expect(Array.from(store.selectedFeeds).sort()).toEqual(['f1', 'f2']);

    store.selectAllFeeds(false);
    expect(Array.from(store.selectedFeeds)).toEqual([]);
  });

  // PRKB-01: Feed refresh
  it('refreshes only selected feeds and reports aggregated success', async () => {
    const store = usePrkbStore();
    store.feeds = [
      { id: 'f1', name: 'Arxiv', url: 'a', feed_type: 'arxiv', enabled: true, last_fetched_at: null, health_status: 'healthy', total_fetched: 10, parse_errors: 0, last_error: null },
      { id: 'f2', name: 'DBLP', url: 'b', feed_type: 'rss', enabled: true, last_fetched_at: null, health_status: 'healthy', total_fetched: 5, parse_errors: 0, last_error: null },
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

  // PRKB-01: Toggle feed enabled
  it('toggles feed enabled status', async () => {
    const store = usePrkbStore();
    store.feeds = [
      { id: 'f1', name: 'Feed 1', url: 'a', feed_type: 'rss', enabled: true, last_fetched_at: null, health_status: 'healthy', total_fetched: 0, parse_errors: 0, last_error: null },
    ];
    axiosMock.patch.mockResolvedValueOnce({ data: { status: 'updated' } });

    await store.toggleFeedEnabled('f1', false);

    expect(axiosMock.patch).toHaveBeenCalledWith('/api/prkb/feeds/f1', { enabled: false });
    expect(store.feeds[0].enabled).toBe(false);
    expect(MessagePlugin.success).toHaveBeenCalledWith('Feed disabled');
  });

  // PRKB-02: Inbox triage actions
  it('marks inbox item as read', async () => {
    const store = usePrkbStore();
    store.inbox = [{
      id: 'i1', feed_id: 'f1', external_id: 'e1', title: 'Test Paper',
      authors: ['Author A'], abstract_text: 'Abstract', url: 'http://test.com',
      pdf_url: null, publish_date: '2024-01-01', is_read: false, is_saved: false,
      fetched_at: '2024-01-01', publication: 'cs.AI', state: 'new', priority: null, note: null,
    }];
    axiosMock.patch.mockResolvedValueOnce({ data: { status: 'updated' } });

    await store.markInboxRead('i1');

    expect(axiosMock.patch).toHaveBeenCalledWith('/api/prkb/inbox/i1', { state: 'read', is_read: true });
    expect(store.inbox[0].is_read).toBe(true);
    expect(store.inbox[0].state).toBe('read');
  });

  it('skips inbox item', async () => {
    const store = usePrkbStore();
    store.inbox = [{
      id: 'i1', feed_id: 'f1', external_id: 'e1', title: 'Test',
      authors: [], abstract_text: '', url: '', pdf_url: null,
      publish_date: '2024-01-01', is_read: false, is_saved: false,
      fetched_at: '2024-01-01', state: 'new', priority: null, note: null,
    }];
    axiosMock.patch.mockResolvedValueOnce({ data: { status: 'updated' } });

    await store.skipInboxItem('i1');

    expect(axiosMock.patch).toHaveBeenCalledWith('/api/prkb/inbox/i1', { state: 'skipped' });
    expect(store.inbox.length).toBe(0);
  });

  it('sets inbox item priority', async () => {
    const store = usePrkbStore();
    store.inbox = [{
      id: 'i1', feed_id: 'f1', external_id: 'e1', title: 'Test',
      authors: [], abstract_text: '', url: '', pdf_url: null,
      publish_date: '2024-01-01', is_read: false, is_saved: false,
      fetched_at: '2024-01-01', state: 'new', priority: null, note: null,
    }];
    axiosMock.patch.mockResolvedValueOnce({ data: { status: 'updated' } });

    await store.setInboxPriority('i1', 4);

    expect(axiosMock.patch).toHaveBeenCalledWith('/api/prkb/inbox/i1', { priority: 4 });
    expect(store.inbox[0].priority).toBe(4);
  });

  // PRKB-05: Collections
  it('creates a collection', async () => {
    const store = usePrkbStore();
    axiosMock.post.mockResolvedValueOnce({ data: { id: 'c1' } });
    axiosMock.get.mockResolvedValueOnce({ data: [] });

    const id = await store.createCollection('My Watchlist', 'watchlist', 'desc');

    expect(axiosMock.post).toHaveBeenCalledWith('/api/prkb/collections', {
      name: 'My Watchlist',
      collection_type: 'watchlist',
      description: 'desc',
    });
    expect(id).toBe('c1');
    expect(MessagePlugin.success).toHaveBeenCalledWith('Collection created');
  });

  // PRKB-08: Export
  it('exports papers as bibtex', async () => {
    const store = usePrkbStore();
    axiosMock.post.mockResolvedValueOnce({ data: '@article{test, title={Test}}' });

    // Mock URL/Blob APIs
    const createObjectURLMock = vi.fn(() => 'blob:test');
    const revokeObjectURLMock = vi.fn();
    global.URL.createObjectURL = createObjectURLMock;
    global.URL.revokeObjectURL = revokeObjectURLMock;

    const clickMock = vi.fn();
    vi.spyOn(document, 'createElement').mockReturnValue({ click: clickMock, href: '', download: '' } as any);

    await store.exportPapers('bibtex');

    expect(axiosMock.post).toHaveBeenCalledWith('/api/prkb/export', {
      format: 'bibtex',
      collection_id: undefined,
      paper_ids: undefined,
    });
    expect(MessagePlugin.success).toHaveBeenCalledWith('Export complete');
  });

  // PRKB-08: Import
  it('imports bibtex', async () => {
    const store = usePrkbStore();
    const mockResult = { imported: 2, duplicates: 1, errors: 0, details: ['Imported: Paper A', 'Imported: Paper B', 'Duplicate: Paper C'] };
    axiosMock.post.mockResolvedValueOnce({ data: mockResult });
    axiosMock.get.mockResolvedValue({ data: [] });

    const result = await store.importBibtex('@article{a, title={Paper A}}', true, true);

    expect(axiosMock.post).toHaveBeenCalledWith('/api/prkb/import/bibtex', {
      bibtex: '@article{a, title={Paper A}}',
      merge_tags: true,
      merge_notes: true,
    });
    expect(result).toEqual(mockResult);
    expect(MessagePlugin.success).toHaveBeenCalledWith('Imported 2 papers, 1 duplicates');
  });
});
