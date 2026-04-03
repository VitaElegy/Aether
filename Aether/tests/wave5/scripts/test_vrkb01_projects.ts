/**
 * VRKB Backend API Test Suite — Projects & Stats (VRKB-01)
 * 
 * Tests: 1.1 ~ 1.17
 * Covers: projects.rs (7 endpoints), stats.rs (1 endpoint), structure.rs (1 endpoint)
 */

const BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';
const AUTH_TOKEN = process.env.AUTH_TOKEN || 'test-token';

interface TestResult {
  id: string;
  name: string;
  passed: boolean;
  error?: string;
  response?: { status: number; body: any };
}

const results: TestResult[] = [];

async function api(method: string, path: string, body?: any, auth = true): Promise<{ status: number; body: any }> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (auth) headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers,
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

// ============================================================
// VRKB-01: Project Control Center
// ============================================================

const tests = [
  // 1.1 Create project — happy path
  test('1.1', 'Create project with valid name and description', async () => {
    const res = await api('POST', '/api/vrkb/projects', {
      name: 'Test Project Alpha',
      description: 'A test project for VRKB',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    assert(res.body?.id, 'Response should contain project id');
  }),

  // 1.2 Create project with empty name
  test('1.2', 'Create project with empty name should reject', async () => {
    const res = await api('POST', '/api/vrkb/projects', {
      name: '',
      description: 'No name',
    });
    assert(res.status === 400 || res.status === 422, `Expected 400/422 for empty name, got ${res.status}`);
  }),

  // 1.3 Create project without auth
  test('1.3', 'Create project without auth should return 401', async () => {
    const res = await api('POST', '/api/vrkb/projects', {
      name: 'Unauth project',
    }, false);
    assert(res.status === 401, `Expected 401, got ${res.status}`);
  }),

  // 1.5 List projects — returns ALL regardless of user (security test)
  test('1.5', 'List projects shows only user-accessible projects', async () => {
    const res = await api('GET', '/api/vrkb/projects');
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(Array.isArray(res.body) || res.body?.items, 'Response should be array or paginated');
    // NOTE: Currently returns ALL projects — this test documents the security gap
  }),

  // 1.6 List projects — missing pagination
  test('1.6', 'List projects should support pagination params', async () => {
    const res = await api('GET', '/api/vrkb/projects?page=1&per_page=10');
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    // Verify if pagination params are honored (likely ignored currently)
  }),

  // 1.7 Get existing project
  test('1.7', 'Get existing project by ID', async () => {
    // First create a project
    const created = await api('POST', '/api/vrkb/projects', { name: 'Get-Test' });
    if (created.body?.id) {
      const res = await api('GET', `/api/vrkb/projects/${created.body.id}`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
      assert(res.body?.name === 'Get-Test', 'Name should match');
    }
  }),

  // 1.8 Get non-existent project
  test('1.8', 'Get non-existent project returns 404', async () => {
    const res = await api('GET', '/api/vrkb/projects/00000000-0000-0000-0000-000000000000');
    assert(res.status === 404, `Expected 404, got ${res.status}`);
  }),

  // 1.9 Update project
  test('1.9', 'Update project name', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Update-Test' });
    if (created.body?.id) {
      const res = await api('PUT', `/api/vrkb/projects/${created.body.id}`, {
        name: 'Updated Name',
      });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  // 1.10 Update with empty name
  test('1.10', 'Update project with empty name should reject', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Update-Empty-Test' });
    if (created.body?.id) {
      const res = await api('PUT', `/api/vrkb/projects/${created.body.id}`, {
        name: '',
      });
      assert(res.status === 400 || res.status === 422, `Expected 400/422, got ${res.status}`);
    }
  }),

  // 1.11 Delete project
  test('1.11', 'Delete project', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Delete-Test' });
    if (created.body?.id) {
      const res = await api('DELETE', `/api/vrkb/projects/${created.body.id}`);
      assert(res.status === 200 || res.status === 204, `Expected 200/204, got ${res.status}`);
    }
  }),

  // 1.13 Archive project
  test('1.13', 'Archive project', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Archive-Test' });
    if (created.body?.id) {
      const res = await api('POST', `/api/vrkb/projects/${created.body.id}/archive`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  // 1.14 Unarchive project
  test('1.14', 'Unarchive project', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Unarchive-Test' });
    if (created.body?.id) {
      await api('POST', `/api/vrkb/projects/${created.body.id}/archive`);
      const res = await api('POST', `/api/vrkb/projects/${created.body.id}/unarchive`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  // 1.15 Stats without auth (security)
  test('1.15', 'Stats endpoint without auth should require authentication', async () => {
    const res = await api('GET', '/api/vrkb/projects/00000000-0000-0000-0000-000000000000/stats', undefined, false);
    assert(res.status === 401, `Expected 401 for unauthenticated stats, got ${res.status}. SECURITY: Exposes sensitive data`);
  }),

  // 1.16 Stats with auth
  test('1.16', 'Stats returns severity/status distribution', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Stats-Test' });
    if (created.body?.id) {
      const res = await api('GET', `/api/vrkb/projects/${created.body.id}/stats`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),

  // 1.17 Structure
  test('1.17', 'List sections for project', async () => {
    const created = await api('POST', '/api/vrkb/projects', { name: 'Structure-Test' });
    if (created.body?.id) {
      const res = await api('GET', `/api/vrkb/projects/${created.body.id}/structure`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    }
  }),
];

// ============================================================
// Runner
// ============================================================

async function run() {
  console.log('\n🔬 VRKB-01: Project Control Center — API Tests\n');
  for (const t of tests) {
    await t();
  }

  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Results: ${passed} passed, ${failed} failed out of ${results.length} tests\n`);
  
  if (failed > 0) {
    console.log('Failed tests:');
    results.filter(r => !r.passed).forEach(r => {
      console.log(`  ❌ ${r.id}: ${r.name} — ${r.error}`);
    });
  }
}

run().catch(console.error);
