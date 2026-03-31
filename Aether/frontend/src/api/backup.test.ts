import axios from 'axios';
import { backupApi, extractBackupApiError, formatBackupApiError } from './backup';

vi.mock('axios', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
  },
}));

const axiosMock = vi.mocked(axios, true);

describe('backupApi', () => {
  beforeEach(() => {
    localStorage.clear();
    localStorage.setItem('token', 'backup-token');
  });

  it('sends auth headers for list and create calls', async () => {
    axiosMock.get.mockResolvedValueOnce({ data: ['file.akb'] });
    axiosMock.post.mockResolvedValueOnce({ data: { filename: 'export.akb' } });

    await backupApi.list();
    await backupApi.create('kb-1');

    expect(axiosMock.get).toHaveBeenCalledWith('/api/backups', expect.objectContaining({
      headers: expect.objectContaining({
        Authorization: 'Bearer backup-token',
      }),
    }));
    expect(axiosMock.post).toHaveBeenCalledWith('/api/backups', { kb_id: 'kb-1' }, expect.objectContaining({
      headers: expect.objectContaining({
        Authorization: 'Bearer backup-token',
      }),
    }));
  });

  it('uploads restore and preview payloads with auth headers', async () => {
    const file = new File(['zip'], 'backup.akb', { type: 'application/zip' });
    axiosMock.post.mockResolvedValue({ data: { ok: true } });

    await backupApi.restore(file);
    await backupApi.preview(file);

    expect(axiosMock.post).toHaveBeenNthCalledWith(
      1,
      '/api/backups/restore',
      expect.any(FormData),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer backup-token',
        }),
      }),
    );
    expect(axiosMock.post).toHaveBeenNthCalledWith(
      2,
      '/api/backups/preview',
      expect.any(FormData),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer backup-token',
        }),
      }),
    );
  });

  it('extracts structured backup diagnostics from API responses', () => {
    const details = extractBackupApiError({
      response: {
        status: 400,
        data: {
          error: 'The backup archive is missing meta.json.',
          code: 'missing_meta',
          details: 'Invalid backup: missing meta.json',
          hint: 'This usually means the file was not created by Aether.',
          stage: 'preview',
        },
      },
    });

    expect(details).toEqual({
      error: 'The backup archive is missing meta.json.',
      code: 'missing_meta',
      details: 'Invalid backup: missing meta.json',
      hint: 'This usually means the file was not created by Aether.',
      stage: 'preview',
      status: 400,
    });
    expect(formatBackupApiError(details)).toContain('Code: missing_meta');
  });

  it('falls back to plain string error payloads', () => {
    const details = extractBackupApiError({
      response: {
        status: 500,
        data: 'Failed to write uploaded backup: disk full',
      },
    }, 'Backup request failed.');

    expect(details.error).toBe('Failed to write uploaded backup: disk full');
    expect(details.status).toBe(500);
    expect(formatBackupApiError(details)).toContain('HTTP: 500');
  });
});
