import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import axios from 'axios';

// --- Types should match Backend ---
export interface Memo {
    id: string; // UUID
    // Node
    title: string;
    created_at: string; // ISO
    updated_at: string;

    // Memo
    content: string; // JSON string or raw text (Block-First JSON)
    priority: string; // P0, P1, P2, P3
    status: string; // Todo, Doing, Done, Archived
    color: string; // Yellow, Red, Green, Blue, Purple, Gray
    is_pinned: boolean; // boolean
    due_at?: string; // ISO
    reminder_at?: string; // ISO
    tags: string[];
}

export interface CreateMemoPayload {
    title: string;
    content: string; // Default "{}"
    tags: string[];
    visibility: string; // 'Private', 'Public', 'Internal'
    priority?: string;
    status?: string;
    color?: string;
    is_pinned?: boolean;
    due_at?: string;
    reminder_at?: string;
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
}

export const useMemosStore = defineStore('memos', () => {
    // State
    const memos = ref<Memo[]>([]);
    const loading = ref(false);
    const error = ref<string | null>(null);
    const currentView = ref<'masonry' | 'kanban' | 'calendar' | 'list'>('masonry');

    // Filters
    const searchQuery = ref('');
    const filterTags = ref<string[]>([]);
    const filterProject = ref<string | null>(null); // TODO: Project support

    // Getters
    const filteredMemos = computed(() => {
        let list = memos.value;
        if (searchQuery.value) {
            const q = searchQuery.value.toLowerCase();
            list = list.filter(m =>
                m.title.toLowerCase().includes(q) ||
                m.content.toLowerCase().includes(q)
            );
        }
        if (filterTags.value.length > 0) {
            list = list.filter(m => filterTags.value.every(t => m.tags.includes(t)));
        }
        // Sort by pinned, then date desc
        return list.sort((a, b) => {
            if (a.is_pinned && !b.is_pinned) return -1;
            if (!a.is_pinned && b.is_pinned) return 1;
            return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
        });
    });

    const kanbanColumns = computed(() => {
        const cols = {
            Todo: [] as Memo[],
            Doing: [] as Memo[],
            Done: [] as Memo[],
            Archived: [] as Memo[] // Usually hidden in default view
        };
        filteredMemos.value.forEach(m => {
            if (cols[m.status as keyof typeof cols]) {
                cols[m.status as keyof typeof cols].push(m);
            } else {
                cols['Todo'].push(m); // Fallback
            }
        });
        return cols;
    });

    // Actions
    async function fetchMemos() {
        loading.value = true;
        try {
            const res = await axios.get('/api/memos');
            memos.value = res.data;
        } catch (e: any) {
            error.value = e.message || 'Failed to fetch memos';
        } finally {
            loading.value = false;
        }
    }

    async function createMemo(payload: CreateMemoPayload) {
        try {
            const res = await axios.post('/api/memos', payload);
            // Optimistic add? Need full object from BE or fetch again.
            // Usually simpler to fetch again for ID generation consistency, OR BE returns ID and we construct manual object.
            // Let's refetch for simplicity of "created_at" etc.
            await fetchMemos();
            return res.data; // ID
        } catch (e: any) {
            console.error('Create memo failed', e);
            throw e;
        }
    }

    async function updateMemo(id: string, payload: UpdateMemoPayload) {
        // Optimistic Update
        const idx = memos.value.findIndex(m => m.id === id);
        const original = idx !== -1 ? { ...memos.value[idx] } : null;

        if (idx !== -1) {
            // Apply partial update locally
            memos.value[idx] = { ...memos.value[idx], ...payload } as Memo;
        }

        try {
            await axios.put(`/api/memos/${id}`, payload);
        } catch (e) {
            // Revert
            if (original && idx !== -1) memos.value[idx] = original;
            console.error('Update memo failed', e);
        }
    }

    async function deleteMemo(id: string) {
        const idx = memos.value.findIndex(m => m.id === id);
        if (idx !== -1) memos.value.splice(idx, 1);

        try {
            await axios.delete(`/api/memos/${id}`);
        } catch (e) {
            // Re-fetch to sync state if failed
            await fetchMemos();
        }
    }

    // Drag Helper
    async function moveMemoToStatus(id: string, newStatus: string) {
        await updateMemo(id, { status: newStatus });
    }

    // UI State
    const ui = ref({
        showEditor: false,
        isCreating: false,
        editingMemo: null as Memo | null
    });

    function openEditor(memo: Memo | null = null) {
        ui.value.editingMemo = memo ? JSON.parse(JSON.stringify(memo)) : null;
        ui.value.isCreating = !memo;
        ui.value.showEditor = true;
    }

    function closeEditor() {
        ui.value.showEditor = false;
        ui.value.editingMemo = null;
        ui.value.isCreating = false;
    }

    return {
        memos,
        loading,
        error,
        currentView,
        searchQuery,
        filterTags,
        filteredMemos,
        kanbanColumns,
        ui, // Export UI state
        fetchMemos,
        createMemo,
        updateMemo,
        deleteMemo,
        moveMemoToStatus,
        openEditor,
        closeEditor
    };
});
