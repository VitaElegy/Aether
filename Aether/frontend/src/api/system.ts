import axios from 'axios';

export interface GitCommit {
    hash: string;
    short_hash: string;
    author: string;
    date: string;
    message: string;
}

export interface SystemSettings {
    max_upload_size_mb: number;
    [key: string]: any;
}

export const systemApi = {
    getGitLog: async () => {
        const res = await axios.get<GitCommit[]>('/api/system/git-log');
        return res;
    },
    getSettings: async () => {
        return await axios.get<SystemSettings>('/api/system/settings');
    },
    updateSettings: async (settings: Partial<SystemSettings>) => {
        return await axios.put('/api/system/settings', settings);
    }
};
