import axios from 'axios';

export interface DraftData {
    target_article_id?: string | null;
    title?: string;
    body?: string;
    tags?: string[];
    category?: string;
    knowledge_base_id?: string | null;
    parent_id?: string | null;
    updated_at?: string;
}

export const draftApi = {
    async get(): Promise<DraftData | null> {
        try {
            const response = await axios.get('/api/draft');
            if (response.status === 204) return null;
            return response.data;
        } catch (error) {
            console.error('Failed to fetch draft:', error);
            return null;
        }
    },

    async save(data: DraftData): Promise<void> {
        // Backend DTO might not support parent_id yet, but we include it in interface for future proofing
        // We will send it, if backend ignores it, fine.
        await axios.put('/api/draft', data);
    },

    async delete(): Promise<void> {
        await axios.delete('/api/draft');
    }
};
