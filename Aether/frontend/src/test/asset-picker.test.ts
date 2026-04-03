import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useAssetPicker } from '@/composables/useAssetPicker';

// Mock the assets API
vi.mock('@/api/assets', () => ({
    assetsApi: {
        list: vi.fn().mockResolvedValue({
            items: [
                {
                    id: 'asset-1',
                    title: 'test-image.png',
                    body: {
                        type: 'Custom',
                        data: {
                            asset_type: 'image_asset',
                            display_name: 'test-image.png',
                            file_path: 'uploads/ab/abc123',
                            mime_type: 'image/png',
                            hash: 'abc123',
                            size_bytes: 1024,
                            metadata: { extension: 'png', preview_kind: 'image' },
                        },
                    },
                    category: 'Asset',
                    created_at: '2026-01-01T00:00:00Z',
                },
                {
                    id: 'asset-2',
                    title: 'report.pdf',
                    body: {
                        type: 'Custom',
                        data: {
                            asset_type: 'pdf_asset',
                            display_name: 'report.pdf',
                            file_path: 'uploads/de/def456',
                            mime_type: 'application/pdf',
                            hash: 'def456',
                            size_bytes: 2048,
                            metadata: { extension: 'pdf', preview_kind: 'document' },
                        },
                    },
                    category: 'Asset',
                    created_at: '2026-01-02T00:00:00Z',
                },
            ],
            stats: { total: 2, images: 1, pdfs: 1, files: 0, ip_assets: 0, domain_assets: 0, credential_stubs: 0, snippets: 0 },
            filtered_count: 2,
        }),
        upload: vi.fn().mockResolvedValue({ id: 'new-asset', title: 'uploaded.jpg' }),
        getAssetUrl: vi.fn((id: string) => `/api/assets/${id}`),
    },
    extractAssetPayload: vi.fn((asset: any) => asset.body?.data ?? {}),
    getAssetDisplayName: vi.fn((asset: any) => asset.body?.data?.display_name || asset.title),
    getAssetType: vi.fn((asset: any) => asset.body?.data?.asset_type || 'file_asset'),
    getAssetTypeLabel: vi.fn((type: string) => type),
    isImageAsset: vi.fn((asset: any) => asset.body?.data?.asset_type === 'image_asset'),
}));

describe('useAssetPicker', () => {
    let picker: ReturnType<typeof useAssetPicker>;

    beforeEach(() => {
        picker = useAssetPicker();
        picker._resetForTesting();
    });

    it('T-A05-01: openPicker returns Promise that resolves with selected assets', async () => {
        const promise = picker.openPicker({ mode: 'modal' });
        expect(picker.isOpen.value).toBe(true);
        expect(picker.mode.value).toBe('modal');

        // Select an asset and confirm
        picker.toggleAssetSelection({
            id: 'asset-1',
            title: 'test',
            body: { type: 'Custom', data: {} as any },
            category: 'Asset',
            created_at: '',
        });
        picker.confirmSelection();

        const result = await promise;
        expect(result.cancelled).toBe(false);
        expect(result.assets).toHaveLength(1);
        expect(result.assets[0].id).toBe('asset-1');
    });

    it('T-A05-04: multiple mode allows multi-select', async () => {
        picker.openPicker({ mode: 'modal', multiple: true });
        expect(picker.multiple.value).toBe(true);

        const asset1 = { id: 'a1', title: 't1', body: { type: 'Custom' as const, data: {} as any }, category: '', created_at: '' };
        const asset2 = { id: 'a2', title: 't2', body: { type: 'Custom' as const, data: {} as any }, category: '', created_at: '' };

        picker.toggleAssetSelection(asset1);
        picker.toggleAssetSelection(asset2);

        expect(picker.selectedAssets.value).toHaveLength(2);
    });

    it('T-A05-05: single mode replaces selection', async () => {
        picker.openPicker({ mode: 'modal', multiple: false });

        const asset1 = { id: 'a1', title: 't1', body: { type: 'Custom' as const, data: {} as any }, category: '', created_at: '' };
        const asset2 = { id: 'a2', title: 't2', body: { type: 'Custom' as const, data: {} as any }, category: '', created_at: '' };

        picker.toggleAssetSelection(asset1);
        picker.toggleAssetSelection(asset2);

        // In single mode, only last selection kept
        expect(picker.selectedAssets.value).toHaveLength(1);
        expect(picker.selectedAssets.value[0].id).toBe('a2');
    });

    it('T-A05-06: acceptTypes filters picker initialization', async () => {
        picker.openPicker({ acceptTypes: ['image_asset'] });
        expect(picker.acceptTypes.value).toEqual(['image_asset']);
    });

    it('T-A05-11: cancel resolves with cancelled=true', async () => {
        const promise = picker.openPicker({ mode: 'modal' });
        picker.closePicker();

        const result = await promise;
        expect(result.cancelled).toBe(true);
        expect(result.assets).toHaveLength(0);
    });

    it('T-A05-12: isAssetSelected returns correct status', () => {
        picker.openPicker({ mode: 'modal', multiple: true });

        const asset = { id: 'test-id', title: 't', body: { type: 'Custom' as const, data: {} as any }, category: '', created_at: '' };
        picker.toggleAssetSelection(asset);

        expect(picker.isAssetSelected('test-id')).toBe(true);
        expect(picker.isAssetSelected('other-id')).toBe(false);
    });

    it('T-A05-14: closePicker resets isOpen', () => {
        picker.openPicker({ mode: 'modal' });
        expect(picker.isOpen.value).toBe(true);

        picker.closePicker();
        expect(picker.isOpen.value).toBe(false);
    });

    it('T-A05-09: recent assets tracked after confirm', async () => {
        const promise = picker.openPicker({ mode: 'modal' });
        const asset = { id: 'recent-1', title: 'recent', body: { type: 'Custom' as const, data: {} as any }, category: '', created_at: '' };

        picker.toggleAssetSelection(asset);
        picker.confirmSelection();

        await promise;
        expect(picker.recentAssets.value).toHaveLength(1);
        expect(picker.recentAssets.value[0].id).toBe('recent-1');
    });
});
