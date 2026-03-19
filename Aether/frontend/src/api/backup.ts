import axios from 'axios';

export interface BackupFile {
    filename: string;
    kb_id: string;
    timestamp: string;
    size: number;
}

export interface ImportSection {
    name: string;
    count: number;
    action: string;
}

export interface ImportSummary {
    total_items: number;
    sections: ImportSection[];
    conflicts: string[];
}

export const backupApi = {
    list: async (): Promise<string[]> => {
        const res = await axios.get('/api/backups', {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
        return res.data;
    },

    create: async (kbId: string): Promise<{ filename: string }> => {
        const res = await axios.post('/api/backups', { kb_id: kbId }, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
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
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
        return res.data;
    },

    preview: async (file: File): Promise<ImportSummary> => {
        const formData = new FormData();
        formData.append('file', file);
        const res = await axios.post('/api/backups/preview', formData, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
        return res.data;
    }
};
