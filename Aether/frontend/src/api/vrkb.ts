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

export interface VrkbAsset {
    id: string;
    hash: string;
    storage_path: string;
    mime_type: string;
    size_bytes: number;
    created_at: string;
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
    },

    // Assets
    uploadAsset: async (file: File) => {
        const formData = new FormData();
        formData.append('file', file);
        const response = await axios.post('/api/vrkb/assets', formData, {
            headers: {
                ...getAuthHeaders().headers,
                'Content-Type': 'multipart/form-data'
            }
        });
        return response.data;
    },
    // List Assets
    listAssets: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/assets`, getAuthHeaders());
        return response.data; // Expected: VrkbAsset[]
    },
    deleteAsset: async (id: string) => {
        await axios.delete(`/api/vrkb/assets/${id}`, getAuthHeaders());
    },

    // Stats (Overview)
    getProjectStats: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/stats`, getAuthHeaders());
        return response.data; // Expected: VrkbStats
    },

    // Team
    getTeam: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/members`, getAuthHeaders());
        return response.data; // Expected: VrkbMember[]
    },
    addMember: async (projectId: string, userId: string, role: string) => {
        await axios.post(`/api/vrkb/projects/${projectId}/members`, { user_id: userId, role }, getAuthHeaders());
    },
    removeMember: async (projectId: string, userId: string) => {
        await axios.delete(`/api/vrkb/projects/${projectId}/members/${userId}`, getAuthHeaders());
    },
    updateMemberRole: async (projectId: string, userId: string, role: string) => {
        await axios.put(`/api/vrkb/projects/${projectId}/members/${userId}`, { role }, getAuthHeaders());
    },
    searchUsers: async (query: string) => {
        const response = await axios.get(`/api/users/search?q=${query}`, getAuthHeaders());
        return response.data;
    },

    // Docs
    listDocs: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/docs`, getAuthHeaders());
        return response.data; // Expected: VrkbDoc[]
    },
    createDoc: async (projectId: string, title: string, parentId?: string | null) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/docs`, { title, parent_id: parentId }, getAuthHeaders());
        return response.data;
    },
    updateDoc: async (docId: string, title: string, content: any, parentId?: string | null) => {
        const response = await axios.put(`/api/vrkb/docs/${docId}`, { title, content, parent_id: parentId }, getAuthHeaders());
        return response.data;
    },
    deleteDoc: async (docId: string) => {
        await axios.delete(`/api/vrkb/docs/${docId}`, getAuthHeaders());
    },
    moveDoc: async (doc: any, parentId: string | null) => {
        await axios.put(`/api/vrkb/docs/${doc.id}`, {
            title: doc.title,
            content: doc.content,
            parent_id: parentId
        }, getAuthHeaders());
    },

    // Specs
    getSpecs: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/specs`, getAuthHeaders());
        // For simplicity, returning the first spec's content or empty string if none.
        // Or return the array and let UI handle it.
        // Based on previous UI, it expected a string. 
        // Let's assume we want the list in real implementation or just the first 'README' spec.
        if (response.data && response.data.length > 0) {
            return response.data[0].content || "";
        }
        return "";
    },
    saveSpecs: async (projectId: string, title: string, content: string) => {
        // Simplified upsert logic for single spec
        // We need an ID for upsert, let frontend generator or backend handle.
        // The backend handler expects `id` in payload.
        // Detailed implementation would require listing first.

        // For now, let's assume we create a deterministic UUID or just list and update first.
        // Checking getSpecs above returns content.

        // Real implementation:
        // 1. List specs.
        // 2. If exists, update.
        // 3. If not, create.

        // As a quick fix, we'll try to just hit PUT.
        // But PUT needs ID.
        // Let's change the pattern: getSpecs returns list.
        // UI needs to be updated or we adapt here.
        // Let's adapt:
        const list = await axios.get(`/api/vrkb/projects/${projectId}/specs`, getAuthHeaders());
        let specId = "00000000-0000-0000-0000-000000000000"; // Dummy or new
        let version = 1;

        if (list.data && list.data.length > 0) {
            specId = list.data[0].id;
            version = list.data[0].version + 1;
        } else {
            // Generate new ID if creating
            // specId = self.crypto.randomUUID(); // Browser API
            // Or rely on backend to accept new ID. 
            // Ideally project has 1 main spec.
            // We'll generate a random UUID if not found.
            specId = crypto.randomUUID();
        }

        const data = {
            id: specId,
            title,
            content,
            version
        };

        await axios.put(`/api/vrkb/projects/${projectId}/specs`, data, getAuthHeaders());
    }
};
