import axios from 'axios';

export interface VrkbProject {
    id: string;
    name: string;
    repository_url?: string;
    quota_bytes: number;
    settings?: any;
    created_at: string;
    updated_at: string;
}

export interface VrkbSection {
    id: string;
    project_id: string;
    title: string;
    checklist?: any;
    created_at: string;
    updated_at: string;
}

export interface VrkbFinding {
    id: string;
    section_id: string;
    title: string;
    status: string; // Pending, Triage, Fixing, Verified
    severity: string; // Low, Medium, High, Critical
    content?: any;
    is_triage: boolean;
    author_id?: string;
    created_at: string;
    updated_at: string;
}

// Helper to get headers
const getAuthHeaders = () => ({
    headers: {
        'Authorization': `Bearer ${localStorage.getItem('token')}`
    }
});

export const vrkbApi = {
    // Projects
    listProjects: async () => {
        const response = await axios.get<VrkbProject[]>('/api/vrkb/projects', getAuthHeaders());
        return response.data;
    },
    getProject: async (id: string) => {
        const response = await axios.get<VrkbProject>(`/api/vrkb/projects/${id}`, getAuthHeaders());
        return response.data;
    },
    createProject: async (data: Partial<VrkbProject>) => {
        const response = await axios.post<VrkbProject>('/api/vrkb/projects', data, getAuthHeaders());
        return response.data;
    },
    updateProject: async (id: string, data: Partial<VrkbProject>) => {
        const response = await axios.put<VrkbProject>(`/api/vrkb/projects/${id}`, data, getAuthHeaders());
        return response.data;
    },
    deleteProject: async (id: string) => {
        await axios.delete(`/api/vrkb/projects/${id}`, getAuthHeaders());
    },

    // Sections
    listSections: async (projectId: string) => {
        const response = await axios.get<VrkbSection[]>(`/api/vrkb/projects/${projectId}/sections`, getAuthHeaders());
        return response.data;
    },
    createSection: async (data: Partial<VrkbSection>) => {
        const response = await axios.post<VrkbSection>('/api/vrkb/sections', data, getAuthHeaders());
        return response.data;
    },

    // Findings
    listFindings: async (projectId: string) => {
        const response = await axios.get<VrkbFinding[]>(`/api/vrkb/findings?project_id=${projectId}`, getAuthHeaders());
        return response.data;
    },
    createFinding: async (data: Partial<VrkbFinding>) => {
        if (!data.section_id) throw new Error("Section ID required for findings");
        const response = await axios.post<VrkbFinding>(`/api/vrkb/sections/${data.section_id}/findings`, data, getAuthHeaders());
        return response.data;
    },
    updateFinding: async (id: string, data: Partial<VrkbFinding>) => {
        const response = await axios.put<VrkbFinding>(`/api/vrkb/findings/${id}`, data, getAuthHeaders());
        return response.data;
    },
    deleteFinding: async (id: string) => {
        await axios.delete(`/api/vrkb/findings/${id}`, getAuthHeaders());
    }
};
