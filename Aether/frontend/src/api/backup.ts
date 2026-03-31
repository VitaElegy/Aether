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

export interface BackupApiErrorDetails {
    error: string;
    code?: string;
    details?: string;
    hint?: string;
    stage?: string;
    status?: number;
}

type BackupApiErrorPayload = Partial<BackupApiErrorDetails> & {
    message?: string;
};

const readString = (value: unknown): string | undefined => {
    return typeof value === 'string' && value.trim().length > 0 ? value : undefined;
};

const normalizeErrorPayload = (value: unknown): BackupApiErrorPayload | null => {
    if (!value) {
        return null;
    }

    if (typeof value === 'string') {
        try {
            const parsed = JSON.parse(value);
            return normalizeErrorPayload(parsed);
        } catch {
            return { error: value };
        }
    }

    if (typeof value !== 'object') {
        return null;
    }

    const record = value as Record<string, unknown>;
    return {
        error: readString(record.error),
        message: readString(record.message),
        code: readString(record.code),
        details: readString(record.details),
        hint: readString(record.hint),
        stage: readString(record.stage),
    };
};

export const extractBackupApiError = (
    error: unknown,
    fallbackMessage = 'Backup request failed.'
): BackupApiErrorDetails => {
    const response = (error as { response?: { status?: number; data?: unknown } } | undefined)?.response;
    const payload = normalizeErrorPayload(response?.data);
    const status = response?.status;

    let message = payload?.error || payload?.message;
    if (!message && status === 401) {
        message = 'Authentication failed: you are not authorized to perform this backup action.';
    }

    if (!message) {
        message = readString((error as { message?: unknown } | undefined)?.message) || fallbackMessage;
    }

    return {
        error: message,
        code: payload?.code,
        details: payload?.details,
        hint: payload?.hint,
        stage: payload?.stage,
        status,
    };
};

export const formatBackupApiError = (details: BackupApiErrorDetails): string => {
    const lines = [details.error];

    if (details.details) {
        lines.push(`Details: ${details.details}`);
    }

    if (details.hint) {
        lines.push(`Hint: ${details.hint}`);
    }

    if (details.code) {
        lines.push(`Code: ${details.code}`);
    }

    if (details.status) {
        lines.push(`HTTP: ${details.status}`);
    }

    return lines.join('\n');
};

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
