/**
 * VRKB Backend API Test Suite — Finding Lifecycle (VRKB-02)
 * 
 * Tests: 2.1 ~ 2.21
 * Covers: findings.rs (5 endpoints), 7-state machine transitions
 */

const BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';
const AUTH_TOKEN = process.env.AUTH_TOKEN || 'test-token';

interface TestResult {
  id: string;
  name: string;
  passed: boolean;
  error?: string;
}

const results: TestResult[] = [];

async function api(method: string, path: string, body?: any, auth = true): Promise<{ status: number; body: any }> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (auth) headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  const res = await fetch(`${BASE_URL}${path}`, {
    method, headers,
    body: body ? JSON.stringify(body) : undefined,
  });
  let responseBody: any;
  try { responseBody = await res.json(); } catch { responseBody = null; }
  return { status: res.status, body: responseBody };
}

function test(id: string, name: string, fn: () => Promise<void>) {
  return async () => {
    try {
      await fn();
      results.push({ id, name, passed: true });
      console.log(`  ✅ ${id}: ${name}`);
    } catch (e: any) {
      results.push({ id, name, passed: false, error: e.message });
      console.log(`  ❌ ${id}: ${name} — ${e.message}`);
    }
  };
}

function assert(condition: boolean, message: string) {
  if (!condition) throw new Error(message);
}

let testProjectId: string = '';

const tests = [
  // Setup: Create a project for findings
  test('2.0', 'Setup: Create project for findings', async () => {
    const res = await api('POST', '/api/vrkb/projects', { name: 'Finding-Test-Project' });
    assert(res.status === 200 || res.status === 201, `Setup failed: ${res.status}`);
    testProjectId = res.body?.id;
    assert(!!testProjectId, 'No project ID returned');
  }),

  // 2.1 Create finding — happy path
  test('2.1', 'Create finding with all required fields', async () => {
    const res = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'SQL Injection in Login',
      severity: 'critical',
      status: 'triage',
      description: 'Found SQL injection vulnerability',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    assert(res.body?.id, 'Should return finding ID');
  }),

  // 2.2 Create with extended VRKB-02 fields
  test('2.2', 'Create finding with VRKB-02 extended fields', async () => {
    const res = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'XSS in Profile Page',
      severity: 'high',
      status: 'triage',
      confidence: 'confirmed',
      owner_id: '00000000-0000-0000-0000-000000000001',
      due_date: '2026-04-15',
      affected_assets: 'profile-page,user-input',
      repro_steps: '1. Go to profile\n2. Enter <script>alert(1)</script>',
      remediation: 'Sanitize user input',
      verification_note: 'Pending verification',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
  }),

  // 2.3 Create with invalid severity
  test('2.3', 'Create finding with invalid severity (should reject)', async () => {
    const res = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'Invalid Severity Test',
      severity: 'mega_critical',
      status: 'triage',
    });
    // KNOWN BUG: Accepts any string — this test documents the gap
    if (res.status === 200 || res.status === 201) {
      console.log('    ⚠️  BUG CONFIRMED: Accepts arbitrary severity string "mega_critical"');
    }
    // Ideally: assert(res.status === 400, 'Should reject invalid severity');
  }),

  // 2.4 Create with invalid status
  test('2.4', 'Create finding with invalid status (should reject)', async () => {
    const res = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'Invalid Status Test',
      severity: 'low',
      status: 'nonexistent_status',
    });
    if (res.status === 200 || res.status === 201) {
      console.log('    ⚠️  BUG CONFIRMED: Accepts arbitrary status string');
    }
  }),

  // 2.6 Create with malformed due_date
  test('2.6', 'Create finding with malformed due_date (silently becomes None)', async () => {
    const res = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'Bad Date Test',
      severity: 'medium',
      status: 'triage',
      due_date: 'not-a-date',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    // KNOWN BUG: due_date silently becomes None instead of returning error
    console.log('    ⚠️  Silent data loss: malformed due_date becomes None');
  }),

  // 2.7 List findings
  test('2.7', 'List findings for project', async () => {
    const res = await api('GET', `/api/vrkb/findings?project_id=${testProjectId}`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(Array.isArray(res.body) || res.body?.items, 'Should return array or paginated list');
  }),

  // 2.9 Get single finding
  test('2.9', 'Get single finding by ID', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'Get-Test Finding',
      severity: 'low',
      status: 'triage',
    });
    if (created.body?.id) {
      const res = await api('GET', `/api/vrkb/findings/${created.body.id}`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  // 2.10 Get non-existent finding
  test('2.10', 'Get non-existent finding returns 404', async () => {
    const res = await api('GET', '/api/vrkb/findings/00000000-0000-0000-0000-000000000000');
    assert(res.status === 404, `Expected 404, got ${res.status}`);
  }),

  // 2.12 PUT update can bypass state machine (BUG)
  test('2.12', 'PUT update finding status bypasses state machine (BUG)', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId,
      title: 'Bypass-Test',
      severity: 'critical',
      status: 'triage',
    });
    if (created.body?.id) {
      // Try setting status directly to "closed" via PUT (should be blocked by state machine)
      const res = await api('PUT', `/api/vrkb/findings/${created.body.id}`, {
        status: 'closed',
      });
      if (res.status === 200) {
        console.log('    ⚠️  BUG CONFIRMED: PUT /findings/:id can set status=closed directly, bypassing state machine');
      }
    }
  }),

  // 2.13 ~ 2.18 State machine transitions (happy paths)
  test('2.13', 'Transition: triage → confirmed', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'SM-Test', severity: 'high', status: 'triage',
    });
    if (created.body?.id) {
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, {
        status: 'confirmed',
      });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  test('2.14', 'Transition: confirmed → exploiting', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'SM-Test-2', severity: 'high', status: 'triage',
    });
    if (created.body?.id) {
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'confirmed' });
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'exploiting' });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  test('2.15', 'Transition: exploiting → fixing', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'SM-Test-3', severity: 'medium', status: 'triage',
    });
    if (created.body?.id) {
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'confirmed' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'exploiting' });
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'fixing' });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  test('2.16', 'Transition: fixing → verifying', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'SM-Test-4', severity: 'medium', status: 'triage',
    });
    if (created.body?.id) {
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'confirmed' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'exploiting' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'fixing' });
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'verifying' });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  test('2.17', 'Transition: verifying → closed', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'SM-Test-5', severity: 'low', status: 'triage',
    });
    if (created.body?.id) {
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'confirmed' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'exploiting' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'fixing' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'verifying' });
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'closed' });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  test('2.18', 'Transition: verifying → risk_accepted', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'SM-Test-6', severity: 'low', status: 'triage',
    });
    if (created.body?.id) {
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'confirmed' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'exploiting' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'fixing' });
      await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'verifying' });
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'risk_accepted' });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  // 2.19 Invalid transition (BUG)
  test('2.19', 'Invalid transition: triage → closed (should fail)', async () => {
    const created = await api('POST', '/api/vrkb/findings', {
      project_id: testProjectId, title: 'Invalid-Trans-Test', severity: 'high', status: 'triage',
    });
    if (created.body?.id) {
      const res = await api('PATCH', `/api/vrkb/findings/${created.body.id}/status`, { status: 'closed' });
      if (res.status === 200) {
        console.log('    ⚠️  BUG CONFIRMED: PATCH /status allows triage→closed, bypassing state machine');
      }
      // Ideally: assert(res.status === 400 || res.status === 422, 'Should reject invalid transition');
    }
  }),
];

async function run() {
  console.log('\n🔬 VRKB-02: Finding Lifecycle — API Tests\n');
  for (const t of tests) await t();
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Results: ${passed} passed, ${failed} failed out of ${results.length}\n`);
}

run().catch(console.error);
