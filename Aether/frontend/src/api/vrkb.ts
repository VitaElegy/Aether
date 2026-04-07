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
    /** 7-state lifecycle: triage → confirmed → exploiting → fixing → verifying → closed / risk_accepted */
    status: string;
    severity: string; // Low, Medium, High, Critical
    content?: any;
    is_triage: boolean;
    author_id?: string;
    // VRKB-02 extended fields
    confidence?: string; // certain / firm / tentative
    owner_id?: string;
    due_date?: string; // ISO8601
    affected_assets?: any; // JSON array of asset refs
    repro_steps?: string;
    remediation?: string;
    verification_note?: string;
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
        'Authorization': `Bearer ${localStorage.getItem('aether_token')}`
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
        if (!data.project_id) throw new Error("project_id is required for createSection");
        const response = await axios.post<VrkbSection>(`/api/vrkb/projects/${data.project_id}/sections`, data, getAuthHeaders());
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

    // Finding Status (dedicated PATCH endpoint)
    updateFindingStatus: async (id: string, status: string) => {
        await axios.patch(`/api/vrkb/findings/${id}/status`, { status }, getAuthHeaders());
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
    listAssets: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/assets`, getAuthHeaders());
        return response.data;
    },
    deleteAsset: async (id: string) => {
        await axios.delete(`/api/vrkb/assets/${id}`, getAuthHeaders());
    },

    // Stats (Overview)
    getProjectStats: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/stats`, getAuthHeaders());
        return response.data;
    },

    // Team
    getTeam: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/members`, getAuthHeaders());
        return response.data;
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
        return response.data;
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
        if (response.data && response.data.length > 0) {
            return response.data[0].content || "";
        }
        return "";
    },
    saveSpecs: async (projectId: string, title: string, content: string) => {
        const list = await axios.get(`/api/vrkb/projects/${projectId}/specs`, getAuthHeaders());
        let specId = "00000000-0000-0000-0000-000000000000";
        let version = 1;

        if (list.data && list.data.length > 0) {
            specId = list.data[0].id;
            version = list.data[0].version + 1;
        } else {
            specId = crypto.randomUUID();
        }

        const data = {
            id: specId,
            title,
            content,
            version
        };

        await axios.put(`/api/vrkb/projects/${projectId}/specs`, data, getAuthHeaders());
    },

    // Trash Management
    listTrash: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/trash`, getAuthHeaders());
        return response.data;
    },
    restoreDoc: async (docId: string) => {
        await axios.post(`/api/vrkb/docs/${docId}/restore`, {}, getAuthHeaders());
    },
    permanentDeleteDoc: async (docId: string) => {
        await axios.delete(`/api/vrkb/docs/${docId}/permanent`, getAuthHeaders());
    },

    // --- VRKB-06: Asset Link/Unlink ---
    linkAsset: async (data: { asset_id: string; target_type: string; target_id: string; virtual_path?: string }) => {
        const response = await axios.post('/api/vrkb/assets/link', data, getAuthHeaders());
        return response.data;
    },
    unlinkAsset: async (data: { asset_id: string; target_type: string; target_id: string }) => {
        const response = await axios.post('/api/vrkb/assets/unlink', data, getAuthHeaders());
        return response.data;
    },
    getAssetUsage: async (assetId: string) => {
        const response = await axios.get(`/api/vrkb/assets/${assetId}/usage`, getAuthHeaders());
        return response.data;
    },

    // --- VRKB-07: Doc Repo Enhancements ---
    moveDocTo: async (docId: string, parentId: string | null) => {
        const response = await axios.post(`/api/vrkb/docs/${docId}/move`, { parent_id: parentId }, getAuthHeaders());
        return response.data;
    },
    listDocTemplates: async () => {
        const response = await axios.get('/api/vrkb/docs/templates', getAuthHeaders());
        return response.data;
    },
    createDocFromTemplate: async (projectId: string, templateId: string, title: string) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/docs/from-template`, { template_id: templateId, title }, getAuthHeaders());
        return response.data;
    },
    generateReport: async (projectId: string, options?: { include_findings?: boolean; include_appendix?: boolean }) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/report`, options || {}, getAuthHeaders());
        return response.data;
    },

    // --- VRKB-08: Members & Roles ---
    getMemberPermissions: async (projectId: string, userId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/members/${userId}/permissions`, getAuthHeaders());
        return response.data;
    },
    getPermissionMatrix: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/permissions`, getAuthHeaders());
        return response.data;
    },

    // --- VRKB-09: Audit Log ---
    listAuditLogs: async (projectId: string, params?: { limit?: number; offset?: number; event_type?: string }) => {
        const query = new URLSearchParams();
        if (params?.limit) query.set('limit', params.limit.toString());
        if (params?.offset) query.set('offset', params.offset.toString());
        if (params?.event_type) query.set('event_type', params.event_type);
        const response = await axios.get(`/api/vrkb/projects/${projectId}/audit?${query.toString()}`, getAuthHeaders());
        return response.data;
    },

    // --- VRKB-10: Portability ---
    // NOTE: VRKB project export/import uses the portability API (/api/portability/:kb_id/export/start, etc.)
    // These are handled by the portability store and API, not direct VRKB endpoints.

    // --- VRKB-03: Triage Queue ---
    getTriageQueue: async (projectId: string, filter: string = 'unreviewed') => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/triage?filter=${filter}`, getAuthHeaders());
        return response.data;
    },
    getTriageStats: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/triage/stats`, getAuthHeaders());
        return response.data;
    },
    acceptFinding: async (projectId: string, findingId: string) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/triage/${findingId}/accept`, {}, getAuthHeaders());
        return response.data;
    },
    rejectFinding: async (projectId: string, findingId: string) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/triage/${findingId}/reject`, {}, getAuthHeaders());
        return response.data;
    },
    mergeFinding: async (projectId: string, findingId: string, canonicalId: string) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/triage/${findingId}/merge`, { canonical_id: canonicalId }, getAuthHeaders());
        return response.data;
    },
    requestEvidence: async (projectId: string, findingId: string) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/triage/${findingId}/request-evidence`, {}, getAuthHeaders());
        return response.data;
    },

    // --- VRKB-04: Checklist System ---
    getChecklist: async (sectionId: string) => {
        const response = await axios.get(`/api/vrkb/sections/${sectionId}/checklist`, getAuthHeaders());
        return response.data;
    },
    createChecklistItem: async (sectionId: string, data: { title: string; is_blocker?: boolean; description?: string }) => {
        const response = await axios.post(`/api/vrkb/sections/${sectionId}/checklist`, data, getAuthHeaders());
        return response.data;
    },
    updateChecklistItem: async (sectionId: string, itemId: string, data: any) => {
        const response = await axios.put(`/api/vrkb/sections/${sectionId}/checklist/${itemId}`, data, getAuthHeaders());
        return response.data;
    },
    getChecklistSummary: async (sectionId: string) => {
        const response = await axios.get(`/api/vrkb/sections/${sectionId}/checklist/summary`, getAuthHeaders());
        return response.data;
    },

    // --- VRKB-05: Evidence Blocks ---
    getEvidence: async (projectId: string, attachedToType?: string, attachedToId?: string) => {
        let url = `/api/vrkb/projects/${projectId}/evidence`;
        const params: string[] = [];
        if (attachedToType) params.push(`attached_to_type=${attachedToType}`);
        if (attachedToId) params.push(`attached_to_id=${attachedToId}`);
        if (params.length > 0) url += '?' + params.join('&');
        const response = await axios.get(url, getAuthHeaders());
        return response.data;
    },
    createEvidence: async (projectId: string, data: any) => {
        const response = await axios.post(`/api/vrkb/projects/${projectId}/evidence`, data, getAuthHeaders());
        return response.data;
    },
    deleteEvidence: async (projectId: string, evidenceId: string) => {
        await axios.delete(`/api/vrkb/projects/${projectId}/evidence/${evidenceId}`, getAuthHeaders());
    },

    // --- VRKB-09: Notifications ---
    getNotifications: async (projectId: string) => {
        const response = await axios.get(`/api/vrkb/projects/${projectId}/notifications`, getAuthHeaders());
        return response.data;
    },
    markNotificationRead: async (notificationId: string) => {
        await axios.post(`/api/vrkb/notifications/${notificationId}/read`, {}, getAuthHeaders());
    },
};
