import { createPinia, setActivePinia } from 'pinia';
import axios from 'axios';
import { useMemosStore, type Memo } from './memos';

vi.mock('axios', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  },
}));

const axiosMock = vi.mocked(axios, true);

function makeMemo(overrides: Partial<Memo>): Memo {
  return {
    id: crypto.randomUUID(),
    title: 'Untitled',
    created_at: '2026-03-19T12:00:00.000Z',
    updated_at: '2026-03-19T12:00:00.000Z',
    content: 'default body',
    priority: 'P2',
    status: 'Todo',
    color: 'Yellow',
    is_pinned: false,
    tags: [],
    ...overrides,
  };
}

describe('useMemosStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('filters memos by query and tags, then keeps pinned items first', () => {
    const store = useMemosStore();

    store.memos = [
      makeMemo({
        id: '1',
        title: 'Kernel note',
        content: 'memory corruption investigation',
        tags: ['Dev'],
        created_at: '2026-03-18T08:00:00.000Z',
      }),
      makeMemo({
        id: '2',
        title: 'Paper idea',
        content: 'memory hierarchy reading list',
        tags: ['Paper', 'Dev'],
        is_pinned: true,
        created_at: '2026-03-19T08:00:00.000Z',
      }),
      makeMemo({
        id: '3',
        title: 'Life',
        content: 'buy groceries',
        tags: ['Life'],
      }),
    ];

    store.searchQuery = 'memory';
    store.filterTags = ['Dev'];

    expect(store.filteredMemos.map((memo) => memo.id)).toEqual(['2', '1']);
  });

  it('loads user settings into pinned tags and current view', async () => {
    const store = useMemosStore();
    axiosMock.get.mockResolvedValueOnce({
      data: {
        pinned_tags: ['Paper', 'Dev'],
        view_mode: 'kanban',
      },
    });

    await store.fetchUserSettings();

    expect(store.pinnedTags).toEqual(['Paper', 'Dev']);
    expect(store.currentView).toBe('kanban');
  });
});

