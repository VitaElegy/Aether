import axios from 'axios';

const API_BASE = '/api';

export const tagsApi = {
    async list(): Promise<string[]> {
        const response = await axios.get<string[]>(`${API_BASE}/tags`, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        });
        return response.data;
    }
};
