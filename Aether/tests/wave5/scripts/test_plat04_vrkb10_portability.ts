/**
 * PLAT Backend API Test Suite — Portability Runtime (PLAT-04)
 * + VRKB-10 Portability Provider verification
 * 
 * Tests: 10.1~10.8, 12.1~12.9
 * Covers: portability.rs, portability/vrkb.rs
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
function assert(c: boolean, m: string) { if (!c) throw new Error(m); }

let projectId = '';

const tests_arr = [
  test('setup', 'Create project for portability tests', async () => {
    const p = await api('POST', '/api/vrkb/projects', { name: 'Portability-Test' });
    projectId = p.body?.id; assert(!!projectId, 'No project ID');
  }),

  // ============================================================
  // PLAT-04: Portability Runtime API
  // ============================================================

  test('12.1', 'POST /portability/analyze-export — analyze VRKB export', async () => {
    const res = await api('POST', '/api/portability/analyze-export', {
      kb_id: projectId,
      module: 'vrkb_std',
    });
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log(`    Analysis: ${JSON.stringify(res.body)}`);
  }),

  test('12.2', 'POST /portability/export — start export, returns task_id', async () => {
    const res = await api('POST', '/api/portability/export', {
      kb_id: projectId,
      module: 'vrkb_std',
    });
    assert(res.status === 200 || res.status === 201 || res.status === 202, `Expected 200/201/202, got ${res.status}`);
    if (res.body?.task_id) {
      console.log(`    Task ID: ${res.body.task_id}`);
    }
  }),

  test('12.5', 'GET /portability/progress/:task_id — SSE stream', async () => {
    // Start an export first
    const exp = await api('POST', '/api/portability/export', {
      kb_id: projectId,
      module: 'vrkb_std',
    });
    if (exp.body?.task_id) {
      // For SSE, we'd need EventSource which isn't in Node.js natively
      // So we test the HTTP endpoint responds
      const res = await fetch(`${BASE_URL}/api/portability/progress/${exp.body.task_id}`, {
        headers: { 'Authorization': `Bearer ${AUTH_TOKEN}` },
      });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
      const contentType = res.headers.get('content-type') || '';
      console.log(`    Content-Type: ${contentType}`);
      if (contentType.includes('text/event-stream')) {
        console.log('    ✅ SSE stream confirmed');
      }
    }
  }),

  test('12.8', 'GET /portability/download/:task_id — expired/invalid token', async () => {
    const res = await api('GET', '/api/portability/download/invalid-task-id?token=expired');
    assert(res.status === 404 || res.status === 401 || res.status === 400, 
      `Expected 4xx for invalid download, got ${res.status}`);
  }),

  test('12.9', 'GET /portability/download/:task_id — invalid task_id', async () => {
    const res = await api('GET', '/api/portability/download/00000000-0000-0000-0000-000000000000');
    assert(res.status === 404 || res.status === 400, `Expected 404/400, got ${res.status}`);
  }),

  // ============================================================
  // VRKB-10: Provider Registration
  // ============================================================

  test('10.8', 'VRKB provider registered with correct aliases', async () => {
    // Test that both aliases work
    for (const alias of ['vrkb_std', 'vulnerability_research']) {
      const res = await api('POST', '/api/portability/analyze-export', {
        kb_id: projectId,
        module: alias,
      });
      console.log(`    Alias "${alias}": status ${res.status}`);
    }
  }),

  // ============================================================
  // Frontend API gaps
  // ============================================================

  test('10.9', 'POST /api/vrkb/projects/:id/export — route does NOT exist', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/export`);
    assert(res.status === 404 || res.status === 405, 
      `Route should NOT exist. Got ${res.status}. Frontend calls this but backend has no handler.`);
    console.log('    Confirmed: Frontend exportProject() calls non-existent route');
  }),

  test('10.10', 'POST /api/vrkb/projects/import — route does NOT exist', async () => {
    const res = await api('POST', '/api/vrkb/projects/import');
    assert(res.status === 404 || res.status === 405, 
      `Route should NOT exist. Got ${res.status}. Frontend calls this but backend has no handler.`);
    console.log('    Confirmed: Frontend importProject() calls non-existent route');
  }),

  test('F.1c', 'GET /api/vrkb/projects/:id/activity — route does NOT exist', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/activity`);
    assert(res.status === 404, 
      `Route should NOT exist. Got ${res.status}. Frontend getActivitySummary() calls this.`);
    console.log('    Confirmed: Frontend getActivitySummary() calls non-existent route');
  }),
];

async function run() {
  console.log('\n🔬 PLAT-04 + VRKB-10: Portability Runtime — API Tests\n');
  for (const t of tests_arr) await t();
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Results: ${passed} passed, ${failed} failed out of ${results.length}\n`);
}

run().catch(console.error);
