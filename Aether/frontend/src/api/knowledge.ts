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
    renderer_id?: string;
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
    renderer_id?: string;
    visibility?: 'Public' | 'Private' | 'Internal';
}

export interface UpdateKnowledgeBaseRequest {
    title?: string;
    description?: string;
    tags?: string[];
    cover_image?: string;
    cover_offset_y?: number;
    renderer_id?: string;
    visibility?: 'Public' | 'Private' | 'Internal';
}

export interface Collaborator {
    user_id: string;
    username: string;
    avatar_url?: string;
    role: string;
}

// Helper to get headers
const getAuthHeaders = () => ({
    headers: {
        'Authorization': `Bearer ${localStorage.getItem('token')}`
    }
});

export const knowledgeApi = {
    async list(): Promise<KnowledgeBase[]> {
        const response = await axios.get(`${API_BASE}/knowledge-bases`, getAuthHeaders());
        // [DEBUG] Inspect Raw Response
        // console.log('[API] Knowledge List Raw:', response.data);
        // Check first item for renderer_id
        // if (response.data.length > 0) {
        //     console.debug('[API] Sample KB Renderer:', response.data[0].title, response.data[0].renderer_id);
        // }
        return response.data;
    },

    async get(id: string): Promise<KnowledgeBase> {
        const response = await axios.get(`${API_BASE}/knowledge-bases/${id}`, getAuthHeaders());
        return response.data;
    },

    async create(payload: CreateKnowledgeBaseRequest): Promise<KnowledgeBase> {
        const response = await axios.post(`${API_BASE}/knowledge-bases`, payload, getAuthHeaders());
        return response.data;
    },

    async update(id: string, payload: UpdateKnowledgeBaseRequest): Promise<void> {
        await axios.put(`${API_BASE}/knowledge-bases/${id}`, payload, getAuthHeaders());
    },

    async delete(id: string): Promise<void> {
        await axios.delete(`${API_BASE}/knowledge-bases/${id}`, getAuthHeaders());
    },

    async listCollaborators(id: string): Promise<Collaborator[]> {
        const response = await axios.get<Collaborator[]>(`${API_BASE}/knowledge-bases/${id}/collaborators`, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
        return response.data;
    },

    async addCollaborator(id: string, userId: string, role: string): Promise<void> {
        await axios.post(`${API_BASE}/knowledge-bases/${id}/collaborators`, { user_id: userId, role }, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
    },

    async removeCollaborator(id: string, userId: string): Promise<void> {
        await axios.delete(`${API_BASE}/knowledge-bases/${id}/collaborators/${userId}`, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
    }
};
