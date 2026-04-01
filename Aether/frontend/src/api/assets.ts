import axios from 'axios';

export type AssetType =
  | 'image_asset'
  | 'pdf_asset'
  | 'file_asset'
  | 'ip_asset'
  | 'domain_asset'
  | 'credential_stub'
  | 'snippet_asset';
export type AssetPreviewKind = 'image' | 'document' | 'file' | 'structured';

export interface AssetMetadata {
    extension?: string | null;
    preview_kind?: AssetPreviewKind;
    classification_source?: string;
}

export interface AssetPayload {
    version?: number;
    asset_type?: AssetType;
    display_name?: string;
    file_path: string;
    original_filename?: string;
    mime_type?: string;
    hash?: string;
    size_bytes?: number;
    metadata?: AssetMetadata;
}

export interface AssetNode {
    id: string;
    title: string;
    body: {
        type: 'Custom';
        data: AssetPayload;
    };
    category: string;
    created_at: string;
    updated_at?: string;
}

export interface AssetStats {
    total: number;
    images: number;
    pdfs: number;
    files: number;
    ip_assets: number;
    domain_assets: number;
    credential_stubs: number;
    snippets: number;
}

export interface ListAssetsParams {
    q?: string;
    asset_type?: AssetType;
    limit?: number;
    offset?: number;
    sort_by?: 'newest' | 'largest' | 'name';
}

export interface AssetCatalogResponse {
    items: AssetNode[];
    stats: AssetStats;
    filtered_count: number;
}

export interface AssetReferenceItem {
    content_id: string;
    title: string;
    category?: string;
    knowledge_base_id?: string;
    knowledge_base_title?: string;
    updated_at: string;
    reference_type: 'embed' | 'reference' | string;
    snippet: string;
}

export function extractAssetPayload(asset: Pick<AssetNode, 'title' | 'body'>): AssetPayload {
    const payload = asset.body?.data ?? ({} as AssetPayload);
    const mimeType = payload.mime_type?.trim().toLowerCase() || 'application/octet-stream';
    const extension = payload.metadata?.extension?.trim().toLowerCase()
        || payload.original_filename?.split('.').pop()?.trim().toLowerCase()
        || payload.display_name?.split('.').pop()?.trim().toLowerCase()
        || null;

    const assetType = payload.asset_type ?? inferAssetType(payload, mimeType, extension);
    const previewKind = payload.metadata?.preview_kind ?? inferPreviewKind(assetType);

    return {
        version: payload.version ?? 1,
        asset_type: assetType,
        display_name: payload.display_name || payload.original_filename || asset.title,
        file_path: payload.file_path || '',
        original_filename: payload.original_filename || asset.title,
        mime_type: mimeType,
        hash: payload.hash,
        size_bytes: payload.size_bytes,
        metadata: {
            extension,
            preview_kind: previewKind,
            classification_source: payload.metadata?.classification_source,
        },
    };
}

export function getAssetDisplayName(asset: Pick<AssetNode, 'title' | 'body'>): string {
    return extractAssetPayload(asset).display_name || asset.title;
}

export function getAssetType(asset: Pick<AssetNode, 'title' | 'body'>): AssetType {
    return extractAssetPayload(asset).asset_type || 'file_asset';
}

export function getAssetTypeLabel(assetType: AssetType): string {
    switch (assetType) {
        case 'image_asset':
            return 'Image';
        case 'pdf_asset':
            return 'PDF';
        case 'ip_asset':
            return 'IP';
        case 'domain_asset':
            return 'Domain';
        case 'credential_stub':
            return 'Credential';
        case 'snippet_asset':
            return 'Snippet';
        default:
            return 'File';
    }
}

export function isImageAsset(asset: Pick<AssetNode, 'title' | 'body'>): boolean {
    return getAssetType(asset) === 'image_asset';
}

export function inferAssetType(
    payload: Partial<AssetPayload>,
    normalizedMimeType = payload.mime_type?.trim().toLowerCase() || '',
    extension = payload.metadata?.extension || payload.original_filename?.split('.').pop()?.trim().toLowerCase() || payload.display_name?.split('.').pop()?.trim().toLowerCase() || null,
): AssetType {
    // Explicit asset_type in payload takes precedence for structured types
    if (payload.asset_type && isStructuredAssetType(payload.asset_type)) {
        return payload.asset_type;
    }

    if (normalizedMimeType.startsWith('image/')) {
        return 'image_asset';
    }

    if (normalizedMimeType === 'application/pdf' || extension === 'pdf') {
        return 'pdf_asset';
    }

    return 'file_asset';
}

/** Returns true for asset types that are not inferred from MIME/extension. */
export function isStructuredAssetType(assetType: AssetType): boolean {
    return ['ip_asset', 'domain_asset', 'credential_stub', 'snippet_asset'].includes(assetType);
}

function inferPreviewKind(assetType: AssetType): AssetPreviewKind {
    switch (assetType) {
        case 'image_asset':
            return 'image';
        case 'pdf_asset':
            return 'document';
        case 'ip_asset':
        case 'domain_asset':
        case 'credential_stub':
        case 'snippet_asset':
            return 'structured';
        default:
            return 'file';
    }
}

export const assetsApi = {
    async list(params?: ListAssetsParams): Promise<AssetCatalogResponse> {
        const response = await axios.get('/api/assets', { params });
        return response.data;
    },

    async listReferences(id: string): Promise<AssetReferenceItem[]> {
        const response = await axios.get(`/api/assets/${id}/references`);
        return response.data;
    },

    async delete(id: string): Promise<void> {
        await axios.delete(`/api/assets/${id}`, {
            headers: {
                Authorization: `Bearer ${localStorage.getItem('token')}`,
            },
        });
    },

    async upload(file: File): Promise<AssetNode> {
        const formData = new FormData();
        formData.append('file', file);

        const response = await axios.post('/api/assets', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
                Authorization: `Bearer ${localStorage.getItem('token')}`,
            },
        });
        return response.data;
    },

    getAssetUrl(id: string, contextId?: string) {
        let url = `/api/assets/${id}`;
        if (contextId) {
            url += `?context=${contextId}`;
        }
        return url;
    },
};
