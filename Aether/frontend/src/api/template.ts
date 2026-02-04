import axios from 'axios';

export interface LayoutTemplate {
    id: string;
    renderer_id: string;
    title: string;
    description: string;
    thumbnail?: string;
    tags: string[];
    config?: any;
    created_at: string;
}

export interface CreateTemplateDto {
    renderer_id: string;
    title: string;
    description: string;
    thumbnail?: string;
    tags: string[];
    config?: any;
}

export interface UpdateTemplateDto {
    renderer_id?: string;
    title?: string;
    description?: string;
    thumbnail?: string;
    tags?: string[];
    config?: any;
}

export const templateApi = {
    list: async (): Promise<{ data: LayoutTemplate[] }> => {
        // Axios returns the whole response object, we need to match the expected return type of the caller.
        // In registry we did: const { data } = await templateApi.list();
        // So this should return the AxiosResponse or similar structure
        return axios.get<LayoutTemplate[]>('/api/templates');
    },
    create: (data: CreateTemplateDto) => axios.post<LayoutTemplate>('/api/templates', data),
    update: (id: string, data: UpdateTemplateDto) => axios.put<LayoutTemplate>(`/api/templates/${id}`, data),
    delete: (id: string) => axios.delete(`/api/templates/${id}`),
};
