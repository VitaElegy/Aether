import { describe, it, expect } from 'vitest';
import type { PermissionExplanation, ContextChainItem } from '@/api/permissions';

describe('Permission Explanation Types', () => {
    it('T-A06-01: PermissionExplanation struct has correct shape', () => {
        const explanation: PermissionExplanation = {
            allowed: true,
            reason_code: 'direct_grant',
            reason_text: 'Granted via direct viewer relation',
            context_chain: [
                {
                    entity_id: '123e4567-e89b-12d3-a456-426614174000',
                    entity_type: 'node',
                    relation: 'viewer',
                    via: 'direct',
                },
            ],
            referenced_by: ['article-1', 'article-2'],
        };

        expect(explanation.allowed).toBe(true);
        expect(explanation.reason_code).toBe('direct_grant');
        expect(explanation.context_chain).toHaveLength(1);
        expect(explanation.referenced_by).toHaveLength(2);
    });

    it('T-A06-02: direct_grant reason_code for direct access', () => {
        const explanation: PermissionExplanation = {
            allowed: true,
            reason_code: 'direct_grant',
            reason_text: "Granted via direct 'viewer' relation on node:abc",
            context_chain: [
                {
                    entity_id: 'abc',
                    entity_type: 'node',
                    relation: 'viewer',
                    via: 'direct',
                },
            ],
            referenced_by: [],
        };
        expect(explanation.reason_code).toBe('direct_grant');
        expect(explanation.context_chain[0].via).toBe('direct');
    });

    it('T-A06-03: group_membership reason_code for group access', () => {
        const explanation: PermissionExplanation = {
            allowed: true,
            reason_code: 'group_membership',
            reason_text: "Granted via group membership ('editor') on node:xyz",
            context_chain: [
                {
                    entity_id: 'xyz',
                    entity_type: 'node',
                    relation: 'editor',
                    via: 'group',
                },
            ],
            referenced_by: [],
        };
        expect(explanation.reason_code).toBe('group_membership');
        expect(explanation.context_chain[0].via).toBe('group');
    });

    it('T-A06-04: parent_inheritance reason_code for parent access', () => {
        const explanation: PermissionExplanation = {
            allowed: true,
            reason_code: 'parent_inheritance',
            reason_text: "Granted via parent inheritance",
            context_chain: [
                {
                    entity_id: 'child-node',
                    entity_type: 'node',
                    relation: 'viewer',
                    via: 'parent',
                },
                {
                    entity_id: 'parent-node',
                    entity_type: 'node',
                    relation: 'viewer',
                    via: 'direct',
                },
            ],
            referenced_by: [],
        };
        expect(explanation.reason_code).toBe('parent_inheritance');
        expect(explanation.context_chain).toHaveLength(2);
    });

    it('T-A06-05: denied reason_code when no access', () => {
        const explanation: PermissionExplanation = {
            allowed: false,
            reason_code: 'denied',
            reason_text: 'User has none of the required relations',
            context_chain: [],
            referenced_by: [],
        };
        expect(explanation.allowed).toBe(false);
        expect(explanation.reason_code).toBe('denied');
        expect(explanation.context_chain).toHaveLength(0);
    });

    it('T-A06-06: context_chain items have correct structure', () => {
        const item: ContextChainItem = {
            entity_id: '00000000-0000-0000-0000-000000000001',
            entity_type: 'node',
            relation: 'owner',
            via: 'direct',
        };
        expect(item.entity_id).toBeTruthy();
        expect(item.entity_type).toBe('node');
        expect(['viewer', 'editor', 'owner', 'author', 'parent']).toContain(item.relation);
        expect(['direct', 'group', 'parent']).toContain(item.via);
    });
});
