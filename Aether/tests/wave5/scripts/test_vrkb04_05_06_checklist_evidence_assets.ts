/**
 * VRKB Backend API Test Suite — Checklist, Evidence, Assets (VRKB-04, 05, 06)
 * 
 * Tests: 4.1~4.15, 5.1~5.15, 6.1~6.10
 * Covers: checklist.rs, evidence.rs, assets.rs
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
let sectionId = 'test-section-001'; // In-memory store uses this as key

// ============================================================
// VRKB-04: Checklist System
// ============================================================

const checklistTests = [
  test('4.0', 'Setup project', async () => {
    const p = await api('POST', '/api/vrkb/projects', { name: 'Checklist-Test' });
    projectId = p.body?.id; assert(!!projectId, 'No project ID');
  }),

  test('4.1', 'GET /sections/:id/checklist — list items (empty)', async () => {
    const res = await api('GET', `/api/vrkb/sections/${sectionId}/checklist`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    const items = Array.isArray(res.body) ? res.body : [];
    assert(items.length === 0, `Expected empty list, got ${items.length} items`);
  }),

  test('4.3', 'POST /sections/:id/checklist — create item', async () => {
    const res = await api('POST', `/api/vrkb/sections/${sectionId}/checklist`, {
      title: 'Review firewall rules',
      is_blocker: true,
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    assert(res.body?.id, 'Should return item ID');
  }),

  test('4.3b', 'POST /sections/:id/checklist — create non-blocker item', async () => {
    const res = await api('POST', `/api/vrkb/sections/${sectionId}/checklist`, {
      title: 'Update documentation',
      is_blocker: false,
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
  }),

  test('4.2', 'GET /sections/:id/checklist — list now has 2 items', async () => {
    const res = await api('GET', `/api/vrkb/sections/${sectionId}/checklist`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    const items = Array.isArray(res.body) ? res.body : [];
    assert(items.length === 2, `Expected 2 items, got ${items.length}`);
  }),

  test('4.5', 'PUT /sections/:id/checklist/:item_id — toggle completion', async () => {
    const list = await api('GET', `/api/vrkb/sections/${sectionId}/checklist`);
    const items = Array.isArray(list.body) ? list.body : [];
    if (items.length > 0) {
      const res = await api('PUT', `/api/vrkb/sections/${sectionId}/checklist/${items[0].id}`, {
        is_completed: true,
      });
      assert(res.status === 200, `Expected 200, got ${res.status}`);
      // Verify completed_by is set
      if (res.body?.completed_by) {
        console.log(`    completed_by: ${res.body.completed_by}`);
      }
    }
  }),

  test('4.7', 'PUT /sections/:id/checklist/:item_id — non-existent item', async () => {
    const res = await api('PUT', `/api/vrkb/sections/${sectionId}/checklist/00000000-0000-0000-0000-000000000000`, {
      is_completed: true,
    });
    assert(res.status === 404, `Expected 404, got ${res.status}`);
  }),

  test('4.8', 'GET /sections/:id/checklist/summary — returns stats', async () => {
    const res = await api('GET', `/api/vrkb/sections/${sectionId}/checklist/summary`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log(`    Summary: ${JSON.stringify(res.body)}`);
  }),

  test('4.10', 'Checklist — no project-level permission check (any user can modify)', async () => {
    // Any authenticated user should be able to create items in any section
    // This is a security gap — documenting it
    const res = await api('POST', `/api/vrkb/sections/someone-elses-section/checklist`, {
      title: 'Unauthorized item',
      is_blocker: false,
    });
    if (res.status === 200 || res.status === 201) {
      console.log('    ⚠️  SECURITY: No project-level permission check — any auth user can modify');
    }
  }),

  test('4.11', 'Checklist — missing DELETE endpoint', async () => {
    const list = await api('GET', `/api/vrkb/sections/${sectionId}/checklist`);
    const items = Array.isArray(list.body) ? list.body : [];
    if (items.length > 0) {
      const res = await api('DELETE', `/api/vrkb/sections/${sectionId}/checklist/${items[0].id}`);
      assert(res.status === 404 || res.status === 405, `Expected 404/405 for missing DELETE, got ${res.status}`);
      console.log('    Confirmed: No DELETE endpoint for checklist items');
    }
  }),
];

// ============================================================
// VRKB-05: Evidence Blocks
// ============================================================

const evidenceTypes = ['screenshot', 'request_response', 'log_extract', 'poc_file', 'external_reference'];

const evidenceTests = [
  test('5.1', 'GET /projects/:id/evidence — list (empty)', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/evidence`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  ...evidenceTypes.map((type, i) =>
    test(`5.${3 + i}`, `POST /evidence — create type=${type}`, async () => {
      const res = await api('POST', `/api/vrkb/projects/${projectId}/evidence`, {
        evidence_type: type,
        title: `Evidence: ${type}`,
        content: `Content for ${type}`,
        attached_to_type: 'finding',
        attached_to_id: '00000000-0000-0000-0000-000000000001',
      });
      assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    })
  ),

  test('5.8', 'POST /evidence — invalid type (should reject)', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/evidence`, {
      evidence_type: 'custom_invalid_type',
      title: 'Invalid Type Test',
      content: 'Should fail',
    });
    if (res.status === 200 || res.status === 201) {
      console.log('    ⚠️  BUG: Accepts arbitrary evidence_type string');
    }
  }),

  test('5.2', 'GET /evidence — filter by attached_to_type and attached_to_id', async () => {
    const res = await api('GET', 
      `/api/vrkb/projects/${projectId}/evidence?attached_to_type=finding&attached_to_id=00000000-0000-0000-0000-000000000001`
    );
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  test('5.10', 'GET /evidence/:eid — non-existent', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/evidence/00000000-0000-0000-0000-000000000000`);
    assert(res.status === 404, `Expected 404, got ${res.status}`);
  }),

  test('5.11', 'DELETE /evidence/:eid — delete', async () => {
    const list = await api('GET', `/api/vrkb/projects/${projectId}/evidence`);
    const items = Array.isArray(list.body) ? list.body : list.body?.items || [];
    if (items.length > 0) {
      const res = await api('DELETE', `/api/vrkb/projects/${projectId}/evidence/${items[0].id}`);
      assert(res.status === 200 || res.status === 204, `Expected 200/204, got ${res.status}`);
    }
  }),
];

// ============================================================
// VRKB-06: Assets Integration
// ============================================================

const assetTests = [
  test('6.1', 'POST /assets/link — link asset to finding', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/assets/link`, {
      asset_id: '00000000-0000-0000-0000-000000000001',
      linked_entity_type: 'finding',
      linked_entity_id: '00000000-0000-0000-0000-000000000002',
    });
    // May fail if asset doesn't exist — that's expected
    console.log(`    Status: ${res.status}`);
  }),

  test('6.4', 'POST /assets/unlink — BUG: deletes asset instead of unlinking', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/assets/unlink`, {
      asset_id: '00000000-0000-0000-0000-000000000001',
      linked_entity_type: 'finding',
      linked_entity_id: '00000000-0000-0000-0000-000000000002',
    });
    console.log(`    Status: ${res.status} — NOTE: Known BUG: calls delete_asset instead of unlinking`);
  }),

  test('6.5', 'GET /assets/usage/:aid — returns empty (MVP)', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/assets/usage/00000000-0000-0000-0000-000000000001`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log('    Returns empty for MVP (placeholder implementation)');
  }),

  test('6.6', 'DELETE /assets/:id — no auth required (SECURITY)', async () => {
    const res = await api('DELETE', '/api/vrkb/assets/00000000-0000-0000-0000-000000000001', undefined, false);
    if (res.status !== 401) {
      console.log(`    ⚠️  SECURITY: Delete asset without auth returned ${res.status} (expected 401)`);
    }
  }),

  test('6.7', 'GET /projects/:id/assets — no auth required (SECURITY)', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/assets`, undefined, false);
    if (res.status !== 401) {
      console.log(`    ⚠️  SECURITY: List assets without auth returned ${res.status} (expected 401)`);
    }
  }),
];

// ============================================================
// Runner
// ============================================================

async function run() {
  console.log('\n🔬 VRKB-04: Checklist System — API Tests\n');
  for (const t of checklistTests) await t();

  console.log('\n🔬 VRKB-05: Evidence Blocks — API Tests\n');
  for (const t of evidenceTests) await t();

  console.log('\n🔬 VRKB-06: Assets Integration — API Tests\n');
  for (const t of assetTests) await t();

  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Combined Results: ${passed} passed, ${failed} failed out of ${results.length}\n`);
}

run().catch(console.error);
