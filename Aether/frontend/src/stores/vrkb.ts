import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import axios from 'axios';

// --- Types ---

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
    status: string; // Triage, Confirming, In Remediation, Verified, Closed
    severity: string; // Critical, High, Medium, Low, Info
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

// --- Store ---

export const useVrkbStore = defineStore('vrkb', () => {
    // State
    const projects = ref<VrkbProject[]>([]);
    const currentProject = ref<VrkbProject | null>(null);
    const sections = ref<VrkbSection[]>([]);
    const findings = ref<VrkbFinding[]>([]);

    const isLoading = ref(false);
    const error = ref<string | null>(null);

    // Getters
    const triageFindings = computed(() => findings.value.filter(f => f.is_triage));
    const activeFindings = computed(() => findings.value.filter(f => !f.is_triage));

    // Actions
    async function fetchProjects() {
        isLoading.value = true;
        try {
            const res = await axios.get('/api/vrkb/projects');
            projects.value = res.data;
        } catch (e: any) {
            error.value = e.message;
        } finally {
            isLoading.value = false;
        }
    }

    async function createProject(name: string, repoUrl?: string) {
        try {
            const res = await axios.post('/api/vrkb/projects', {
                name,
                repository_url: repoUrl
            });
            projects.value.unshift(res.data);
            return res.data;
        } catch (e: any) {
            error.value = e.message;
            throw e;
        }
    }

    async function selectProject(id: string) {
        const p = projects.value.find(p => p.id === id);
        if (p) currentProject.value = p;

        // Load details
        await Promise.all([
            fetchSections(id),
            fetchFindings(id)
        ]);
    }

    async function fetchSections(projectId: string) {
        const res = await axios.get(`/api/vrkb/projects/${projectId}/sections`);
        sections.value = res.data;
    }

    async function createSection(projectId: string, title: string) {
        const res = await axios.post(`/api/vrkb/projects/${projectId}/sections`, { title });
        sections.value.push(res.data);
    }

    async function fetchFindings(projectId: string) {
        // We fetch all findings for the project via query param
        const res = await axios.get('/api/vrkb/findings', {
            params: { project_id: projectId }
        });
        findings.value = res.data;
    }

    async function createFinding(sectionId: string, title: string, severity: string, content: any) {
        const res = await axios.post(`/api/vrkb/sections/${sectionId}/findings`, {
            title,
            severity,
            content,
            status: 'Triage',
            is_triage: true // Default to triage
        });
        findings.value.unshift(res.data);
    }

    async function updateFindingStatus(id: string, status: string) {
        await axios.patch(`/api/vrkb/findings/${id}/status`, { status });
        const f = findings.value.find(f => f.id === id);
        if (f) {
            f.status = status;
            // If moved out of Triage? For now keep is_triage as is or auto-update?
            // Let's assume manual toggle or backend logic. 
            // Phase 6 simplification: just update status locally
        }
    }

    async function uploadAsset(file: File) {
        const formData = new FormData();
        formData.append('file', file);
        const res = await axios.post('/api/vrkb/assets', formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
        });
        return res.data as VrkbAsset;
    }

    return {
        projects,
        currentProject,
        sections,
        findings,
        isLoading,
        error,
        triageFindings,
        activeFindings,
        fetchProjects,
        createProject,
        selectProject,
        fetchSections,
        createSection,
        fetchFindings,
        createFinding,
        updateFindingStatus,
        uploadAsset
    };
});
