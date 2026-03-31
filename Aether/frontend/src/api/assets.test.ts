import axios from 'axios';
import {
  assetsApi,
  extractAssetPayload,
  getAssetDisplayName,
  getAssetType,
  getAssetTypeLabel,
  inferAssetType,
  isImageAsset,
} from './assets';

vi.mock('axios', () => ({
  default: {
    delete: vi.fn(),
    get: vi.fn(),
    post: vi.fn(),
  },
}));

const axiosMock = vi.mocked(axios, true);

describe('assetsApi', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('uploads assets with multipart content type and auth header', async () => {
    localStorage.setItem('token', 'asset-token');
    const file = new File(['hello'], 'note.png', { type: 'image/png' });
    axiosMock.post.mockResolvedValueOnce({ data: { id: 'asset-1' } });

    await assetsApi.upload(file);

    expect(axiosMock.post).toHaveBeenCalledWith(
      '/api/assets',
      expect.any(FormData),
      expect.objectContaining({
        headers: expect.objectContaining({
          'Content-Type': 'multipart/form-data',
          Authorization: 'Bearer asset-token',
        }),
      }),
    );
  });

  it('builds contextual asset urls when a context id is provided', () => {
    expect(assetsApi.getAssetUrl('asset-1')).toBe('/api/assets/asset-1');
    expect(assetsApi.getAssetUrl('asset-1', 'article-9')).toBe('/api/assets/asset-1?context=article-9');
  });

  it('loads typed asset catalogs from the dedicated assets endpoint', async () => {
    axiosMock.get.mockResolvedValueOnce({
      data: {
        items: [{ id: 'asset-1' }],
        stats: { total: 1, images: 1, pdfs: 0, files: 0 },
        filtered_count: 1,
      },
    });

    const response = await assetsApi.list({ q: 'diagram', asset_type: 'image_asset', limit: 50 });

    expect(axiosMock.get).toHaveBeenCalledWith('/api/assets', {
      params: { q: 'diagram', asset_type: 'image_asset', limit: 50 },
    });
    expect(response.stats.total).toBe(1);
  });

  it('loads asset reference contexts from the dedicated endpoint', async () => {
    axiosMock.get.mockResolvedValueOnce({
      data: [
        {
          content_id: 'note-1',
          title: 'Audit Note',
          reference_type: 'embed',
          snippet: '... [[asset:asset-1]] ...',
        },
      ],
    });

    const response = await assetsApi.listReferences('asset-1');

    expect(axiosMock.get).toHaveBeenCalledWith('/api/assets/asset-1/references');
    expect(response[0]?.content_id).toBe('note-1');
  });

  it('deletes assets with auth header', async () => {
    localStorage.setItem('token', 'asset-token');
    axiosMock.delete.mockResolvedValueOnce({});

    await assetsApi.delete('asset-1');

    expect(axiosMock.delete).toHaveBeenCalledWith('/api/assets/asset-1', {
      headers: {
        Authorization: 'Bearer asset-token',
      },
    });
  });

  it('normalizes legacy asset payloads into typed assets', () => {
    const asset = {
      id: 'asset-9',
      title: 'paper.pdf',
      body: {
        type: 'Custom' as const,
        data: {
          file_path: 'uploads/ab/hash',
          original_filename: 'paper.pdf',
          mime_type: 'application/octet-stream',
          hash: 'hash',
          size_bytes: 1024,
        },
      },
      category: 'Asset',
      created_at: '2026-03-19T00:00:00.000Z',
    };

    const payload = extractAssetPayload(asset);

    expect(payload.asset_type).toBe('pdf_asset');
    expect(payload.metadata?.preview_kind).toBe('document');
    expect(getAssetDisplayName(asset)).toBe('paper.pdf');
    expect(getAssetType(asset)).toBe('pdf_asset');
    expect(getAssetTypeLabel('pdf_asset')).toBe('PDF');
    expect(isImageAsset(asset)).toBe(false);
  });

  it('infers image assets from mime type', () => {
    expect(inferAssetType({
      file_path: 'uploads/ab/hash',
      original_filename: 'diagram.png',
      mime_type: 'image/png',
    })).toBe('image_asset');
  });
});
