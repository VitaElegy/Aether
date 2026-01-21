import { defineStore } from 'pinia';
import { ref } from 'vue';
import { vrkbApi, type VrkbProject, type VrkbFinding } from '@/api/vrkb';
// import { useContentStore } from '@/stores/content'; // Removed invalid import

export const useVrkbStore = defineStore('vrkb', () => {
    const projects = ref<VrkbProject[]>([]);
    const currentProject = ref<VrkbProject | null>(null);
    const findings = ref<VrkbFinding[]>([]);
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

    const selectProject = async (id: string) => {
        isLoading.value = true;
        try {
            const proj = await vrkbApi.getProject(id);
            currentProject.value = proj;

            // Should also load findings?
            const f = await vrkbApi.listFindings(id);
            findings.value = f;

            // Optional: Update global content context if we had one
        } finally {
            isLoading.value = false;
        }
    };

    return {
        projects,
        currentProject,
        findings,
        isLoading,
        fetchProjects,
        createProject,
        selectProject
    };
});
