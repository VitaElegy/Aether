import axios from 'axios';

export interface AssetNode {
    id: string;
    title: string;
    body: {
        type: 'Custom';
        data: {
            file_path: string;
            original_filename: string;
            mime_type: string;
            hash: string;
            size_bytes: number;
        }
    };
    category: string; // "Asset"
    created_at: string;
}

export const assetsApi = {
    // Upload Asset
    async upload(file: File): Promise<AssetNode> {
        const formData = new FormData();
        formData.append('file', file);
        
        const response = await axios.post('/api/assets', formData, {
            headers: {
                'Content-Type': 'multipart/form-data'
            }
        });
        return response.data;
    },

    // Get Asset URL (helper)
    getAssetUrl(id: string, contextId?: string) {
        let url = `/api/assets/${id}`;
        if (contextId) {
            url += `?context=${contextId}`;
        }
        return url;
    }
};
