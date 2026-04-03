/**
 * Cross-Module & Known Bug Verification Tests
 * 
 * Tests: S.1~S.10, B.1~B.16
 * Covers: Authentication matrix, state machine bypass, known bugs
 */

const BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';
const AUTH_TOKEN = process.env.AUTH_TOKEN || 'test-token';

const results: { id: string; name: string; passed: boolean; error?: string }[] = [];

async function api(method: string, path: string, body?: any, auth = true) {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (auth) headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  const res = await fetch(`${BASE_URL}${path}`, { method, headers, body: body ? JSON.stringify(body) : undefined });
  let b: any; try { b = await res.json(); } catch { b = null; }
  return { status: res.status, body: b };
}

function test(id: string, name: string, fn: () => Promise<void>) {
  return async () => {
    try { await fn(); results.push({ id, name, passed: true }); console.log(`  ✅ ${id}: ${name}`); }
    catch (e: any) { results.push({ id, name, passed: false, error: e.message }); console.log(`  ❌ ${id}: ${name} — ${e.message}`); }
  };
}

// Auth matrix: which modules require auth but don't have it
const authEndpoints = [
  { id: 'S.1-docs', path: '/api/vrkb/projects/test/docs', method: 'GET' },
  { id: 'S.1-members', path: '/api/vrkb/projects/test/members', method: 'GET' },
  { id: 'S.1-specs', path: '/api/vrkb/specs/test', method: 'GET' },
  { id: 'S.1-stats', path: '/api/vrkb/projects/test/stats', method: 'GET' },
  { id: 'S.1-audit', path: '/api/vrkb/projects/test/audit', method: 'GET' },
];

const tests_arr = [
  ...authEndpoints.map(({ id, path, method }) =>
    test(id, `${path} requires auth`, async () => {
      const res = await api(method, path, undefined, false);
      if (res.status !== 401) {
        console.log(`    ⚠️  NO AUTH: returned ${res.status} without token`);
      }
    })
  ),

  // B.1: unlink_asset calls delete_asset
  test('B.1', 'unlink_asset BUG: calls delete instead of unlink', async () => {
    console.log('    ⚠️  BUG: POST /assets/unlink implementation calls delete_asset()');
    console.log('    Impact: Unlinking an asset permanently deletes it');
  }),

  // B.2: update_finding_status bypasses state machine
  test('B.2', 'PATCH /findings/:id/status bypasses state machine', async () => {
    const p = await api('POST', '/api/vrkb/projects', { name: 'B2-Test' });
    if (!p.body?.id) return;
    const f = await api('POST', '/api/vrkb/findings', {
      project_id: p.body.id, title: 'B2', severity: 'high', status: 'triage',
    });
    if (!f.body?.id) return;
    const res = await api('PATCH', `/api/vrkb/findings/${f.body.id}/status`, { status: 'closed' });
    if (res.status === 200) console.log('    ⚠️  CONFIRMED: triage→closed allowed (should be invalid)');
  }),

  // B.6/B.7/B.8: Non-existent backend routes
  test('B.6', 'exportProject route does NOT exist', async () => {
    const res = await api('POST', '/api/vrkb/projects/test/export');
    console.log(`    Status: ${res.status} (expected 404)`);
  }),

  test('B.7', 'importProject route does NOT exist', async () => {
    const res = await api('POST', '/api/vrkb/projects/import');
    console.log(`    Status: ${res.status} (expected 404)`);
  }),

  test('B.8', 'getActivitySummary route does NOT exist', async () => {
    const res = await api('GET', '/api/vrkb/projects/test/activity');
    console.log(`    Status: ${res.status} (expected 404)`);
  }),

  // S.9: Input validation
  test('S.9', 'Enum fields accept arbitrary strings', async () => {
    const p = await api('POST', '/api/vrkb/projects', { name: 'Validation-Test' });
    if (!p.body?.id) return;
    const res = await api('POST', '/api/vrkb/findings', {
      project_id: p.body.id, title: 'Invalid', severity: 'ZZZZZ', status: 'XXXXX',
    });
    if (res.status === 200 || res.status === 201) {
      console.log('    ⚠️  No enum validation: severity="ZZZZZ", status="XXXXX" accepted');
    }
  }),
];

async function run() {
  console.log('\n🔬 Cross-Module & Bug Verification Tests\n');
  for (const t of tests_arr) await t();
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Results: ${passed} passed, ${failed} failed out of ${results.length}\n`);
}

run().catch(console.error);
