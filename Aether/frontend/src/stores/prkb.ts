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
    publication?: string; // Journal or Venue
}

export interface Author {
    id: string;
    name: string;
    canonical_name?: string;
    profile_url?: string;
}

export interface Venue {
    id: string;
    name: string;
    tier?: string;
}

export interface Signals {
    citation_count: number;
    github_stars: number;
    sota_rank?: string;
    last_updated: string;
}

export interface BibTexInfo {
    publisher?: string;
    editor?: string;
    pages?: string;
    doi?: string;
    isbn?: string;
}

export interface PaperMetadata {
    track?: string;
    series?: string;
    bibtex?: BibTexInfo;
    subjects: string[];
    keywords: string[];
}

export interface Paper {
    id: string;
    title: string;
    authors: Author[];
    abstract_text: string;
    url: string;
    pdf_url: string | null;
    pdf_local_path?: string;
    venue?: Venue;
    publish_date: string;
    arxiv_id?: string;
    source: string;
    saved_at: string;
    is_read: boolean;
    state: string;
    tags: string[];
    signals?: Signals;
    metadata?: PaperMetadata;
}

export const usePrkbStore = defineStore('prkb', () => {
    const feeds = ref<Feed[]>([]);
    const inbox = ref<InboxItem[]>([]);
    const library = ref<Paper[]>([]);
    const loading = ref(false); // Restored missing state
    const loadingFeeds = ref(new Set<string>());

    const inboxTotalCount = ref(0);
    const publications = ref<string[]>([]); // Facets
    const venues = ref<Venue[]>([]);

    // Feeds
    const fetchFeeds = async () => {
        try {
            const res = await axios.get('/api/prkb/feeds');
            feeds.value = res.data;
        } catch (e) {
            console.error(e);
        }
    };

    const fetchPublications = async () => {
        try {
            const res = await axios.get('/api/prkb/publications');
            publications.value = res.data;
        } catch (e) {
            console.error(e);
        }
    };

    const fetchVenues = async () => {
        try {
            const res = await axios.get('/api/prkb/venues');
            venues.value = res.data;
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

    const fetchInbox = async (unreadOnly = false, publication?: string) => {
        loading.value = true;
        try {
            const params: any = { unread_only: unreadOnly, limit: 100 };
            if (publication) {
                params.publication = publication;
            }
            const res = await axios.get('/api/prkb/inbox', { params });
            // Handle both old array format (fallback) and new object format
            if (Array.isArray(res.data)) {
                inbox.value = res.data;
                inboxTotalCount.value = res.data.length; // Approximate fallback
            } else {
                inbox.value = res.data.items;
                inboxTotalCount.value = res.data.total;
            }
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

    const updatePaper = async (id: string, updates: { state?: string; is_read?: boolean }) => {
        try {
            await axios.patch(`/api/prkb/papers/${id}`, updates);

            // Optimistic Update in Library
            const paper = library.value.find(p => p.id === id);
            if (paper) {
                if (updates.state) paper.state = updates.state;
                if (updates.is_read !== undefined) paper.is_read = updates.is_read;
            }
            MessagePlugin.success('Paper updated');
        } catch (e) {
            MessagePlugin.error('Failed to update paper');
            console.error(e);
        }
    };

    const trashPaper = async (paper: InboxItem | Paper) => {
        try {
            // Check if it's an inbox item or library paper
            if ('state' in paper && paper.state !== 'Inbox') {
                // Library Paper
                await updatePaper(paper.id, { state: 'Trash' });
                // Remove from library view locally
                library.value = library.value.filter(p => p.id !== paper.id);
            } else {
                // Inbox Item
                await axios.patch(`/api/prkb/inbox/${paper.id}`, { state: 'Trash' });
                // remove from inbox locally
                inbox.value = inbox.value.filter(i => i.id !== paper.id);
                MessagePlugin.success('Moved to Trash');
            }
        } catch (e) {
            MessagePlugin.error('Failed to trash item');
            console.error(e);
        }
    };

    // Library
    const fetchLibrary = async (venueId?: string) => {
        loading.value = true;
        try {
            const params: any = { limit: 100 };
            if (venueId) {
                params.venue_id = venueId;
            }
            const res = await axios.get('/api/prkb/papers', { params });
            library.value = res.data;
        } finally {
            loading.value = false;
        }
    };

    return {
        feeds,
        inbox,
        inboxTotalCount,
        publications,
        venues,
        library,
        loading,
        loadingFeeds,
        fetchFeeds,
        fetchPublications,
        fetchVenues,
        createFeed,
        deleteFeed,
        fetchInbox,
        refreshFeeds,
        savePaper,
        updatePaper,
        trashPaper,
        fetchLibrary
    };
});
