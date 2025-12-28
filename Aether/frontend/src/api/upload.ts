import axios from 'axios';

const API_BASE = '/api';

export const uploadApi = {
    async uploadFile(file: File): Promise<string> {
        const formData = new FormData();
        formData.append('file', file);

        const response = await axios.post<{ url: string }>(`${API_BASE}/upload`, formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
        return response.data.url;
    }
};
