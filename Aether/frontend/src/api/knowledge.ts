import axios from 'axios';

const API_BASE = '/api';

export interface KnowledgeBase {
    id: string;
    author_id: string;
    title: string;
    description?: string;
    tags?: string[];
    cover_image?: string;
    cover_offset_y: number; // Percentage 0-100
    visibility: 'Public' | 'Private' | 'Internal';
    created_at: string;
    updated_at: string;
}

export interface CreateKnowledgeBaseRequest {
    title: string;
    description?: string;
    tags?: string[];
    cover_image?: string;
    cover_offset_y?: number;
    visibility?: 'Public' | 'Private' | 'Internal';
}

export interface UpdateKnowledgeBaseRequest {
    title?: string;
    description?: string;
    tags?: string[];
    cover_image?: string;
    cover_offset_y?: number;
    visibility?: 'Public' | 'Private' | 'Internal';
}

export const knowledgeApi = {
    async list(): Promise<KnowledgeBase[]> {
        const response = await axios.get(`${API_BASE}/knowledge-bases`);
        return response.data;
    },

    async get(id: string): Promise<KnowledgeBase> {
        const response = await axios.get(`${API_BASE}/knowledge-bases/${id}`);
        return response.data;
    },

    async create(payload: CreateKnowledgeBaseRequest): Promise<KnowledgeBase> {
        const response = await axios.post(`${API_BASE}/knowledge-bases`, payload);
        // The backend returns { id: string }, so we might need to fetch it or construct a partial object.
        // Usually, for a list update, we'd want the full object.
        // Checking backend: create handler returns { id: ... }.
        // For UI responsiveness, we might want to fetch it immediately or return just ID.
        // Let's return the ID response for now or chain a get.
        // Ideally backend returns full object, but current impl returns {id}.
        // Let's just return what the backend returns and let the UI handle fetching or optimistic update.
        return response.data;
    },

    async update(id: string, payload: UpdateKnowledgeBaseRequest): Promise<void> {
        await axios.put(`${API_BASE}/knowledge-bases/${id}`, payload);
    },

    async delete(id: string): Promise<void> {
        await axios.delete(`${API_BASE}/knowledge-bases/${id}`);
    }
};
