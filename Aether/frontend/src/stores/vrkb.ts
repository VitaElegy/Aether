import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { vrkbApi, type VrkbProject, type VrkbFinding } from '@/api/vrkb';
import { portabilityApi } from '@/api/portability';

export const useVrkbStore = defineStore('vrkb', () => {
    const projects = ref<VrkbProject[]>([]);
    const currentProject = ref<VrkbProject | null>(null);
    const findings = ref<VrkbFinding[]>([]);
    const sections = ref<any[]>([]); // Need detailed type later
    const isLoading = ref(false);

    // VRKB-08: Permission state
    const permissionMatrix = ref<any>(null);
    const currentUserPermissions = ref<string[]>([]);

    // VRKB-09: Audit state
    const auditLogs = ref<any[]>([]);

    // VRKB-01: Project stats
    const projectStats = ref<any>(null);

    // VRKB-03: Triage Queue
    const triageQueue = ref<{ unreviewed: any[]; duplicates: any[]; stale: any[]; missingEvidence: any[] }>({
        unreviewed: [], duplicates: [], stale: [], missingEvidence: [],
    });
    const triageLoading = ref(false);

    // VRKB-05: Evidence
    const evidence = ref<any[]>([]);
    const projectAssets = ref<any[]>([]);

    // VRKB-02: Findings by status (computed)
    const findingsByStatus = computed(() => {
        const grouped: Record<string, VrkbFinding[]> = {
            triage: [], confirmed: [], exploiting: [], fixing: [],
            verifying: [], closed: [], risk_accepted: [],
        };
        for (const f of findings.value) {
            if (grouped[f.status]) grouped[f.status].push(f);
            else grouped[f.status] = [f];
        }
        return grouped;
    });

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
        } catch (e) {
            console.error("Failed to fetch findings", e);
            findings.value = [];
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

            // VRKB-08: Load permission matrix and current user permissions in background
            vrkbApi.getPermissionMatrix(id).then(m => {
                permissionMatrix.value = m;
            }).catch(() => {});

            // Try to load current user's permissions
            try {
                const userId = localStorage.getItem('userId');
                if (userId) {
                    const memberPerms = await vrkbApi.getMemberPermissions(id, userId);
                    currentUserPermissions.value = memberPerms.permissions || [];
                }
            } catch {
                // User may not be a member yet, default to empty permissions
                currentUserPermissions.value = [];
            }
        } finally {
            isLoading.value = false;
        }
    };

    const updateFindingStatus = async (id: string, status: string) => {
        // Optimistic update with rollback
        const f = findings.value.find(x => x.id === id);
        const prevStatus = f?.status;
        if (f) f.status = status;

        try {
            await vrkbApi.updateFindingStatus(id, status);
        } catch (e) {
            // Rollback on failure
            if (f && prevStatus !== undefined) {
                f.status = prevStatus;
            }
            throw e;
        }
    };

    const createFinding = async (sectionId: string, title: string, severity: string, content: any) => {
        const newFinding = await vrkbApi.createFinding({
            section_id: sectionId,
            title,
            severity,
            status: 'triage',
            content,
            is_triage: true
        });
        findings.value.push(newFinding);
        return newFinding;
    };

    const updateFinding = async (id: string, data: Partial<VrkbFinding>) => {
        const updated = await vrkbApi.updateFinding(id, data);
        const idx = findings.value.findIndex(f => f.id === id);
        if (idx !== -1) {
            findings.value[idx] = updated;
        }
        return updated;
    };

    const uploadAsset = async (file: File) => {
        isLoading.value = true;
        try {
            return await vrkbApi.uploadAsset(file);
        } finally {
            isLoading.value = false;
        }
    };

    // VRKB-06: Link asset to project
    const linkAssetToProject = async (assetId: string, projectId: string) => {
        return await vrkbApi.linkAsset({
            asset_id: assetId,
            target_type: 'project',
            target_id: projectId,
        });
    };

    // VRKB-07: Generate report
    const generateReport = async (projectId: string, options?: { include_findings?: boolean; include_appendix?: boolean }) => {
        isLoading.value = true;
        try {
            return await vrkbApi.generateReport(projectId, options);
        } finally {
            isLoading.value = false;
        }
    };

    // VRKB-08: Check permission
    const hasPermission = (action: string): boolean => {
        return currentUserPermissions.value.includes(action);
    };

    // VRKB-09: Fetch audit logs
    const fetchAuditLogs = async (projectId: string, params?: any) => {
        try {
            const result = await vrkbApi.listAuditLogs(projectId, params);
            auditLogs.value = result.items || [];
            return result;
        } catch (e) {
            console.error("Failed to fetch audit logs", e);
            return { items: [], total: 0 };
        }
    };

    // VRKB-10: Export project (via portability API)
    const exportProject = async (projectId: string) => {
        isLoading.value = true;
        try {
            return await portabilityApi.startExport(projectId);
        } finally {
            isLoading.value = false;
        }
    };

    // VRKB-10: Import project (via portability API)
    const importProject = async (projectId: string, file: File) => {
        isLoading.value = true;
        try {
            const result = await portabilityApi.startImport(projectId, file);
            await fetchProjects(); // Refresh project list
            return result;
        } finally {
            isLoading.value = false;
        }
    };

    // VRKB-01: Load project stats
    const loadProjectStats = async (projectId: string) => {
        try {
            projectStats.value = await vrkbApi.getProjectStats(projectId);
        } catch (e) {
            console.error('Failed to load project stats', e);
        }
    };

    // VRKB-03: Triage Queue actions
    const loadTriageQueue = async (projectId: string) => {
        triageLoading.value = true;
        try {
            const [unreviewed, duplicates, stale, missingEvidence] = await Promise.all([
                vrkbApi.getTriageQueue(projectId, 'unreviewed'),
                vrkbApi.getTriageQueue(projectId, 'duplicates'),
                vrkbApi.getTriageQueue(projectId, 'stale'),
                vrkbApi.getTriageQueue(projectId, 'missing_evidence'),
            ]);
            triageQueue.value = {
                unreviewed: unreviewed || [],
                duplicates: duplicates || [],
                stale: stale || [],
                missingEvidence: missingEvidence || [],
            };
        } catch (e) {
            console.error('Failed to load triage queue', e);
        } finally {
            triageLoading.value = false;
        }
    };

    const triageAccept = async (projectId: string, findingId: string) => {
        await vrkbApi.acceptFinding(projectId, findingId);
        await loadTriageQueue(projectId);
    };
    const triageReject = async (projectId: string, findingId: string) => {
        await vrkbApi.rejectFinding(projectId, findingId);
        await loadTriageQueue(projectId);
    };
    const triageMerge = async (projectId: string, findingId: string, canonicalId: string) => {
        await vrkbApi.mergeFinding(projectId, findingId, canonicalId);
        await loadTriageQueue(projectId);
    };
    const triageRequestEvidence = async (projectId: string, findingId: string) => {
        await vrkbApi.requestEvidence(projectId, findingId);
        await loadTriageQueue(projectId);
    };

    // VRKB-05: Evidence actions
    const loadEvidence = async (projectId: string, entityType: string, entityId: string) => {
        try {
            evidence.value = await vrkbApi.getEvidence(projectId, entityType, entityId);
        } catch (e) { console.error('Failed to load evidence', e); }
    };
    const loadProjectAssets = async (projectId: string) => {
        try {
            projectAssets.value = await vrkbApi.listAssets(projectId);
        } catch (e) { console.error('Failed to load project assets', e); }
    };

    return {
        projects,
        currentProject,
        findings,
        sections,
        isLoading,
        permissionMatrix,
        currentUserPermissions,
        auditLogs,
        fetchProjects,
        createProject,
        selectProject,
        fetchFindings,
        updateFindingStatus,
        createFinding,
        updateFinding,
        uploadAsset,
        linkAssetToProject,
        generateReport,
        hasPermission,
        fetchAuditLogs,
        exportProject,
        importProject,
        // VRKB-01
        projectStats,
        loadProjectStats,
        // VRKB-02
        findingsByStatus,
        // VRKB-03
        triageQueue,
        triageLoading,
        loadTriageQueue,
        triageAccept,
        triageReject,
        triageMerge,
        triageRequestEvidence,
        // VRKB-05/06
        evidence,
        projectAssets,
        loadEvidence,
        loadProjectAssets
    };
});

export type { VrkbProject, VrkbFinding, VrkbSection, VrkbAsset } from '@/api/vrkb';
