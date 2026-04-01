import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import axios from 'axios';

// ──────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────

export interface LinkedEntity {
    target_id: string;
    target_type: string; // "article" | "asset" | "paper" | "finding" | "doc" | "memo"
    target_title: string;
    anchor_text?: string;
}

export interface Memo {
    id: string;
    title: string;
    created_at: string;
    updated_at: string;
    content: string;
    priority: string;
    status: string;
    color: string;
    is_pinned: boolean;
    due_at?: string;
    reminder_at?: string;
    tags: string[];
    // MEMO-01
    channel?: string;
    excerpt?: string;
    // MEMO-05
    linked_entities: LinkedEntity[];
    // MEMO-06
    scheduled_at?: string;
    snoozed_until?: string;
    reviewed_at?: string;
}

export interface CreateMemoPayload {
    title: string;
    content: string;
    tags: string[];
    visibility: string;
    priority?: string;
    status?: string;
    color?: string;
    is_pinned?: boolean;
    due_at?: string;
    reminder_at?: string;
    channel?: string;
    linked_entities?: LinkedEntity[];
    scheduled_at?: string;
    snoozed_until?: string;
}

export interface UpdateMemoPayload {
    title?: string;
    content?: string;
    tags?: string[];
    visibility?: string;
    priority?: string;
    status?: string;
    color?: string;
    is_pinned?: boolean;
    due_at?: string;
    reminder_at?: string;
    channel?: string;
    linked_entities?: LinkedEntity[];
    scheduled_at?: string;
    snoozed_until?: string;
    reviewed_at?: string;
}

// MEMO-03: Saved View
export interface SavedViewFilters {
    tags: string[];
    channel?: string;
    status?: string;
    priority?: string;
    search?: string;
    date_from?: string;
    date_to?: string;
    is_pinned?: boolean;
    queue?: string;
}

export interface SavedView {
    id: string;
    name: string;
    icon?: string;
    filters: SavedViewFilters;
    sort_by?: string;
    sort_dir?: string;
    view_mode?: string;
    pinned: boolean;
    position: number;
    created_at: string;
    updated_at: string;
}

// MEMO-04: Bulk Update
export interface BulkUpdatePayload {
    status?: string;
    tags_add?: string[];
    tags_remove?: string[];
    channel?: string;
    is_pinned?: boolean;
    priority?: string;
    snoozed_until?: string;
}

export type ViewMode = 'stream' | 'masonry' | 'kanban' | 'calendar' | 'list' | 'timeline';
export type ReviewQueue = 'due_today' | 'overdue' | 'stale' | 'unresolved' | null;

export const useMemosStore = defineStore('memos', () => {
    // ──────────────────────────────────────────────
    // State
    // ──────────────────────────────────────────────
    const memos = ref<Memo[]>([]);
    const loading = ref(false);
    const error = ref<string | null>(null);
    const currentView = ref<ViewMode>('stream');

    // Filters
    const searchQuery = ref('');
    const filterTags = ref<string[]>([]);
    const filterChannel = ref<string | null>(null);
    const filterStatus = ref<string | null>(null);
    const filterPriority = ref<string | null>(null);
    const pinnedTags = ref<string[]>([]);
    const filterProject = ref<string | null>(null);

    // MEMO-06: Active review queue
    const activeQueue = ref<ReviewQueue>(null);

    // Workflow
    const workflow = ref<string[]>(['Todo', 'Doing', 'Done']);

    // MEMO-03: Saved Views
    const savedViews = ref<SavedView[]>([]);
    const activeViewId = ref<string | null>(null);

    // MEMO-01: Channels
    const channels = computed(() => {
        const chs = new Set<string>();
        memos.value.forEach(m => {
            if (m.channel) chs.add(m.channel);
        });
        return Array.from(chs).sort();
    });

    // ──────────────────────────────────────────────
    // Getters
    // ──────────────────────────────────────────────
    const filteredMemos = computed(() => {
        let list = memos.value;

        // Exclude snoozed memos unless explicitly viewing them
        if (activeQueue.value !== 'unresolved') {
            const now = new Date().toISOString();
            list = list.filter(m => !m.snoozed_until || m.snoozed_until <= now);
        }

        if (searchQuery.value) {
            const q = searchQuery.value.toLowerCase();
            list = list.filter(m =>
                m.title.toLowerCase().includes(q) ||
                m.content.toLowerCase().includes(q) ||
                m.tags.some(t => t.toLowerCase().includes(q))
            );
        }
        if (filterTags.value.length > 0) {
            if (filterTags.value.includes('__untagged__')) {
                list = list.filter(m => m.tags.length === 0);
            } else {
                list = list.filter(m => filterTags.value.every(t => m.tags.includes(t)));
            }
        }
        if (filterChannel.value) {
            list = list.filter(m => m.channel === filterChannel.value);
        }
        if (filterStatus.value) {
            list = list.filter(m => m.status === filterStatus.value);
        }
        if (filterPriority.value) {
            list = list.filter(m => m.priority === filterPriority.value);
        }

        // Sort: pinned first, then by date desc
        return list.sort((a, b) => {
            if (a.is_pinned && !b.is_pinned) return -1;
            if (!a.is_pinned && b.is_pinned) return 1;
            return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
        });
    });

    const untaggedCount = computed(() =>
        memos.value.filter(m => m.tags.length === 0).length
    );

    const uniqueTags = computed(() => {
        const counts = new Map<string, number>();
        memos.value.forEach(m => {
            m.tags.forEach(t => counts.set(t, (counts.get(t) || 0) + 1));
        });
        return Array.from(counts.entries())
            .map(([name, count]) => ({ name, count }))
            .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
    });

    const kanbanColumns = computed(() => {
        const cols: Record<string, Memo[]> = {};
        workflow.value.forEach(status => { cols[status] = []; });
        filteredMemos.value.forEach(m => {
            if (cols[m.status]) {
                cols[m.status].push(m);
            } else {
                const fallback = workflow.value[0] || 'Todo';
                if (!cols[fallback]) cols[fallback] = [];
                cols[fallback].push(m);
            }
        });
        return cols;
    });

    // MEMO-06: Review queue counts
    const reviewCounts = computed(() => {
        const now = new Date();
        const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
        const todayEnd = new Date(todayStart.getTime() + 86400000);
        const staleThreshold = new Date(now.getTime() - 7 * 86400000);

        let dueToday = 0, overdue = 0, stale = 0, unresolved = 0;
        for (const m of memos.value) {
            if (m.status === 'Done' || m.status === 'Archived') continue;
            if (m.due_at) {
                const d = new Date(m.due_at);
                if (d >= todayStart && d < todayEnd) dueToday++;
                if (d < now) overdue++;
            }
            if (new Date(m.updated_at) < staleThreshold) stale++;
            if (m.snoozed_until && new Date(m.snoozed_until) <= now) unresolved++;
        }
        return { dueToday, overdue, stale, unresolved };
    });

    // ──────────────────────────────────────────────
    // Data Fetching
    // ──────────────────────────────────────────────
    async function fetchMemos(queue?: ReviewQueue) {
        loading.value = true;
        try {
            await fetchWorkflow();
            const params: Record<string, string> = {};
            if (queue) params.queue = queue;
            const res = await axios.get('/api/memos', { params });
            memos.value = res.data;
        } catch (e: any) {
            error.value = e.message || 'Failed to fetch memos';
        } finally {
            loading.value = false;
        }
    }

    async function fetchWorkflow() {
        try {
            const res = await axios.get('/api/memos/workflow');
            if (res.data?.columns) workflow.value = res.data.columns;
        } catch (e) {
            console.warn('Failed to fetch workflow, using default', e);
        }
    }

    async function saveWorkflow(columns: string[]) {
        workflow.value = columns;
        try {
            await axios.put('/api/memos/workflow', { columns });
        } catch (e) {
            console.error('Failed to save workflow', e);
        }
    }

    // ──────────────────────────────────────────────
    // User Settings
    // ──────────────────────────────────────────────
    async function fetchUserSettings() {
        try {
            const res = await axios.get('/api/users/settings/memos');
            if (res.data) {
                if (res.data.pinned_tags) pinnedTags.value = res.data.pinned_tags;
                if (res.data.view_mode) currentView.value = res.data.view_mode;
            }
        } catch (e) {
            console.warn('Failed to fetch memo settings', e);
        }
    }

    async function saveUserSettings() {
        try {
            await axios.put('/api/users/settings/memos', {
                pinned_tags: pinnedTags.value,
                view_mode: currentView.value
            });
        } catch (e) {
            console.error('Failed to save memo settings', e);
        }
    }

    async function togglePinTag(tag: string) {
        if (pinnedTags.value.includes(tag)) {
            pinnedTags.value = pinnedTags.value.filter(t => t !== tag);
        } else {
            pinnedTags.value.push(tag);
        }
        await saveUserSettings();
    }

    // ──────────────────────────────────────────────
    // CRUD
    // ──────────────────────────────────────────────
    async function createMemo(payload: CreateMemoPayload) {
        try {
            const res = await axios.post('/api/memos', payload);
            await fetchMemos();
            return res.data;
        } catch (e: any) {
            console.error('Create memo failed', e);
            throw e;
        }
    }

    async function updateMemo(id: string, payload: UpdateMemoPayload) {
        const idx = memos.value.findIndex(m => m.id === id);
        const original = idx !== -1 ? { ...memos.value[idx] } : null;
        if (idx !== -1) {
            memos.value[idx] = { ...memos.value[idx], ...payload } as Memo;
        }
        try {
            await axios.put(`/api/memos/${id}`, payload);
        } catch (e) {
            if (original && idx !== -1) memos.value[idx] = original as Memo;
            console.error('Update memo failed', e);
        }
    }

    async function deleteMemo(id: string) {
        const idx = memos.value.findIndex(m => m.id === id);
        if (idx !== -1) memos.value.splice(idx, 1);
        try {
            await axios.delete(`/api/memos/${id}`);
        } catch (e) {
            await fetchMemos();
        }
    }

    async function moveMemoToStatus(id: string, newStatus: string) {
        await updateMemo(id, { status: newStatus });
    }

    // ──────────────────────────────────────────────
    // MEMO-01: Quick Actions
    // ──────────────────────────────────────────────
    async function quickAction(id: string, action: string, snoozeUntil?: string) {
        try {
            await axios.post(`/api/memos/${id}/action`, {
                action,
                snooze_until: snoozeUntil
            });
            await fetchMemos();
        } catch (e) {
            console.error('Quick action failed', e);
        }
    }

    async function archiveMemo(id: string) { return quickAction(id, 'archive'); }
    async function pinMemo(id: string) { return quickAction(id, 'pin'); }
    async function unpinMemo(id: string) { return quickAction(id, 'unpin'); }
    async function snoozeMemo(id: string, until?: string) { return quickAction(id, 'snooze', until); }
    async function convertToTask(id: string) { return quickAction(id, 'convert_task'); }
    async function convertToNote(id: string) { return quickAction(id, 'convert_note'); }

    // ──────────────────────────────────────────────
    // MEMO-03: Saved Views
    // ──────────────────────────────────────────────
    async function fetchSavedViews() {
        try {
            const res = await axios.get('/api/memos/views');
            savedViews.value = res.data;
        } catch (e) {
            console.warn('Failed to fetch saved views', e);
        }
    }

    async function createSavedView(view: {
        name: string;
        icon?: string;
        filters: SavedViewFilters;
        sort_by?: string;
        sort_dir?: string;
        view_mode?: string;
        pinned?: boolean;
    }) {
        try {
            await axios.post('/api/memos/views', view);
            await fetchSavedViews();
        } catch (e) {
            console.error('Failed to create saved view', e);
        }
    }

    async function deleteSavedView(viewId: string) {
        savedViews.value = savedViews.value.filter(v => v.id !== viewId);
        try {
            await axios.delete(`/api/memos/views/${viewId}`);
        } catch (e) {
            console.error('Failed to delete saved view', e);
            await fetchSavedViews();
        }
    }

    function applySavedView(view: SavedView) {
        activeViewId.value = view.id;
        filterTags.value = view.filters.tags || [];
        filterChannel.value = view.filters.channel || null;
        filterStatus.value = view.filters.status || null;
        filterPriority.value = view.filters.priority || null;
        searchQuery.value = view.filters.search || '';
        if (view.view_mode) currentView.value = view.view_mode as ViewMode;
        if (view.filters.queue) {
            activeQueue.value = view.filters.queue as ReviewQueue;
        }
    }

    function saveCurrentAsView(name: string) {
        return createSavedView({
            name,
            filters: {
                tags: [...filterTags.value],
                channel: filterChannel.value || undefined,
                status: filterStatus.value || undefined,
                priority: filterPriority.value || undefined,
                search: searchQuery.value || undefined,
                queue: activeQueue.value || undefined,
            },
            view_mode: currentView.value,
        });
    }

    // ──────────────────────────────────────────────
    // MEMO-04: Bulk Operations
    // ──────────────────────────────────────────────
    async function bulkUpdate(ids: string[], update: BulkUpdatePayload) {
        try {
            const res = await axios.post('/api/memos/bulk-update', { ids, update });
            await fetchMemos();
            return res.data;
        } catch (e) {
            console.error('Bulk update failed', e);
            await fetchMemos();
        }
    }

    async function bulkDelete(ids: string[]) {
        const originals = [...memos.value];
        memos.value = memos.value.filter(m => !ids.includes(m.id));
        try {
            await axios.post('/api/memos/bulk-delete', { ids });
            ui.value.selectedIds.clear();
            ui.value.selectionMode = false;
        } catch (e) {
            console.error('Bulk delete failed', e);
            memos.value = originals;
            await fetchMemos();
        }
    }

    async function mergeMemos(sourceIds: string[], title?: string) {
        try {
            const res = await axios.post('/api/memos/merge', {
                source_ids: sourceIds,
                title
            });
            await fetchMemos();
            return res.data;
        } catch (e) {
            console.error('Merge failed', e);
        }
    }

    async function splitMemo(id: string, splitAt: number, newTitle?: string) {
        try {
            const res = await axios.post(`/api/memos/${id}/split`, {
                split_at: splitAt,
                new_title: newTitle
            });
            await fetchMemos();
            return res.data;
        } catch (e) {
            console.error('Split failed', e);
        }
    }

    // ──────────────────────────────────────────────
    // MEMO-05: Backlinks
    // ──────────────────────────────────────────────
    async function fetchBacklinks(id: string): Promise<Memo[]> {
        try {
            const res = await axios.get(`/api/memos/${id}/backlinks`);
            return res.data;
        } catch (e) {
            console.error('Failed to fetch backlinks', e);
            return [];
        }
    }

    // ──────────────────────────────────────────────
    // MEMO-06: Rhythm & Review
    // ──────────────────────────────────────────────
    async function setActiveQueue(queue: ReviewQueue) {
        activeQueue.value = queue;
        if (queue) {
            await fetchMemos(queue);
        } else {
            await fetchMemos();
        }
    }

    async function markReviewed(id: string) {
        await updateMemo(id, { reviewed_at: new Date().toISOString() });
    }

    // ──────────────────────────────────────────────
    // MEMO-07: Import / Export
    // ──────────────────────────────────────────────
    async function exportMemos(format: string, ids?: string[]) {
        try {
            const res = await axios.post('/api/memos/export', { format, ids });
            return res.data;
        } catch (e) {
            console.error('Export failed', e);
        }
    }

    async function importMemos(items: any[], options?: { mergeTags?: boolean; mergeChannels?: boolean; detectDuplicates?: boolean }) {
        try {
            const res = await axios.post('/api/memos/import', {
                memos: items,
                merge_tags: options?.mergeTags,
                merge_channels: options?.mergeChannels,
                detect_duplicates: options?.detectDuplicates ?? true,
            });
            await fetchMemos();
            return res.data;
        } catch (e) {
            console.error('Import failed', e);
        }
    }

    // ──────────────────────────────────────────────
    // UI State
    // ──────────────────────────────────────────────
    const ui = ref({
        showEditor: false,
        isCreating: false,
        editingMemo: null as Memo | null,
        selectionMode: false,
        selectedIds: new Set<string>(),
        // MEMO-05: Backlink panel
        showBacklinks: false,
        backlinksTargetId: null as string | null,
        backlinksData: [] as Memo[],
        // MEMO-07: Import/Export dialogs
        showExportDialog: false,
        showImportDialog: false,
    });

    function openEditor(memo: Memo | null = null) {
        if (ui.value.selectionMode) return;
        ui.value.editingMemo = memo ? JSON.parse(JSON.stringify(memo)) : null;
        ui.value.isCreating = !memo;
        ui.value.showEditor = true;
    }

    function closeEditor() {
        ui.value.showEditor = false;
        ui.value.editingMemo = null;
        ui.value.isCreating = false;
    }

    // Selection Actions
    function toggleSelectionMode(active: boolean) {
        ui.value.selectionMode = active;
        if (!active) ui.value.selectedIds.clear();
    }

    function toggleSelection(id: string) {
        if (ui.value.selectedIds.has(id)) {
            ui.value.selectedIds.delete(id);
        } else {
            ui.value.selectedIds.add(id);
        }
    }

    function selectAll() {
        filteredMemos.value.forEach(m => ui.value.selectedIds.add(m.id));
    }

    function deselectAll() {
        ui.value.selectedIds.clear();
    }

    // MEMO-05: Open backlink panel
    async function openBacklinks(targetId: string) {
        ui.value.backlinksTargetId = targetId;
        ui.value.showBacklinks = true;
        ui.value.backlinksData = await fetchBacklinks(targetId);
    }

    function closeBacklinks() {
        ui.value.showBacklinks = false;
        ui.value.backlinksTargetId = null;
        ui.value.backlinksData = [];
    }

    // Filter helpers
    function clearAllFilters() {
        filterTags.value = [];
        filterChannel.value = null;
        filterStatus.value = null;
        filterPriority.value = null;
        searchQuery.value = '';
        activeQueue.value = null;
        activeViewId.value = null;
    }

    return {
        // State
        memos, loading, error, currentView,
        searchQuery, filterTags, filterChannel, filterStatus, filterPriority,
        filteredMemos, uniqueTags, untaggedCount, kanbanColumns,
        workflow, ui, pinnedTags, channels,
        // MEMO-03
        savedViews, activeViewId,
        // MEMO-06
        activeQueue, reviewCounts,
        // Data fetching
        fetchMemos, fetchWorkflow, saveWorkflow,
        fetchUserSettings, saveUserSettings, togglePinTag,
        // CRUD
        createMemo, updateMemo, deleteMemo, moveMemoToStatus,
        // MEMO-01: Quick Actions
        quickAction, archiveMemo, pinMemo, unpinMemo, snoozeMemo,
        convertToTask, convertToNote,
        // MEMO-03: Saved Views
        fetchSavedViews, createSavedView, deleteSavedView,
        applySavedView, saveCurrentAsView,
        // MEMO-04: Bulk Ops
        bulkUpdate, bulkDelete, mergeMemos, splitMemo,
        // MEMO-05: Backlinks
        fetchBacklinks, openBacklinks, closeBacklinks,
        // MEMO-06: Review
        setActiveQueue, markReviewed,
        // MEMO-07: Import/Export
        exportMemos, importMemos,
        // UI
        openEditor, closeEditor,
        toggleSelectionMode, toggleSelection, selectAll, deselectAll,
        clearAllFilters,
        filterProject,
    };
});
