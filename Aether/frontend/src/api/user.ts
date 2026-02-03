import axios from 'axios';

export interface UserSummary {
    id: string;
    username: string;
    display_name: string | null;
    avatar_url: string | null;
}

export const userApi = {
    search: async (query: string = '', limit: number = 20, offset: number = 0) => {
        const res = await axios.get<UserSummary[]>(`/api/users/search`, {
            params: { q: query, limit, offset }
        });
        return res;
    },
};
