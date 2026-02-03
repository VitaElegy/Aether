import axios from 'axios';

export interface GitCommit {
    hash: string;
    short_hash: string;
    author: string;
    date: string;
    message: string;
}

export const systemApi = {
    getGitLog: async () => {
        const res = await axios.get<GitCommit[]>('/api/system/git-log');
        return res;
    },
};
