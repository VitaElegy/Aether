import { ref, computed } from 'vue';
import axios from 'axios';
import { useAuthStore } from '@/stores/auth';
import type { KnowledgeBase } from '@/api/knowledge';

export interface ContentItem {
    id: string;
    title: string;
    body: string | { type: string; data: string }; // Handle both string (Legacy) and Object (New)
    category: string;
    tags: string[];
    status: 'Draft' | 'Published' | 'Archived';
    visibility: 'Public' | 'Internal' | 'Private';
    author_id: string;
    author_name: string;
    author_avatar?: string;
    created_at: number; // Timestamp
    updated_at: number; // Timestamp
    knowledge_base_id: string | null;
    parent_id: string | null;
    slug?: string;
    type?: 'Article' | 'Folder'; // Add Type
}

export function useContent() {
    const authStore = useAuthStore();

    const article = ref<ContentItem | null>(null);
    const loading = ref(false);
    const error = ref<string | null>(null);
    const isSaving = ref(false);

    const isAuthor = computed(() => {
        return authStore.user && article.value && authStore.user.id === article.value.author_id;
    });

    const load = async (id: string) => {
        loading.value = true;
        error.value = null;
        try {
            const res = await axios.get(`/api/content/${id}`);
            const data = res.data;

            // Normalize body
            let normalizedBody = data.body;
            if (typeof data.body === 'string') {
                // If purely string, treat as markdown content
                normalizedBody = data.body;
            } else if (data.body && data.body.type === 'Markdown' && typeof data.body.data === 'string') {
                // Unwrap specific structure if needed, or keep object
                // For Editor, we usually want raw string if it's markdown.
                normalizedBody = data.body.data;
            }

            article.value = {
                ...data,
                body: normalizedBody, // Normalized for simpler usage in Editor
                // Ensure status/visibility defaults if missing (defensive)
                status: data.status || 'Draft',
                visibility: data.visibility || 'Public',
            };
        } catch (e: any) {
            console.error("Failed to load content", e);
            error.value = e.response?.data?.error || e.message || "Failed to load content";
            article.value = null;
        } finally {
            loading.value = false;
        }
    };

    const save = async (payload: Partial<ContentItem> & { reason?: string }) => {
        if (!article.value?.id) return;
        isSaving.value = true;
        error.value = null;

        try {
            // Construct strict payload to avoid missing field issues
            const updateData = {
                title: payload.title ?? article.value.title,
                body: payload.body ?? (typeof article.value.body === 'string' ? article.value.body : ''),
                tags: payload.tags ?? article.value.tags,
                category: payload.category ?? article.value.category,
                visibility: payload.visibility ?? article.value.visibility,
                status: payload.status ?? article.value.status, // IMPORTANT: Must create explicit key
                reason: payload.reason,
                knowledge_base_id: payload.knowledge_base_id ?? article.value.knowledge_base_id,
                parent_id: payload.parent_id ?? article.value.parent_id
            };

            await axios.put(`/api/content/${article.value.id}`, updateData);

            // Update local state if successful
            article.value = { ...article.value, ...updateData, body: updateData.body as any }; // Cast for simplicity
        } catch (e: any) {
            console.error("Failed to save content", e);
            error.value = e.response?.data?.error || "Failed to save";
            throw e; // Re-throw to let UI handle specific failure feedback
        } finally {
            isSaving.value = false;
        }
    };
    const create = async (payload: Partial<ContentItem>) => {
        isSaving.value = true;
        error.value = null;
        try {
            const createData = {
                title: payload.title || 'Untitled',
                body: payload.body || '',
                tags: payload.tags || [],
                category: payload.category || '',
                visibility: payload.visibility || 'Public',
                status: payload.status || 'Draft',
                reason: (payload as any).reason, // Optional
                knowledge_base_id: payload.knowledge_base_id || null,
                parent_id: payload.parent_id || null
            };

            const res = await axios.post('/api/content', createData);

            // Immediately sync state with new article
            const data = res.data;
            // The API returns { id: ... } usually, or full object? 
            // Based on backend code: (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response()
            // So we only get ID. We need to set ID and data locally or reload.
            // For safety/consistency, we can just set the ID and data we sent, or better:
            // Let's return the ID so the caller can set draftId, but also populate article.value tentatively

            // Wait, standard practice: we just got ID.
            const newId = data.id;

            // Optimistic / "Good Enough" update before full reload
            article.value = {
                ...createData,
                id: newId,
                author_id: authStore.user?.id || '',
                author_name: authStore.user?.username || '',
                created_at: Date.now(),
                updated_at: Date.now(),
            } as ContentItem;

            return newId;
        } catch (e: any) {
            console.error("Failed to create content", e);
            error.value = e.response?.data?.error || "Failed to create";
            throw e;
        } finally {
            isSaving.value = false;
        }
    };

    return {
        article,
        loading,
        error,
        isSaving,
        isAuthor,
        load,
        save,
        create
    };
}
