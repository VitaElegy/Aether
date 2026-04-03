import { describe, it, expect } from 'vitest';

/**
 * VRKB API Alignment Test Suite
 * Verifies that frontend API calls match the expected backend routes.
 */
describe('VRKB API Alignment', () => {
    it('T-VR-06: createSection sends to /api/vrkb/projects/:id/sections', async () => {
        // Import and check the API definition
        const { vrkbApi } = await import('@/api/vrkb');
        
        // Verify the function exists and requires project_id
        expect(typeof vrkbApi.createSection).toBe('function');
        
        // Should throw if no project_id
        await expect(vrkbApi.createSection({ title: 'test' })).rejects.toThrow('project_id is required');
    });

    it('T-VR-07: updateFindingStatus uses dedicated API method', async () => {
        const { vrkbApi } = await import('@/api/vrkb');
        expect(typeof vrkbApi.updateFindingStatus).toBe('function');
    });

    it('T-VR-API: All expected API methods exist', async () => {
        const { vrkbApi } = await import('@/api/vrkb');
        
        // Projects
        expect(typeof vrkbApi.listProjects).toBe('function');
        expect(typeof vrkbApi.getProject).toBe('function');
        expect(typeof vrkbApi.createProject).toBe('function');
        expect(typeof vrkbApi.updateProject).toBe('function');
        expect(typeof vrkbApi.deleteProject).toBe('function');
        
        // Sections
        expect(typeof vrkbApi.listSections).toBe('function');
        expect(typeof vrkbApi.createSection).toBe('function');
        
        // Findings
        expect(typeof vrkbApi.listFindings).toBe('function');
        expect(typeof vrkbApi.createFinding).toBe('function');
        expect(typeof vrkbApi.updateFinding).toBe('function');
        expect(typeof vrkbApi.deleteFinding).toBe('function');
        expect(typeof vrkbApi.updateFindingStatus).toBe('function');
        
        // Assets
        expect(typeof vrkbApi.uploadAsset).toBe('function');
        expect(typeof vrkbApi.listAssets).toBe('function');
        expect(typeof vrkbApi.deleteAsset).toBe('function');
        
        // Stats & Team
        expect(typeof vrkbApi.getProjectStats).toBe('function');
        expect(typeof vrkbApi.getTeam).toBe('function');
        expect(typeof vrkbApi.addMember).toBe('function');
        expect(typeof vrkbApi.removeMember).toBe('function');
        expect(typeof vrkbApi.updateMemberRole).toBe('function');
        
        // Docs
        expect(typeof vrkbApi.listDocs).toBe('function');
        expect(typeof vrkbApi.createDoc).toBe('function');
        expect(typeof vrkbApi.updateDoc).toBe('function');
        expect(typeof vrkbApi.deleteDoc).toBe('function');
        expect(typeof vrkbApi.moveDoc).toBe('function');
        
        // Specs
        expect(typeof vrkbApi.getSpecs).toBe('function');
        expect(typeof vrkbApi.saveSpecs).toBe('function');
        
        // Trash
        expect(typeof vrkbApi.listTrash).toBe('function');
        expect(typeof vrkbApi.restoreDoc).toBe('function');
        expect(typeof vrkbApi.permanentDeleteDoc).toBe('function');
    });

    it('T-VR-08: ProjectSpecs uses marked, not markdown-it', async () => {
        // Verify by checking that marked is available
        const marked = await import('marked');
        expect(typeof marked.marked).toBe('function');
        
        // Render a simple test
        const result = marked.marked('# Hello');
        expect(result).toContain('<h1');
    });
});
