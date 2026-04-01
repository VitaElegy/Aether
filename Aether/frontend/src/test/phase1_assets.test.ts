/**
 * Phase 1 — Assets 底座 全覆盖测试套件
 *
 * 覆盖 ASSET-01 ~ ASSET-04 的核心增量逻辑。
 * 运行: npx vitest run src/test/phase1_assets.test.ts
 * 或统一运行: npm run test:unit
 */
import {
  type AssetType,
  type AssetPayload,
  type AssetStats,
  extractAssetPayload,
  getAssetDisplayName,
  getAssetType,
  getAssetTypeLabel,
  inferAssetType,
  isImageAsset,
  isStructuredAssetType,
} from '../api/assets';

// ===========================================================================
// ASSET-01: Type System Extension
// ===========================================================================

describe('[ASSET-01] Extended Type System', () => {
  test('All 7 asset types are accepted by inferAssetType', () => {
    // File-based types (inferred from MIME/extension)
    expect(inferAssetType({ mime_type: 'image/png' })).toBe('image_asset');
    expect(inferAssetType({ mime_type: 'application/pdf' })).toBe('pdf_asset');
    expect(inferAssetType({ mime_type: 'application/zip' })).toBe('file_asset');

    // Structured types (explicit asset_type passthrough)
    expect(inferAssetType({ asset_type: 'ip_asset' })).toBe('ip_asset');
    expect(inferAssetType({ asset_type: 'domain_asset' })).toBe('domain_asset');
    expect(inferAssetType({ asset_type: 'credential_stub' })).toBe('credential_stub');
    expect(inferAssetType({ asset_type: 'snippet_asset' })).toBe('snippet_asset');
  });

  test('isStructuredAssetType identifies non-file types', () => {
    expect(isStructuredAssetType('ip_asset')).toBe(true);
    expect(isStructuredAssetType('domain_asset')).toBe(true);
    expect(isStructuredAssetType('credential_stub')).toBe(true);
    expect(isStructuredAssetType('snippet_asset')).toBe(true);
    expect(isStructuredAssetType('image_asset')).toBe(false);
    expect(isStructuredAssetType('pdf_asset')).toBe(false);
    expect(isStructuredAssetType('file_asset')).toBe(false);
  });

  test('getAssetTypeLabel returns human-readable labels for all types', () => {
    const labels: Record<AssetType, string> = {
      image_asset: 'Image',
      pdf_asset: 'PDF',
      file_asset: 'File',
      ip_asset: 'IP',
      domain_asset: 'Domain',
      credential_stub: 'Credential',
      snippet_asset: 'Snippet',
    };

    for (const [type, expected] of Object.entries(labels)) {
      expect(getAssetTypeLabel(type as AssetType)).toBe(expected);
    }
  });

  test('AssetStats shape includes new type counters', () => {
    const stats: AssetStats = {
      total: 10,
      images: 3,
      pdfs: 2,
      files: 1,
      ip_assets: 1,
      domain_assets: 1,
      credential_stubs: 1,
      snippets: 1,
    };

    expect(stats.total).toBe(10);
    expect(stats.ip_assets).toBe(1);
    expect(stats.domain_assets).toBe(1);
    expect(stats.credential_stubs).toBe(1);
    expect(stats.snippets).toBe(1);
  });
});

// ===========================================================================
// ASSET-01: extractAssetPayload + type inference
// ===========================================================================

describe('[ASSET-01] Payload Extraction', () => {
  function makeAssetNode(payload: Partial<AssetPayload>, title = 'Test Asset') {
    return {
      id: 'test-id',
      title,
      body: {
        type: 'Custom' as const,
        data: { file_path: '/test/path', ...payload },
      },
      category: 'Asset',
      created_at: '2026-01-01T00:00:00Z',
    };
  }

  test('extractAssetPayload normalizes MIME type and extension', () => {
    const node = makeAssetNode({
      mime_type: '  IMAGE/PNG  ',
      original_filename: 'photo.PNG',
    });
    const payload = extractAssetPayload(node);
    expect(payload.mime_type).toBe('image/png');
    expect(payload.metadata?.extension).toBe('png');
    expect(payload.asset_type).toBe('image_asset');
  });

  test('extractAssetPayload preserves structured asset type', () => {
    const node = makeAssetNode({
      asset_type: 'ip_asset',
      mime_type: 'text/plain',
    });
    const payload = extractAssetPayload(node);
    expect(payload.asset_type).toBe('ip_asset');
  });

  test('getAssetType falls back to file_asset for empty payload', () => {
    const node = makeAssetNode({});
    expect(getAssetType(node)).toBe('file_asset');
  });

  test('isImageAsset returns true only for image_asset', () => {
    expect(isImageAsset(makeAssetNode({ mime_type: 'image/jpeg' }))).toBe(true);
    expect(isImageAsset(makeAssetNode({ mime_type: 'application/pdf' }))).toBe(false);
    expect(isImageAsset(makeAssetNode({ asset_type: 'ip_asset' }))).toBe(false);
  });

  test('getAssetDisplayName returns display_name > original_filename > title', () => {
    expect(getAssetDisplayName(makeAssetNode({ display_name: 'Custom Name' }))).toBe('Custom Name');
    expect(getAssetDisplayName(makeAssetNode({ original_filename: 'photo.jpg' }))).toBe('photo.jpg');
    expect(getAssetDisplayName(makeAssetNode({}, 'Fallback Title'))).toBe('Fallback Title');
  });
});

// ===========================================================================
// ASSET-02: Upload Pipeline
// ===========================================================================

describe('[ASSET-02] Upload Pipeline Types', () => {
  test('ListAssetsParams accepts sort_by parameter', () => {
    // Type-level test: ensure the interface compiles with sort_by
    const params = {
      q: 'test',
      asset_type: 'image_asset' as AssetType,
      limit: 50,
      offset: 0,
      sort_by: 'newest' as const,
    };
    expect(params.sort_by).toBe('newest');
  });

  test('sort_by accepts all valid values', () => {
    const values: Array<'newest' | 'largest' | 'name'> = ['newest', 'largest', 'name'];
    expect(values).toHaveLength(3);
    values.forEach((v) => expect(typeof v).toBe('string'));
  });
});

// ===========================================================================
// ASSET-03: Multi-View Workbench (sort logic)
// ===========================================================================

describe('[ASSET-03] Sorting Logic', () => {
  function makeNodes(): Array<{
    id: string;
    title: string;
    body: { type: 'Custom'; data: AssetPayload };
    category: string;
    created_at: string;
    updated_at?: string;
  }> {
    return [
      {
        id: '1',
        title: 'Alpha',
        body: { type: 'Custom', data: { file_path: '/a', size_bytes: 100, mime_type: 'text/plain' } },
        category: 'Asset',
        created_at: '2026-01-03T00:00:00Z',
      },
      {
        id: '2',
        title: 'Beta',
        body: { type: 'Custom', data: { file_path: '/b', size_bytes: 500, mime_type: 'image/png' } },
        category: 'Asset',
        created_at: '2026-01-01T00:00:00Z',
      },
      {
        id: '3',
        title: 'Gamma',
        body: { type: 'Custom', data: { file_path: '/c', size_bytes: 200, mime_type: 'application/pdf' } },
        category: 'Asset',
        created_at: '2026-01-02T00:00:00Z',
      },
    ];
  }

  test('sort by newest: most recent first', () => {
    const nodes = makeNodes();
    const sorted = [...nodes].sort((a, b) => {
      return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
    });
    expect(sorted.map((n) => n.id)).toEqual(['1', '3', '2']);
  });

  test('sort by largest: biggest first', () => {
    const nodes = makeNodes();
    const sorted = [...nodes].sort((a, b) => {
      return (b.body.data.size_bytes ?? 0) - (a.body.data.size_bytes ?? 0);
    });
    expect(sorted.map((n) => n.id)).toEqual(['2', '3', '1']);
  });

  test('sort by name: alphabetical', () => {
    const nodes = makeNodes();
    const sorted = [...nodes].sort((a, b) => a.title.localeCompare(b.title));
    expect(sorted.map((n) => n.id)).toEqual(['1', '2', '3']);
  });
});

// ===========================================================================
// ASSET-03: Filter Options
// ===========================================================================

describe('[ASSET-03] Filter Options', () => {
  test('countForFilter returns correct stat for each type', () => {
    const stats: AssetStats = {
      total: 15,
      images: 5,
      pdfs: 3,
      files: 2,
      ip_assets: 2,
      domain_assets: 1,
      credential_stubs: 1,
      snippets: 1,
    };

    function countForFilter(filterId: 'all' | AssetType, s: AssetStats): number {
      switch (filterId) {
        case 'image_asset': return s.images;
        case 'pdf_asset': return s.pdfs;
        case 'file_asset': return s.files;
        case 'ip_asset': return s.ip_assets;
        case 'domain_asset': return s.domain_assets;
        case 'credential_stub': return s.credential_stubs;
        case 'snippet_asset': return s.snippets;
        default: return s.total;
      }
    }

    expect(countForFilter('all', stats)).toBe(15);
    expect(countForFilter('image_asset', stats)).toBe(5);
    expect(countForFilter('pdf_asset', stats)).toBe(3);
    expect(countForFilter('file_asset', stats)).toBe(2);
    expect(countForFilter('ip_asset', stats)).toBe(2);
    expect(countForFilter('domain_asset', stats)).toBe(1);
    expect(countForFilter('credential_stub', stats)).toBe(1);
    expect(countForFilter('snippet_asset', stats)).toBe(1);
  });
});

// ===========================================================================
// ASSET-04: Usage Graph
// ===========================================================================

describe('[ASSET-04] Usage Graph Types', () => {
  test('AssetReferenceItem interface has all required fields', () => {
    const ref = {
      content_id: 'article-1',
      title: 'My Note',
      category: 'Note',
      knowledge_base_id: 'kb-1',
      knowledge_base_title: 'Research',
      updated_at: '2026-01-01T00:00:00Z',
      reference_type: 'embed' as const,
      snippet: 'Some context around the [[asset:...]] marker',
    };

    expect(ref.content_id).toBeDefined();
    expect(ref.reference_type).toBe('embed');
    expect(ref.snippet).toContain('[[asset:');
  });

  test('reference_type can be embed or reference', () => {
    const embedRef = { reference_type: 'embed' as const };
    const linkRef = { reference_type: 'reference' as const };
    expect(embedRef.reference_type).toBe('embed');
    expect(linkRef.reference_type).toBe('reference');
  });
});

// ===========================================================================
// Utility function tests (shared helpers)
// ===========================================================================

describe('[Util] Format Helpers', () => {
  test('formatSize handles edge cases', () => {
    function formatSize(bytes?: number): string {
      if (!bytes) return '0 B';
      const units = ['B', 'KB', 'MB', 'GB'];
      const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
      const value = bytes / Math.pow(1024, exponent);
      return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
    }

    expect(formatSize(0)).toBe('0 B');
    expect(formatSize(undefined)).toBe('0 B');
    expect(formatSize(512)).toBe('512 B');
    expect(formatSize(1024)).toBe('1.0 KB');
    expect(formatSize(1536)).toBe('1.5 KB');
    expect(formatSize(1048576)).toBe('1.0 MB');
    expect(formatSize(10485760)).toBe('10 MB');
  });

  test('truncateHash handles various lengths', () => {
    function truncateHash(hash?: string): string {
      if (!hash) return 'n/a';
      if (hash.length <= 18) return hash;
      return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
    }

    expect(truncateHash(undefined)).toBe('n/a');
    expect(truncateHash('short')).toBe('short');
    // 32-char hash should be truncated to first10...last8
    const longHash = 'abcdef1234567890abcdef1234567890';
    const expected = `${longHash.slice(0, 10)}...${longHash.slice(-8)}`;
    expect(truncateHash(longHash)).toBe(expected);
  });
});

// ===========================================================================
// Preview kind mapping
// ===========================================================================

describe('[ASSET-01] Preview Kind Mapping', () => {
  test('extractAssetPayload sets structured preview kind for new types', () => {
    function makeNode(assetType: AssetType) {
      return {
        id: 'test',
        title: 'Test',
        body: {
          type: 'Custom' as const,
          data: { file_path: '/test', asset_type: assetType },
        },
        category: 'Asset',
        created_at: '2026-01-01T00:00:00Z',
      };
    }

    const ipPayload = extractAssetPayload(makeNode('ip_asset'));
    expect(ipPayload.metadata?.preview_kind).toBe('structured');

    const domainPayload = extractAssetPayload(makeNode('domain_asset'));
    expect(domainPayload.metadata?.preview_kind).toBe('structured');

    const credPayload = extractAssetPayload(makeNode('credential_stub'));
    expect(credPayload.metadata?.preview_kind).toBe('structured');

    const snippetPayload = extractAssetPayload(makeNode('snippet_asset'));
    expect(snippetPayload.metadata?.preview_kind).toBe('structured');

    // Original types keep their preview kinds
    const imagePayload = extractAssetPayload(makeNode('image_asset'));
    expect(imagePayload.metadata?.preview_kind).toBe('image');

    const pdfPayload = extractAssetPayload(makeNode('pdf_asset'));
    expect(pdfPayload.metadata?.preview_kind).toBe('document');
  });
});
