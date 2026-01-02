import axios from 'axios';

const API_URL = '/api/memos';

export const memoApi = {
    list: async (params?: { author_id?: string }) => {
        const response = await axios.get(API_URL, { params });
        return response.data;
    },

    get: async (id: string) => {
        const response = await axios.get(`${API_URL}/${id}`);
        return response.data;
    },

    create: async (data: any) => {
        const response = await axios.post(API_URL, data);
        return response.data;
    },

    delete: async (id: string) => {
        await axios.delete(`${API_URL}/${id}`);
    }
};
