import axios from 'axios';

export interface BackupFile {
    filename: string;
    kb_id: string;
    timestamp: string;
    size: number;
}

export const backupApi = {
    list: async (): Promise<string[]> => {
        const res = await axios.get('/api/backups');
        return res.data;
    },

    create: async (kbId: string): Promise<{ filename: string }> => {
        const res = await axios.post('/api/backups', { kb_id: kbId });
        return res.data;
    },

    getDownloadUrl: (filename: string) => {
        return `/api/backups/download/${filename}`;
    },

    restore: async (file: File): Promise<{ new_kb_id: string }> => {
        const formData = new FormData();
        formData.append('file', file);
        const res = await axios.post('/api/backups/restore', formData, {
            headers: {
                'Content-Type': 'multipart/form-data'
            }
        });
        return res.data;
    }
};
