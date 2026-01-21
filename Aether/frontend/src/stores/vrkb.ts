import { defineStore } from 'pinia';
import { ref } from 'vue';
import { vrkbApi, type VrkbProject, type VrkbFinding } from '@/api/vrkb';
// import { useContentStore } from '@/stores/content'; // Removed invalid import

export const useVrkbStore = defineStore('vrkb', () => {
    const projects = ref<VrkbProject[]>([]);
    const currentProject = ref<VrkbProject | null>(null);
    const findings = ref<VrkbFinding[]>([]);
    const sections = ref<any[]>([]); // Need detailed type later
    const isLoading = ref(false);

    // Actions
    const fetchProjects = async () => {
        isLoading.value = true;
        try {
            console.log("Fetching projects...");
            projects.value = await vrkbApi.listProjects();
        } catch (e) {
            console.error(e);
        } finally {
            isLoading.value = false;
        }
    };

    const createProject = async (name: string, repoUrl?: string) => {
        isLoading.value = true;
        try {
            const newProj = await vrkbApi.createProject({
                name,
                repository_url: repoUrl,
                quota_bytes: 1024 * 1024 * 1024 // 1GB default
            });
            projects.value.push(newProj);
            return newProj;
        } catch (e) {
            console.error("Create project failed", e);
            throw e;
        } finally {
            isLoading.value = false;
        }
    };

    const fetchFindings = async (projectId: string) => {
        isLoading.value = true;
        try {
            findings.value = await vrkbApi.listFindings(projectId);
        } finally {
            isLoading.value = false;
        }
    };

    const selectProject = async (id: string | null) => {
        if (!id) {
            currentProject.value = null;
            return;
        }
        isLoading.value = true;
        try {
            const proj = await vrkbApi.getProject(id);
            currentProject.value = proj;

            // Parallel load
            const [f, s] = await Promise.all([
                vrkbApi.listFindings(id),
                vrkbApi.listSections(id)
            ]);
            findings.value = f;
            sections.value = s;
        } finally {
            isLoading.value = false;
        }
    };

    const updateFindingStatus = async (id: string, status: string) => {
        // Optimistic update
        const f = findings.value.find(x => x.id === id);
        if (f) f.status = status;

        await vrkbApi.updateFinding(id, { status });
    };

    const createFinding = async (sectionId: string, title: string, severity: string, content: any) => {
        const newFinding = await vrkbApi.createFinding({
            section_id: sectionId,
            title,
            severity,
            status: 'Triage',
            content,
            is_triage: true
        });
        findings.value.push(newFinding);
        return newFinding;
    };

    return {
        projects,
        currentProject,
        findings,
        sections,
        isLoading,
        fetchProjects,
        createProject,
        selectProject,
        fetchFindings,
        updateFindingStatus,
        createFinding
    };
});

export type { VrkbProject, VrkbFinding, VrkbSection } from '@/api/vrkb';
