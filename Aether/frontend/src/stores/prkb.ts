import { defineStore } from 'pinia';
import { ref } from 'vue';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';

export interface Feed {
    id: string;
    name: string;
    url: string;
    feed_type: string;
    last_fetched_at: string | null;
}

export interface InboxItem {
    id: string;
    feed_id: string;
    external_id: string; // Added validation
    title: string;
    authors: string[]; // parsed from JSON
    abstract_text: string;
    url: string;
    pdf_url: string | null | undefined;
    publish_date: string;
    is_read: boolean;
    is_saved: boolean;
    fetched_at: string;
}

export interface Paper {
    id: string;
    title: string;
    authors: string[];
    abstract_text: string;
    url: string;
    pdf_url: string | null;
    tags: string[];
    is_read: boolean;
}

export const usePrkbStore = defineStore('prkb', () => {
    const feeds = ref<Feed[]>([]);
    const inbox = ref<InboxItem[]>([]);
    const library = ref<Paper[]>([]);
    const loading = ref(false); // Restored missing state
    const loadingFeeds = ref(new Set<string>());

    // Feeds
    const fetchFeeds = async () => {
        try {
            const res = await axios.get('/api/prkb/feeds');
            feeds.value = res.data;
        } catch (e) {
            console.error(e);
        }
    };

    // ... (createFeed, deleteFeed unchanged)

    // Inbox ...

    const createFeed = async (name: string, url: string, type: string) => {
        try {
            await axios.post('/api/prkb/feeds', { name, url, feed_type: type });
            MessagePlugin.success('Feed added');
            fetchFeeds();
        } catch (e) {
            MessagePlugin.error('Failed to add feed');
        }
    };

    const deleteFeed = async (id: string) => {
        try {
            await axios.delete(`/api/prkb/feeds/${id}`);
            MessagePlugin.success('Feed removed');
            fetchFeeds();
        } catch (e) {
            MessagePlugin.error('Failed to remove feed');
        }
    };

    const fetchInbox = async (unreadOnly = false) => {
        loading.value = true;
        try {
            const res = await axios.get('/api/prkb/inbox', { params: { unread_only: unreadOnly, limit: 100 } });
            inbox.value = res.data;
        } finally {
            loading.value = false;
        }
    };

    const refreshFeeds = async (feedId?: string) => {
        loading.value = true;

        if (feedId) {
            loadingFeeds.value.add(feedId);
        } else {
            // If fetching all, technically all could be marked, but global loading is enough for main button
            // If we want sidebar to spin for all, we could map IDs. 
            // For simplicity, let's rely on global 'loading' for the main button,
            // and 'loadingFeeds' for specific manual triggers.
            // OR: we can mark all as loading if we list them.
            feeds.value.forEach(f => loadingFeeds.value.add(f.id));
        }

        try {
            const res = await axios.post('/api/prkb/fetch', { feed_id: feedId || null });
            const stats = res.data;

            // Build detailed message
            if (stats.details && stats.details.length > 0) {
                const detailsStr = stats.details
                    .filter((d: any) => d.count > 0 || d.status !== 'ok')
                    .map((d: any) => {
                        if (d.status !== 'ok') return `${d.feed_name}: Error`;
                        return `${d.feed_name}: +${d.count}`;
                    })
                    .join(', ');

                if (detailsStr) {
                    MessagePlugin.success(`Fetched ${stats.total_count} items. (${detailsStr})`);
                } else {
                    MessagePlugin.info(`Check completed. No new items.`);
                }
            } else {
                MessagePlugin.success(`Fetched ${stats.total_count} items`);
            }
            fetchInbox();
            fetchFeeds(); // Update last fetched time
        } catch (e) {
            MessagePlugin.error('Failed to refresh feeds');
        } finally {
            loading.value = false;
            if (feedId) {
                loadingFeeds.value.delete(feedId);
            } else {
                loadingFeeds.value.clear();
            }
        }
    };

    const savePaper = async (item: InboxItem) => {
        try {
            await axios.post('/api/prkb/papers', {
                inbox_item_id: item.id,
                title: item.title,
                authors: item.authors,
                abstract_text: item.abstract_text,
                url: item.url,
                pdf_url: item.pdf_url,
                source: 'ArXiv', // Todo: derive from feed
                publish_date: item.publish_date,
                tags: []
            });
            MessagePlugin.success('Paper saved to Library');
            // Optimistic update
            item.is_saved = true;
        } catch (e) {
            MessagePlugin.error('Failed to save paper');
        }
    };

    // Library
    const fetchLibrary = async () => {
        loading.value = true;
        try {
            const res = await axios.get('/api/prkb/papers');
            library.value = res.data;
        } finally {
            loading.value = false;
        }
    };

    return {
        feeds,
        inbox,
        library,
        loading,
        loadingFeeds,
        fetchFeeds,
        createFeed,
        deleteFeed,
        fetchInbox,
        refreshFeeds,
        savePaper,
        fetchLibrary
    };
});
