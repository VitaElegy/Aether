import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';

// ===== PRKB-01: Feed with diagnostics =====
export interface Feed {
    id: string;
    name: string;
    url: string;
    feed_type: string;
    enabled: boolean;
    last_fetched_at: string | null;
    health_status: string;      // "healthy", "degraded", "error", "unknown"
    total_fetched: number;
    parse_errors: number;
    last_error: string | null;
}

// ===== PRKB-02: Inbox triage =====
export interface InboxItem {
    id: string;
    feed_id: string;
    external_id: string;
    title: string;
    authors: string[];
    abstract_text: string;
    url: string;
    pdf_url: string | null | undefined;
    publish_date: string;
    is_read: boolean;
    is_saved: boolean;
    fetched_at: string;
    publication?: string;
    state: string;          // "new", "read", "saved", "skipped", "trashed"
    priority: number | null;
    note: string | null;
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

// ===== PRKB-07: Enhanced Signals =====
export interface Signals {
    citation_count: number;
    github_stars: number;
    sota_rank?: string;
    last_updated: string;
    feed_freshness?: string;
    venue_tier?: string;
    author_recurrence?: number;
    custom_importance?: number;
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

// ===== PRKB-03/06: Paper with PDF lifecycle =====
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
    pdf_status: string;     // "not_attached", "queued", "downloaded", "indexed", "failed"
    notes?: string;
}

// ===== PRKB-05: Collections =====
export interface Collection {
    id: string;
    name: string;
    collection_type: string;    // "watchlist", "reading_queue", "archive", "topic_collection"
    description?: string;
    paper_count: number;
    created_at: string;
    updated_at: string;
}

export interface FetchProgress {
    active: boolean;
    current: number;
    total: number;
    currentFeedName: string;
    results: { success: number; errors: number; newItems: number };
}

// ===== PRKB-08: Import result =====
export interface ImportResult {
    imported: number;
    duplicates: number;
    errors: number;
    details: string[];
}

export const usePrkbStore = defineStore('prkb', () => {
    // --- State ---
    const feeds = ref<Feed[]>([]);
    const inbox = ref<InboxItem[]>([]);
    const library = ref<Paper[]>([]);
    const collections = ref<Collection[]>([]);
    const loading = ref(false);
    const loadingFeeds = ref(new Set<string>());
    const selectedFeeds = ref(new Set<string>());
    const selectedPaper = ref<Paper | null>(null);
    const drawerVisible = ref(false);

    const fetchProgress = ref<FetchProgress>({
        active: false,
        current: 0,
        total: 0,
        currentFeedName: '',
        results: { success: 0, errors: 0, newItems: 0 }
    });

    const inboxTotalCount = ref(0);
    const publications = ref<string[]>([]);
    const venues = ref<Venue[]>([]);
    const searchQuery = ref('');

    // --- Computed ---
    const enabledFeeds = computed(() => feeds.value.filter(f => f.enabled));
    const healthyFeeds = computed(() => feeds.value.filter(f => f.health_status === 'healthy'));
    const errorFeeds = computed(() => feeds.value.filter(f => f.health_status === 'error'));

    // ===== PRKB-01: Feed Control Center =====
    const fetchFeeds = async () => {
        try {
            const res = await axios.get('/api/prkb/feeds');
            feeds.value = res.data;
        } catch (e) {
            console.error(e);
        }
    };

    const createFeed = async (name: string, url: string, type_: string) => {
        try {
            await axios.post('/api/prkb/feeds', { name, url, feed_type: type_ });
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

    const toggleFeedEnabled = async (id: string, enabled: boolean) => {
        try {
            await axios.patch(`/api/prkb/feeds/${id}`, { enabled });
            const feed = feeds.value.find(f => f.id === id);
            if (feed) feed.enabled = enabled;
            MessagePlugin.success(enabled ? 'Feed enabled' : 'Feed disabled');
        } catch (e) {
            MessagePlugin.error('Failed to update feed');
        }
    };

    const testFeedParser = async (id: string) => {
        try {
            const res = await axios.post(`/api/prkb/feeds/${id}/test`);
            if (res.data.status === 'ok') {
                MessagePlugin.success(`Parser OK - ${res.data.sample_count} sample items`);
            } else {
                MessagePlugin.error(`Parser error: ${res.data.message}`);
            }
            return res.data;
        } catch (e) {
            MessagePlugin.error('Test failed');
            return null;
        }
    };

    const toggleFeedSelection = (feedId: string) => {
        if (selectedFeeds.value.has(feedId)) {
            selectedFeeds.value.delete(feedId);
        } else {
            selectedFeeds.value.add(feedId);
        }
    };

    const selectAllFeeds = (forceValue?: boolean) => {
        if (forceValue === true || (forceValue === undefined && selectedFeeds.value.size < feeds.value.length)) {
            feeds.value.forEach(f => selectedFeeds.value.add(f.id));
        } else {
            selectedFeeds.value.clear();
        }
    };

    const refreshFeeds = async (feedId?: string) => {
        if (loading.value || fetchProgress.value.active) return;
        loading.value = true;

        let targetFeeds: Feed[] = [];
        if (feedId) {
            targetFeeds = feeds.value.filter(f => f.id === feedId);
        } else if (selectedFeeds.value.size > 0) {
            targetFeeds = feeds.value.filter(f => selectedFeeds.value.has(f.id));
        } else {
            targetFeeds = [...feeds.value];
        }

        if (targetFeeds.length === 0) {
            loading.value = false;
            MessagePlugin.info('No feeds to refresh.');
            return;
        }

        fetchProgress.value = {
            active: true,
            current: 0,
            total: targetFeeds.length,
            currentFeedName: '',
            results: { success: 0, errors: 0, newItems: 0 }
        };

        for (const feed of targetFeeds) {
            fetchProgress.value.current++;
            fetchProgress.value.currentFeedName = feed.name;
            loadingFeeds.value.add(feed.id);

            try {
                const res = await axios.post('/api/prkb/fetch', { feed_id: feed.id });
                const stats = res.data;

                if (stats.details && stats.details.length > 0) {
                    const detail = stats.details[0];
                    if (detail.status === 'ok') {
                        fetchProgress.value.results.success++;
                        fetchProgress.value.results.newItems += detail.count;
                    } else {
                        fetchProgress.value.results.errors++;
                    }
                } else if (stats.total_count >= 0) {
                    fetchProgress.value.results.success++;
                    fetchProgress.value.results.newItems += stats.total_count;
                }
            } catch (e) {
                console.error(`Failed to fetch feed: ${feed.name}`, e);
                fetchProgress.value.results.errors++;
            } finally {
                loadingFeeds.value.delete(feed.id);
            }
        }

        loading.value = false;
        fetchProgress.value.active = false;
        fetchProgress.value.currentFeedName = '';

        if (feedId) {
            const { success, newItems } = fetchProgress.value.results;
            if (success > 0) {
                MessagePlugin.success(`Fetched ${newItems} new items from ${targetFeeds[0].name}.`);
            } else {
                MessagePlugin.error(`Failed to connect to ${targetFeeds[0].name}. The server may be blocking requests.`);
            }
        } else {
            const { success, errors, newItems } = fetchProgress.value.results;
            if (errors > 0 && success === 0) {
                MessagePlugin.error(`Failed to refresh target feeds. All ${errors} sources errored.`);
            } else if (errors > 0) {
                MessagePlugin.warning(`Refresh complete. ${newItems} new items. (${success} succeeded, ${errors} failed)`);
            } else {
                MessagePlugin.success(`Refresh complete. ${newItems} new items across ${success} feeds.`);
            }
        }

        fetchInbox();
        fetchFeeds();
    };

    // ===== PRKB-02: Inbox Triage =====
    const fetchInbox = async (unreadOnly = false, publication?: string) => {
        loading.value = true;
        try {
            const params: any = { unread_only: unreadOnly, limit: 100 };
            if (publication) params.publication = publication;

            const res = await axios.get('/api/prkb/inbox', { params });
            if (Array.isArray(res.data)) {
                inbox.value = res.data;
                inboxTotalCount.value = res.data.length;
            } else {
                inbox.value = res.data.items;
                inboxTotalCount.value = res.data.total;
            }
        } finally {
            loading.value = false;
        }
    };

    const markInboxRead = async (id: string) => {
        try {
            await axios.patch(`/api/prkb/inbox/${id}`, { state: 'read', is_read: true });
            const item = inbox.value.find(i => i.id === id);
            if (item) { item.is_read = true; item.state = 'read'; }
        } catch (e) { console.error(e); }
    };

    const skipInboxItem = async (id: string) => {
        try {
            await axios.patch(`/api/prkb/inbox/${id}`, { state: 'skipped' });
            inbox.value = inbox.value.filter(i => i.id !== id);
            MessagePlugin.success('Skipped');
        } catch (e) { MessagePlugin.error('Failed to skip'); }
    };

    const setInboxPriority = async (id: string, priority: number) => {
        try {
            await axios.patch(`/api/prkb/inbox/${id}`, { priority });
            const item = inbox.value.find(i => i.id === id);
            if (item) item.priority = priority;
        } catch (e) { console.error(e); }
    };

    const setInboxNote = async (id: string, note: string) => {
        try {
            await axios.patch(`/api/prkb/inbox/${id}`, { note });
            const item = inbox.value.find(i => i.id === id);
            if (item) item.note = note;
        } catch (e) { console.error(e); }
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
                source: item.publication || 'Unknown',
                publish_date: item.publish_date,
                tags: []
            });
            MessagePlugin.success('Paper saved to Library');
            item.is_saved = true;
            item.state = 'saved';
        } catch (e) {
            MessagePlugin.error('Failed to save paper');
        }
    };

    const trashPaper = async (paper: InboxItem | Paper) => {
        try {
            if ('state' in paper && paper.state !== 'Inbox' && 'saved_at' in paper) {
                await updatePaper(paper.id, { state: 'Trash' });
                library.value = library.value.filter(p => p.id !== paper.id);
            } else {
                await axios.patch(`/api/prkb/inbox/${paper.id}`, { state: 'trashed' });
                inbox.value = inbox.value.filter(i => i.id !== paper.id);
                MessagePlugin.success('Moved to Trash');
            }
        } catch (e) {
            MessagePlugin.error('Failed to trash item');
            console.error(e);
        }
    };

    // ===== PRKB-03: Library & Detail Drawer =====
    const fetchLibrary = async (venueId?: string) => {
        loading.value = true;
        try {
            const params: any = { limit: 100 };
            if (venueId) params.venue_id = venueId;
            if (searchQuery.value) params.q = searchQuery.value;
            const res = await axios.get('/api/prkb/papers', { params });
            library.value = res.data;
        } finally {
            loading.value = false;
        }
    };

    const fetchPaperDetail = async (id: string) => {
        try {
            const res = await axios.get(`/api/prkb/papers/${id}`);
            selectedPaper.value = res.data;
            drawerVisible.value = true;
        } catch (e) {
            MessagePlugin.error('Failed to load paper details');
        }
    };

    const updatePaper = async (id: string, updates: {
        state?: string;
        is_read?: boolean;
        tags?: string[];
        notes?: string;
        pdf_status?: string;
    }) => {
        try {
            await axios.patch(`/api/prkb/papers/${id}`, updates);
            const paper = library.value.find(p => p.id === id);
            if (paper) {
                if (updates.state) paper.state = updates.state;
                if (updates.is_read !== undefined) paper.is_read = updates.is_read;
                if (updates.tags) paper.tags = updates.tags;
                if (updates.notes !== undefined) paper.notes = updates.notes;
                if (updates.pdf_status) paper.pdf_status = updates.pdf_status;
            }
            if (selectedPaper.value?.id === id) {
                Object.assign(selectedPaper.value, updates);
            }
            MessagePlugin.success('Paper updated');
        } catch (e) {
            MessagePlugin.error('Failed to update paper');
            console.error(e);
        }
    };

    // ===== PRKB-04: Search =====
    const fetchPublications = async () => {
        try {
            const res = await axios.get('/api/prkb/publications');
            publications.value = res.data;
        } catch (e) { console.error(e); }
    };

    const fetchVenues = async () => {
        try {
            const res = await axios.get('/api/prkb/venues');
            venues.value = res.data;
        } catch (e) { console.error(e); }
    };

    const searchPapers = async (query: string) => {
        searchQuery.value = query;
        await fetchLibrary();
    };

    // ===== PRKB-05: Collections =====
    const fetchCollections = async () => {
        try {
            const res = await axios.get('/api/prkb/collections');
            collections.value = res.data;
        } catch (e) { console.error(e); }
    };

    const createCollection = async (name: string, type_: string, description?: string) => {
        try {
            const res = await axios.post('/api/prkb/collections', {
                name,
                collection_type: type_,
                description
            });
            MessagePlugin.success('Collection created');
            fetchCollections();
            return res.data.id;
        } catch (e) {
            MessagePlugin.error('Failed to create collection');
            return null;
        }
    };

    const deleteCollection = async (id: string) => {
        try {
            await axios.delete(`/api/prkb/collections/${id}`);
            MessagePlugin.success('Collection deleted');
            fetchCollections();
        } catch (e) {
            MessagePlugin.error('Failed to delete collection');
        }
    };

    const addToCollection = async (collectionId: string, paperId: string) => {
        try {
            await axios.post(`/api/prkb/collections/${collectionId}/papers`, { paper_id: paperId });
            MessagePlugin.success('Added to collection');
            fetchCollections();
        } catch (e) {
            MessagePlugin.error('Failed to add to collection');
        }
    };

    const removeFromCollection = async (collectionId: string, paperId: string) => {
        try {
            await axios.delete(`/api/prkb/collections/${collectionId}/papers/${paperId}`);
            MessagePlugin.success('Removed from collection');
            fetchCollections();
        } catch (e) {
            MessagePlugin.error('Failed to remove from collection');
        }
    };

    const fetchCollectionPapers = async (collectionId: string) => {
        try {
            const res = await axios.get(`/api/prkb/collections/${collectionId}/papers`);
            return res.data as Paper[];
        } catch (e) {
            console.error(e);
            return [];
        }
    };

    // ===== PRKB-06: PDF Lifecycle =====
    const updatePdfStatus = async (paperId: string, status: string, localPath?: string) => {
        await updatePaper(paperId, { pdf_status: status });
    };

    const queuePdfDownload = async (paperId: string) => {
        await updatePdfStatus(paperId, 'queued');
        // In a real implementation, this would trigger a background download job
        MessagePlugin.info('PDF download queued');
    };

    // ===== PRKB-07: Signals =====
    const updateSignals = async (paperId: string, signals: Partial<Signals>) => {
        try {
            await axios.patch(`/api/prkb/papers/${paperId}/signals`, signals);
            const paper = library.value.find(p => p.id === paperId);
            if (paper && paper.signals) {
                Object.assign(paper.signals, signals);
            }
            MessagePlugin.success('Signals updated');
        } catch (e) {
            MessagePlugin.error('Failed to update signals');
        }
    };

    // ===== PRKB-08: Portability =====
    const exportPapers = async (format: string, collectionId?: string, paperIds?: string[]) => {
        try {
            const res = await axios.post('/api/prkb/export', {
                format,
                collection_id: collectionId,
                paper_ids: paperIds
            });

            if (format === 'json') {
                const blob = new Blob([JSON.stringify(res.data, null, 2)], { type: 'application/json' });
                downloadBlob(blob, `prkb_export_${Date.now()}.json`);
            } else if (format === 'bibtex') {
                const blob = new Blob([res.data], { type: 'text/plain' });
                downloadBlob(blob, `prkb_export_${Date.now()}.bib`);
            } else if (format === 'markdown') {
                const blob = new Blob([res.data], { type: 'text/markdown' });
                downloadBlob(blob, `prkb_digest_${Date.now()}.md`);
            }
            MessagePlugin.success('Export complete');
        } catch (e) {
            MessagePlugin.error('Export failed');
        }
    };

    const importBibtex = async (bibtex: string, mergeTags = true, mergeNotes = true): Promise<ImportResult | null> => {
        try {
            const res = await axios.post('/api/prkb/import/bibtex', {
                bibtex,
                merge_tags: mergeTags,
                merge_notes: mergeNotes
            });
            const result = res.data as ImportResult;
            MessagePlugin.success(`Imported ${result.imported} papers, ${result.duplicates} duplicates`);
            fetchLibrary();
            return result;
        } catch (e) {
            MessagePlugin.error('Import failed');
            return null;
        }
    };

    // Helper
    const downloadBlob = (blob: Blob, filename: string) => {
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();
        URL.revokeObjectURL(url);
    };

    return {
        // State
        feeds,
        inbox,
        library,
        collections,
        loading,
        loadingFeeds,
        fetchProgress,
        selectedFeeds,
        selectedPaper,
        drawerVisible,
        inboxTotalCount,
        publications,
        venues,
        searchQuery,

        // Computed
        enabledFeeds,
        healthyFeeds,
        errorFeeds,

        // PRKB-01: Feeds
        fetchFeeds,
        createFeed,
        deleteFeed,
        toggleFeedEnabled,
        testFeedParser,
        toggleFeedSelection,
        selectAllFeeds,
        refreshFeeds,

        // PRKB-02: Inbox
        fetchInbox,
        markInboxRead,
        skipInboxItem,
        setInboxPriority,
        setInboxNote,
        savePaper,
        trashPaper,

        // PRKB-03: Library
        fetchLibrary,
        fetchPaperDetail,
        updatePaper,

        // PRKB-04: Search
        fetchPublications,
        fetchVenues,
        searchPapers,

        // PRKB-05: Collections
        fetchCollections,
        createCollection,
        deleteCollection,
        addToCollection,
        removeFromCollection,
        fetchCollectionPapers,

        // PRKB-06: PDF
        updatePdfStatus,
        queuePdfDownload,

        // PRKB-07: Signals
        updateSignals,

        // PRKB-08: Portability
        exportPapers,
        importBibtex,
    };
});
