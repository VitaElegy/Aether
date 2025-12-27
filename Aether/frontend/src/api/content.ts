import axios from 'axios';

export interface ContentVersionSnapshot {
    id: string;
    version: number;
    title: string;
    created_at: string;
    reason?: string;
    editor_id: string;
}

export interface ContentDiff {
    old_version: number;
    new_version: number;
    changes: string; // Unified Diff format
}

export const contentApi = {
    getHistory: async (id: string): Promise<ContentVersionSnapshot[]> => {
        const res = await axios.get(`/api/content/${id}/history`);
        return res.data;
    },

    getVersion: async (id: string, version: number): Promise<any> => {
        const res = await axios.get(`/api/content/${id}/version/${version}`);
        // The endpoint returns raw JSON body of the article (ContentBody format usually)
        return res.data;
    },

    getDiff: async (id: string, v1: number, v2: number): Promise<ContentDiff> => {
        const res = await axios.get(`/api/content/${id}/diff/${v1}/${v2}`);
        return res.data;
    }
};
