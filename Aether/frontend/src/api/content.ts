import axios from 'axios';

export interface ContentVersionSnapshot {
    id: string;
    version: string;
    title: string;
    created_at: string;
    reason?: string;
    editor_id: string;
}

export interface DiffChange {
    tag: 'Equal' | 'Insert' | 'Delete';
    value: string;
}

export interface ContentDiff {
    old_version: string;
    new_version: string;
    changes: DiffChange[];
}


export interface Content {
    id: string;
    title: string;
    slug: string;
    body: any; // ContentBody
    status: 'Draft' | 'Published' | 'Archived';
    visibility: 'Public' | 'Private' | 'Internal';
    category?: string;
    tags: string[];
    author_id: string;
    author_name?: string;
    knowledge_base_id?: string;
    parent_id?: string;
    type: 'Article' | 'Folder';
    created_at: string;
    updated_at: string;
}

export interface CreateContentPayload {
    title: string;
    body: string;
    tags: string[];
    category?: string;
    visibility: 'Public' | 'Private' | 'Internal';
    status?: 'Draft' | 'Published' | 'Archived';
    reason?: string;
    snapshot?: boolean;
    knowledge_base_id?: string;
    parent_id?: string;
    type?: 'Article' | 'Folder';
}

export const contentApi = {
    list: async (params?: { offset?: number; limit?: number; author_id?: string; knowledge_base_id?: string }): Promise<Content[]> => {
        const res = await axios.get('/api/content', { params });
        return res.data;
    },

    get: async (id: string): Promise<Content> => {
        const res = await axios.get(`/api/content/${id}`);
        return res.data;
    },

    create: async (payload: CreateContentPayload): Promise<{ id: string }> => {
        const res = await axios.post('/api/content', payload);
        return res.data;
    },

    update: async (id: string, payload: CreateContentPayload): Promise<{ id: string }> => {
        const res = await axios.put(`/api/content/${id}`, payload);
        return res.data;
    },

    delete: async (id: string): Promise<void> => {
        await axios.delete(`/api/content/${id}`);
    },

    getHistory: async (id: string): Promise<ContentVersionSnapshot[]> => {
        const res = await axios.get(`/api/content/${id}/history`);
        return res.data;
    },

    getVersion: async (id: string, version: string): Promise<any> => {
        const res = await axios.get(`/api/content/${id}/history/${version}`);
        return res.data;
    },

    getDiff: async (id: string, v1: string, v2: string): Promise<ContentDiff> => {
        const res = await axios.get(`/api/content/${id}/diff/${v1}/${v2}`);
        return res.data;
    }
};
