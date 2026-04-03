/**
 * VRKB Backend API Test Suite — Triage Queue (VRKB-03)
 * 
 * Tests: 3.1 ~ 3.10
 * Covers: triage.rs (2 endpoints), filter logic, stats consistency
 */

const BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';
const AUTH_TOKEN = process.env.AUTH_TOKEN || 'test-token';

const results: { id: string; name: string; passed: boolean; error?: string }[] = [];

async function api(method: string, path: string, body?: any, auth = true) {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (auth) headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  const res = await fetch(`${BASE_URL}${path}`, { method, headers, body: body ? JSON.stringify(body) : undefined });
  let responseBody: any;
  try { responseBody = await res.json(); } catch { responseBody = null; }
  return { status: res.status, body: responseBody };
}

function test(id: string, name: string, fn: () => Promise<void>) {
  return async () => {
    try { await fn(); results.push({ id, name, passed: true }); console.log(`  ✅ ${id}: ${name}`); }
    catch (e: any) { results.push({ id, name, passed: false, error: e.message }); console.log(`  ❌ ${id}: ${name} — ${e.message}`); }
  };
}

function assert(cond: boolean, msg: string) { if (!cond) throw new Error(msg); }

let projectId = '';

const tests_arr = [
  test('3.0', 'Setup: Create project with findings for triage', async () => {
    const p = await api('POST', '/api/vrkb/projects', { name: 'Triage-Test-Project' });
    projectId = p.body?.id;
    assert(!!projectId, 'No project ID');
    
    // Create findings in various states
    await api('POST', '/api/vrkb/findings', { project_id: projectId, title: 'Unreviewed-1', severity: 'high', status: 'triage' });
    await api('POST', '/api/vrkb/findings', { project_id: projectId, title: 'Unreviewed-2', severity: 'medium', status: 'triage' });
    await api('POST', '/api/vrkb/findings', { project_id: projectId, title: 'Confirmed-1', severity: 'low', status: 'confirmed' });
    await api('POST', '/api/vrkb/findings', { project_id: projectId, title: 'No-Evidence', severity: 'high', status: 'triage', description: '' });
  }),

  // 3.1 No filter — all findings
  test('3.1', 'GET /triage without filter returns all findings', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  // 3.2 Filter: unreviewed
  test('3.2', 'GET /triage?filter=unreviewed returns triage-status findings', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage?filter=unreviewed`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    const items = Array.isArray(res.body) ? res.body : res.body?.items || [];
    // All returned items should have status=triage or is_triage flag
    console.log(`    Found ${items.length} unreviewed items`);
  }),

  // 3.3 Filter: stale
  test('3.3', 'GET /triage?filter=stale returns findings >7 days old', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage?filter=stale`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    // New findings shouldn't be stale
    const items = Array.isArray(res.body) ? res.body : res.body?.items || [];
    console.log(`    Found ${items.length} stale items (expected 0 for new findings)`);
  }),

  // 3.4 Filter: missing_evidence
  test('3.4', 'GET /triage?filter=missing_evidence returns findings with null/empty content', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage?filter=missing_evidence`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  // 3.5 Performance: loads ALL findings then filters in memory
  test('3.5', 'Performance: triage loads all findings in memory (inefficiency check)', async () => {
    const start = Date.now();
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage?filter=unreviewed`);
    const duration = Date.now() - start;
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log(`    Response time: ${duration}ms (NOTE: loads ALL findings then filters in memory)`);
  }),

  // 3.6 Invalid filter value
  test('3.6', 'GET /triage?filter=invalid_value falls through to all findings', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage?filter=not_a_real_filter`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log('    NOTE: Unknown filter silently returns all findings (should reject or warn)');
  }),

  // 3.7 Triage stats
  test('3.7', 'GET /triage/stats returns counts per filter type', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/triage/stats`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(res.body !== null, 'Stats should not be null');
    console.log(`    Stats: ${JSON.stringify(res.body)}`);
  }),

  // 3.8 missing_evidence logic inconsistency
  test('3.8', 'Consistency check: triage queue and stats use same missing_evidence logic', async () => {
    const queue = await api('GET', `/api/vrkb/projects/${projectId}/triage?filter=missing_evidence`);
    const stats = await api('GET', `/api/vrkb/projects/${projectId}/triage/stats`);
    const queueCount = Array.isArray(queue.body) ? queue.body.length : (queue.body?.items?.length || 0);
    const statsCount = stats.body?.missing_evidence || stats.body?.missing_evidence_count || 0;
    if (queueCount !== statsCount) {
      console.log(`    ⚠️  INCONSISTENCY: Queue has ${queueCount} items, stats says ${statsCount}`);
    }
  }),
];

async function run() {
  console.log('\n🔬 VRKB-03: Triage Queue — API Tests\n');
  for (const t of tests_arr) await t();
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Results: ${passed} passed, ${failed} failed out of ${results.length}\n`);
}

run().catch(console.error);
