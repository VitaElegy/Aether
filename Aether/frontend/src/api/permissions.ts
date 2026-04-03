import axios from 'axios';

/** One link in the permission resolution chain. */
export interface ContextChainItem {
    entity_id: string;
    entity_type: string;
    relation: string;
    /** How the relation was resolved: "direct", "group", "parent" */
    via: string;
}

/** Detailed explanation returned by the permission check endpoint. */
export interface PermissionExplanation {
    allowed: boolean;
    /**
     * One of:
     * - "direct_grant"
     * - "group_membership"
     * - "parent_inheritance"
     * - "author_access"
     * - "context_proxy"
     * - "denied"
     */
    reason_code: string;
    /** Human-readable explanation text. */
    reason_text: string;
    /** The chain of entities that led to the permission decision. */
    context_chain: ContextChainItem[];
    /** IDs of contexts (articles) that reference the target asset. */
    referenced_by: string[];
}

export const permissionsApi = {
    /**
     * Fetches a detailed permission explanation for the given asset.
     * @param assetId  UUID of the asset to check.
     * @param contextId  Optional UUID of a context article (for context-proxy access).
     */
    async explainAssetAccess(
        assetId: string,
        contextId?: string,
    ): Promise<PermissionExplanation> {
        const params: Record<string, string> = {};
        if (contextId) {
            params.context = contextId;
        }
        const response = await axios.get<PermissionExplanation>(
            `/api/assets/${assetId}/permissions`,
            { params },
        );
        return response.data;
    },
};
