import { mount } from '@vue/test-utils';
import MyAssets from './MyAssets.vue';

vi.mock('@/api/assets', async () => {
  const actual = await vi.importActual<typeof import('@/api/assets')>('@/api/assets');
  return {
    ...actual,
    assetsApi: {
      list: vi.fn(),
      listReferences: vi.fn(),
      delete: vi.fn(),
      upload: vi.fn(),
      getAssetUrl: vi.fn((id: string) => `/api/assets/${id}`),
    },
  };
});

import { assetsApi } from '@/api/assets';

const assetsApiMock = vi.mocked(assetsApi, true);

async function flushPromises() {
  await Promise.resolve();
  await Promise.resolve();
}

function makeAsset(overrides: Record<string, unknown> = {}) {
  return {
    id: 'asset-1',
    title: 'diagram.png',
    category: 'Asset',
    created_at: '2026-03-19T00:00:00.000Z',
    updated_at: '2026-03-19T01:00:00.000Z',
    body: {
      type: 'Custom',
      data: {
        asset_type: 'image_asset',
        display_name: 'diagram.png',
        original_filename: 'diagram.png',
        mime_type: 'image/png',
        hash: '1234567890abcdef1234567890abcdef',
        size_bytes: 2048,
        metadata: {
          extension: 'png',
          preview_kind: 'image',
        },
      },
    },
    ...overrides,
  };
}

describe('MyAssets', () => {
  beforeEach(() => {
    vi.stubGlobal('confirm', vi.fn(() => true));
    assetsApiMock.list.mockReset();
    assetsApiMock.listReferences.mockReset();
    assetsApiMock.delete.mockReset();
    assetsApiMock.upload.mockReset();
    assetsApiMock.getAssetUrl.mockImplementation((id: string) => `/api/assets/${id}`);
    assetsApiMock.listReferences.mockResolvedValue([]);
  });

  it('renders empty state when no assets are available', async () => {
    assetsApiMock.list.mockResolvedValueOnce({
      items: [],
      stats: { total: 0, images: 0, pdfs: 0, files: 0 },
      filtered_count: 0,
    } as never);

    const wrapper = mount(MyAssets);
    await flushPromises();

    expect(assetsApiMock.list).toHaveBeenCalledWith({ limit: 200, q: undefined, asset_type: undefined });
    expect(assetsApiMock.listReferences).not.toHaveBeenCalled();
    expect(wrapper.text()).toContain('No assets yet');
  });

  it('uploads selected files then refreshes the asset list with a success message', async () => {
    const uploadedAsset = makeAsset();

    assetsApiMock.list
      .mockResolvedValueOnce({
        items: [],
        stats: { total: 0, images: 0, pdfs: 0, files: 0 },
        filtered_count: 0,
      } as never)
      .mockResolvedValueOnce({
        items: [uploadedAsset as never],
        stats: { total: 1, images: 1, pdfs: 0, files: 0 },
        filtered_count: 1,
      } as never);
    assetsApiMock.upload.mockResolvedValueOnce(uploadedAsset as never);

    const wrapper = mount(MyAssets);
    await flushPromises();

    const input = wrapper.get('input[type="file"]');
    const file = new File(['binary'], 'diagram.png', { type: 'image/png' });
    Object.defineProperty(input.element, 'files', {
      value: [file],
      configurable: true,
    });

    await input.trigger('change');
    await flushPromises();

    expect(assetsApiMock.upload).toHaveBeenCalledWith(file);
    expect(assetsApiMock.list).toHaveBeenCalledTimes(2);
    expect(assetsApiMock.listReferences).toHaveBeenCalledWith('asset-1');
    expect(wrapper.text()).toContain('Uploaded 1 asset.');
    expect(wrapper.text()).toContain('diagram.png');
  });

  it('copies markdown embed syntax for the selected asset', async () => {
    const asset = makeAsset();
    assetsApiMock.listReferences.mockResolvedValueOnce([]);
    assetsApiMock.list.mockResolvedValueOnce({
      items: [asset as never],
      stats: { total: 1, images: 1, pdfs: 0, files: 0 },
      filtered_count: 1,
    } as never);

    const wrapper = mount(MyAssets);
    await flushPromises();

    const copyButton = wrapper.get('button[title="Copy Markdown Link"]');
    await copyButton.trigger('click');

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('![diagram.png]([[asset:asset-1]])');
    expect(wrapper.text()).toContain('Copied embed syntax for diagram.png.');
  });

  it('deletes the selected asset and refreshes the catalog', async () => {
    const asset = makeAsset();
    assetsApiMock.list
      .mockResolvedValueOnce({
        items: [asset as never],
        stats: { total: 1, images: 1, pdfs: 0, files: 0 },
        filtered_count: 1,
      } as never)
      .mockResolvedValueOnce({
        items: [],
        stats: { total: 0, images: 0, pdfs: 0, files: 0 },
        filtered_count: 0,
      } as never);
    assetsApiMock.listReferences.mockResolvedValueOnce([]);
    assetsApiMock.delete.mockResolvedValueOnce(undefined);

    const wrapper = mount(MyAssets);
    await flushPromises();

    await wrapper.get('button[title="Delete Asset"]').trigger('click');
    await flushPromises();

    expect(globalThis.confirm).toHaveBeenCalled();
    expect(assetsApiMock.delete).toHaveBeenCalledWith('asset-1');
    expect(wrapper.text()).toContain('Deleted diagram.png.');
  });

  it('filters non-image assets and updates the detail panel selection', async () => {
    const imageAsset = makeAsset();
    const pdfAsset = makeAsset({
      id: 'asset-2',
      title: 'paper.pdf',
      body: {
        type: 'Custom',
        data: {
          asset_type: 'pdf_asset',
          display_name: 'paper.pdf',
          original_filename: 'paper.pdf',
          mime_type: 'application/pdf',
          hash: 'fedcba0987654321fedcba0987654321',
          size_bytes: 4096,
          metadata: {
            extension: 'pdf',
            preview_kind: 'document',
          },
        },
      },
    });

    assetsApiMock.list.mockResolvedValueOnce({
      items: [imageAsset as never, pdfAsset as never],
      stats: { total: 2, images: 1, pdfs: 1, files: 0 },
      filtered_count: 2,
    } as never).mockResolvedValueOnce({
      items: [pdfAsset as never],
      stats: { total: 2, images: 1, pdfs: 1, files: 0 },
      filtered_count: 1,
    } as never);
    assetsApiMock.listReferences
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([
        {
          content_id: 'note-1',
          title: 'Audit Note',
          knowledge_base_title: 'Research Notes',
          updated_at: '2026-03-19T02:00:00.000Z',
          reference_type: 'reference',
          snippet: 'See [[asset:asset-2]] for the PDF.',
        },
      ] as never);

    const wrapper = mount(MyAssets);
    await flushPromises();

    await wrapper.get('[data-testid="asset-filter-pdf_asset"]').trigger('click');
    await flushPromises();

    expect(assetsApiMock.list).toHaveBeenLastCalledWith({ limit: 200, q: undefined, asset_type: 'pdf_asset' });
    expect(wrapper.text()).toContain('paper.pdf');
    expect(wrapper.text()).not.toContain('diagram.png');
    expect(wrapper.get('[data-testid="asset-detail-panel"]').text()).toContain('paper.pdf');
    expect(wrapper.get('[data-testid="asset-detail-panel"]').text()).toContain('PDF asset stored for cross-KB reuse.');
    expect(wrapper.get('[data-testid="asset-reference-list"]').text()).toContain('Audit Note');
    expect(wrapper.get('[data-testid="asset-reference-list"]').text()).toContain('Research Notes');
  });
});
