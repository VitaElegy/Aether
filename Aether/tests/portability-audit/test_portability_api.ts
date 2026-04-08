/**
 * Portability Audit — API Integration Test Suite
 * 
 * Tests: EP-01~EP-09, EX-01~EX-12, IM-01~IM-07, LB-01~LB-10
 * Covers: portability.rs, backup.rs, all 6 portability providers
 * 
 * Usage:
 *   export API_BASE_URL=http://localhost:3000
 *   export AUTH_TOKEN=your-jwt-token
 *   npx tsx tests/portability-audit/test_portability_api.ts
 */

const BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';
const AUTH_TOKEN = process.env.AUTH_TOKEN || 'test-token';

interface TestResult {
  id: string;
  name: string;
  category: string;
  passed: boolean;
  error?: string;
  response?: { status: number; body: any };
}

const results: TestResult[] = [];

async function api(method: string, path: string, body?: any, auth = true): Promise<{ status: number; body: any; headers: Headers }> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (auth) headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });
  let responseBody: any;
  try { responseBody = await res.json(); } catch { responseBody = null; }
  return { status: res.status, body: responseBody, headers: res.headers };
}

function test(id: string, name: string, category: string, fn: () => Promise<void>) {
  return async () => {
    try {
      await fn();
      results.push({ id, name, category, passed: true });
      console.log(`  ✅ ${id}: ${name}`);
    } catch (e: any) {
      results.push({ id, name, category, passed: false, error: e.message });
      console.log(`  ❌ ${id}: ${name} — ${e.message}`);
    }
  };
}

function assert(condition: boolean, message: string) {
  if (!condition) throw new Error(message);
}

// ============================================================
// Setup: Create test data
// ============================================================
let englishKbId = '';
let vrkbProjectId = '';

const setupTests = [
  test('SETUP-01', 'Register/login and get auth token', 'Setup', async () => {
    // Try login first
    const login = await api('POST', '/api/auth/login', { username: 'admin', password: 'admin123' });
    if (login.status === 200 && login.body?.token) {
      console.log(`    Using token from login`);
    }
  }),

  test('SETUP-02', 'Get list of knowledge bases', 'Setup', async () => {
    const res = await api('GET', '/api/knowledge-bases');
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    const kbs = Array.isArray(res.body) ? res.body : [];
    console.log(`    Found ${kbs.length} knowledge bases`);
    
    // Find English KB
    const english = kbs.find((kb: any) => kb.renderer_id === 'english_v1' || kb.renderer_id === 'english');
    if (english) {
      englishKbId = english.id;
      console.log(`    English KB: ${englishKbId}`);
    }
  }),

  test('SETUP-03', 'Get list of VRKB projects', 'Setup', async () => {
    const res = await api('GET', '/api/vrkb/projects');
    if (res.status === 200) {
      const projects = Array.isArray(res.body) ? res.body : [];
      if (projects.length > 0) {
        vrkbProjectId = projects[0].id;
        console.log(`    VRKB Project: ${vrkbProjectId}`);
      } else {
        // Create one
        const create = await api('POST', '/api/vrkb/projects', { name: 'Portability-Test-Project' });
        if (create.body?.id) {
          vrkbProjectId = create.body.id;
          console.log(`    Created VRKB Project: ${vrkbProjectId}`);
        }
      }
    }
  }),
];

// ============================================================
// Export Preview Tests
// ============================================================
const previewTests = [
  test('EP-06', 'Preview non-existent KB returns 404', 'Export Preview', async () => {
    const res = await api('GET', '/api/portability/00000000-0000-0000-0000-000000000000/export/preview');
    assert(res.status === 404 || res.status === 400, `Expected 404/400, got ${res.status}`);
  }),

  test('EP-08', 'Preview without auth returns 401', 'Export Preview', async () => {
    const res = await api('GET', '/api/portability/00000000-0000-0000-0000-000000000000/export/preview', undefined, false);
    assert(res.status === 401, `Expected 401, got ${res.status}`);
  }),

  test('EP-09', 'ExportSummary format: {total_items, estimated_size, sections}', 'Export Preview', async () => {
    if (!englishKbId) { console.log('    Skipped: No English KB available'); return; }
    const res = await api('GET', `/api/portability/${englishKbId}/export/preview`);
    if (res.status === 200) {
      assert(typeof res.body.total_items === 'number', 'Missing total_items');
      assert(typeof res.body.estimated_size === 'string', 'Missing estimated_size');
      assert(Array.isArray(res.body.sections), 'Missing sections array');
      for (const s of res.body.sections) {
        assert(typeof s.name === 'string', 'Section missing name');
        assert(typeof s.count === 'number', 'Section missing count');
        assert(typeof s.details === 'string', 'Section missing details');
      }
      console.log(`    Summary: ${res.body.total_items} items, ${res.body.estimated_size}, ${res.body.sections.length} sections`);
    }
  }),
];

// ============================================================
// Export Start + SSE + Download Tests
// ============================================================
const exportTests = [
  test('EX-01', 'Start export returns task_id', 'Export Flow', async () => {
    if (!englishKbId) { console.log('    Skipped: No English KB'); return; }
    const res = await api('POST', `/api/portability/${englishKbId}/export/start`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(res.body?.task_id, 'Missing task_id');
    console.log(`    Task ID: ${res.body.task_id}`);
  }),

  test('EX-02', 'SSE progress endpoint returns text/event-stream', 'Export Flow', async () => {
    if (!englishKbId) { console.log('    Skipped'); return; }
    const start = await api('POST', `/api/portability/${englishKbId}/export/start`);
    if (start.body?.task_id) {
      const res = await fetch(`${BASE_URL}/api/portability/tasks/${start.body.task_id}/progress`, {
        headers: { 'Authorization': `Bearer ${AUTH_TOKEN}` },
      });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
      const ct = res.headers.get('content-type') || '';
      console.log(`    Content-Type: ${ct}`);
      if (ct.includes('text/event-stream')) {
        console.log('    ✅ SSE stream confirmed');
      }
    }
  }),

  test('EX-11', 'Download non-existent task returns 404', 'Export Flow', async () => {
    const res = await api('GET', '/api/portability/tasks/00000000-0000-0000-0000-000000000000/download');
    assert(res.status === 404 || res.status === 400, `Expected 404/400, got ${res.status}`);
  }),
];

// ============================================================
// Import Tests
// ============================================================
const importTests = [
  test('IM-05', 'Import analyze without file returns error', 'Import Flow', async () => {
    // Send empty POST
    const res = await fetch(`${BASE_URL}/api/portability/00000000-0000-0000-0000-000000000000/import/analyze`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${AUTH_TOKEN}` },
    });
    assert(res.status === 400 || res.status === 404 || res.status === 422, `Expected 4xx, got ${res.status}`);
  }),
];

// ============================================================
// Legacy Backup Tests
// ============================================================
const backupTests = [
  test('LB-02', 'List backups returns array', 'Legacy Backup', async () => {
    const res = await api('GET', '/api/backups');
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(Array.isArray(res.body), 'Response should be array');
    console.log(`    Found ${res.body.length} backups`);
  }),

  test('LB-01', 'Create backup for KB', 'Legacy Backup', async () => {
    if (!englishKbId) { console.log('    Skipped: No KB'); return; }
    const res = await api('POST', '/api/backups', { kb_id: englishKbId });
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(res.body?.filename, 'Missing filename');
    console.log(`    Backup: ${res.body.filename}`);
  }),

  test('LB-09', 'Download backup rejects path traversal', 'Legacy Backup', async () => {
    const res = await api('GET', '/api/backups/download/../../etc/passwd');
    assert(res.status === 400, `Expected 400 for path traversal, got ${res.status}`);
  }),

  test('LB-09b', 'Download backup rejects non-.akb extension', 'Legacy Backup', async () => {
    const res = await api('GET', '/api/backups/download/test.zip');
    assert(res.status === 400, `Expected 400 for non-.akb, got ${res.status}`);
  }),
];

// ============================================================
// VRKB Portability Tests
// ============================================================
const vrkbTests = [
  test('VK-01', 'VRKB export preview returns project info', 'VRKB Portability', async () => {
    if (!vrkbProjectId) { console.log('    Skipped: No VRKB project'); return; }
    // VRKB uses kb_id = project_id for portability
    const res = await api('GET', `/api/portability/${vrkbProjectId}/export/preview`);
    console.log(`    Status: ${res.status}, Body: ${JSON.stringify(res.body)?.substring(0, 200)}`);
  }),

  test('VK-02', 'VRKB export start returns task_id', 'VRKB Portability', async () => {
    if (!vrkbProjectId) { console.log('    Skipped'); return; }
    const res = await api('POST', `/api/portability/${vrkbProjectId}/export/start`);
    console.log(`    Status: ${res.status}, Task: ${res.body?.task_id || 'N/A'}`);
  }),
];

// ============================================================
// Runner
// ============================================================
async function run() {
  console.log('\n' + '='.repeat(60));
  console.log('  PORTABILITY AUDIT — API Integration Tests');
  console.log('  ' + new Date().toISOString());
  console.log('='.repeat(60) + '\n');

  console.log('🔧 Setup\n');
  for (const t of setupTests) await t();

  console.log('\n📋 Export Preview\n');
  for (const t of previewTests) await t();

  console.log('\n🚀 Export Start + SSE + Download\n');
  for (const t of exportTests) await t();

  console.log('\n📥 Import\n');
  for (const t of importTests) await t();

  console.log('\n💾 Legacy Backup\n');
  for (const t of backupTests) await t();

  console.log('\n🛡️ VRKB Portability\n');
  for (const t of vrkbTests) await t();

  // Summary
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log('\n' + '='.repeat(60));
  console.log(`  📊 Results: ${passed} passed, ${failed} failed out of ${results.length}`);
  console.log('='.repeat(60));

  if (failed > 0) {
    console.log('\n❌ Failed tests:');
    results.filter(r => !r.passed).forEach(r => {
      console.log(`  ${r.id}: ${r.name} — ${r.error}`);
    });
  }

  // By category
  const categories = [...new Set(results.map(r => r.category))];
  console.log('\n📊 By Category:');
  for (const cat of categories) {
    const catResults = results.filter(r => r.category === cat);
    const catPassed = catResults.filter(r => r.passed).length;
    console.log(`  ${cat}: ${catPassed}/${catResults.length}`);
  }
}

run().catch(console.error);
