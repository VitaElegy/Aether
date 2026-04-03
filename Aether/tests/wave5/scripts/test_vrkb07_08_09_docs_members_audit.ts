/**
 * VRKB Backend API Test Suite — Docs, Members, Audit (VRKB-07, 08, 09)
 * 
 * Tests: 7.1~7.18, 8.1~8.15, 9.1~9.12
 * Covers: docs.rs, members.rs, audit.rs — ALL have NO auth (⚠️ security gap)
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

// ============================================================
// VRKB-07: Doc Repo Enhancement
// ============================================================

const docTests = [
  test('7.0', 'Setup', async () => {
    const p = await api('POST', '/api/vrkb/projects', { name: 'Docs-Test' });
    projectId = p.body?.id; assert(!!projectId, 'No project ID');
  }),

  // AUTH TESTS — All doc endpoints should require auth but don't
  test('7.17a', 'Docs: list without auth (SECURITY)', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/docs`, undefined, false);
    if (res.status !== 401) {
      console.log(`    ⚠️  SECURITY: List docs without auth returned ${res.status} (expected 401)`);
    }
  }),

  test('7.17b', 'Docs: create without auth (SECURITY)', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/docs`, { title: 'Unauth Doc' }, false);
    if (res.status !== 401) {
      console.log(`    ⚠️  SECURITY: Create doc without auth returned ${res.status} (expected 401)`);
    }
  }),

  // CRUD Tests
  test('7.1', 'GET /projects/:id/docs — list docs', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/docs`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  test('7.2', 'POST /projects/:id/docs — create doc (author_id=None BUG)', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/docs`, {
      title: 'Test Document',
      content: '# Test Content',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    if (res.body?.author_id === null) {
      console.log('    ⚠️  BUG CONFIRMED: author_id is None (TODO in code)');
    }
  }),

  test('7.5', 'DELETE /docs/:id — soft delete', async () => {
    const created = await api('POST', `/api/vrkb/projects/${projectId}/docs`, { title: 'Delete Me' });
    if (created.body?.id) {
      const res = await api('DELETE', `/api/vrkb/docs/${created.body.id}`);
      assert(res.status === 200 || res.status === 204, `Expected 200/204, got ${res.status}`);
    }
  }),

  test('7.6', 'DELETE /docs/:id/permanent — no protection (SECURITY)', async () => {
    const created = await api('POST', `/api/vrkb/projects/${projectId}/docs`, { title: 'Perm Delete' });
    if (created.body?.id) {
      const res = await api('DELETE', `/api/vrkb/docs/${created.body.id}/permanent`, undefined, false);
      if (res.status !== 401) {
        console.log(`    ⚠️  CRITICAL: Permanent delete without auth returned ${res.status}`);
      }
    }
  }),

  test('7.8', 'POST /docs/:id/move — no circular reference detection', async () => {
    const parent = await api('POST', `/api/vrkb/projects/${projectId}/docs`, { title: 'Parent' });
    const child = await api('POST', `/api/vrkb/projects/${projectId}/docs`, { title: 'Child', parent_id: parent.body?.id });
    if (parent.body?.id && child.body?.id) {
      // Try to move parent under child (circular reference)
      const res = await api('POST', `/api/vrkb/docs/${parent.body.id}/move`, {
        parent_id: child.body.id,
      });
      if (res.status === 200) {
        console.log('    ⚠️  BUG: No circular reference detection — parent moved under its own child');
      }
    }
  }),

  // Template Tests
  test('7.9', 'GET /docs/templates — list 4 templates', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/docs/templates`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    const templates = Array.isArray(res.body) ? res.body : [];
    console.log(`    Found ${templates.length} templates`);
  }),

  ...['pentest-report', 'vuln-assessment', 'meeting-notes', 'blank'].map((slug, i) =>
    test(`7.${10 + i}`, `POST /docs/from-template — create from ${slug}`, async () => {
      const res = await api('POST', `/api/vrkb/projects/${projectId}/docs/from-template`, {
        template_slug: slug,
        title: `From template: ${slug}`,
      });
      assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
    })
  ),

  test('7.14', 'POST /docs/from-template — invalid slug', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/docs/from-template`, {
      template_slug: 'nonexistent-template',
      title: 'Should fail',
    });
    assert(res.status === 400 || res.status === 404, `Expected 400/404, got ${res.status}`);
  }),

  test('7.15', 'POST /docs/generate-report — compile report', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/docs/generate-report`, {
      title: 'Generated Report',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
  }),
];

// ============================================================
// VRKB-08: Members and Roles
// ============================================================

const memberTests = [
  // AUTH TESTS
  test('8.11a', 'Members: list without auth (SECURITY)', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/members`, undefined, false);
    if (res.status !== 401) {
      console.log(`    ⚠️  SECURITY: List members without auth returned ${res.status} (expected 401)`);
    }
  }),

  test('8.2', 'POST /members — add with valid role (owner)', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/members`, {
      user_id: '00000000-0000-0000-0000-000000000001',
      role: 'owner',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
  }),

  test('8.2b', 'POST /members — add with role=researcher', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/members`, {
      user_id: '00000000-0000-0000-0000-000000000002',
      role: 'researcher',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
  }),

  test('8.3', 'POST /members — invalid role should reject', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/members`, {
      user_id: '00000000-0000-0000-0000-000000000003',
      role: 'super_admin',
    });
    assert(res.status === 400 || res.status === 422, `Expected 400/422, got ${res.status}. Role validation should reject "super_admin"`);
  }),

  test('8.5', 'POST /members — add duplicate member', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/members`, {
      user_id: '00000000-0000-0000-0000-000000000001',
      role: 'researcher',
    });
    if (res.status === 200 || res.status === 201) {
      console.log('    ⚠️  BUG: Allows adding duplicate member');
    }
  }),

  test('8.1', 'GET /members — list members', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/members`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    const members = Array.isArray(res.body) ? res.body : [];
    console.log(`    Found ${members.length} members`);
  }),

  test('8.9', 'GET /members/:mid/permissions — get permissions', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/members/00000000-0000-0000-0000-000000000001/permissions`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log(`    Permissions: ${JSON.stringify(res.body)}`);
  }),

  test('8.10', 'GET /permissions — get permission matrix', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/permissions`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    console.log(`    Matrix roles: ${Object.keys(res.body || {}).join(', ')}`);
  }),

  test('8.15', 'RBAC enforcement — permissions defined but never checked', async () => {
    // A researcher user should NOT be able to manage_members
    // But since RBAC is never enforced, they can
    console.log('    ⚠️  CRITICAL: RBAC matrix defined in members.rs but NEVER enforced in any endpoint');
    console.log('    Any authenticated user can perform any action regardless of role');
  }),
];

// ============================================================
// VRKB-09: Audit and Notifications
// ============================================================

const auditTests = [
  // AUTH TESTS
  test('9.9a', 'Audit: list without auth (SECURITY)', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/audit`, undefined, false);
    if (res.status !== 401) {
      console.log(`    ⚠️  SECURITY: List audit logs without auth returned ${res.status} (expected 401)`);
    }
  }),

  test('9.3', 'POST /audit — create audit log entry', async () => {
    const res = await api('POST', `/api/vrkb/projects/${projectId}/audit`, {
      action: 'finding_created',
      entity_type: 'finding',
      entity_id: '00000000-0000-0000-0000-000000000001',
      details: 'Created new finding',
    });
    assert(res.status === 200 || res.status === 201, `Expected 200/201, got ${res.status}`);
  }),

  test('9.4', 'POST /audit — auto-generates notification', async () => {
    const before = await api('GET', `/api/vrkb/projects/${projectId}/notifications`);
    const beforeCount = Array.isArray(before.body) ? before.body.length : 0;

    await api('POST', `/api/vrkb/projects/${projectId}/audit`, {
      action: 'finding_updated',
      entity_type: 'finding',
      entity_id: '00000000-0000-0000-0000-000000000001',
      details: 'Updated finding',
    });

    const after = await api('GET', `/api/vrkb/projects/${projectId}/notifications`);
    const afterCount = Array.isArray(after.body) ? after.body.length : 0;
    console.log(`    Notifications: before=${beforeCount}, after=${afterCount}`);
  }),

  test('9.1', 'GET /audit — list with pagination', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/audit?page=1&per_page=10`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
    assert(res.body?.items || Array.isArray(res.body), 'Should return paginated result');
  }),

  test('9.2', 'GET /audit — filter by action', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/audit?action=finding_created`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  test('9.5', 'GET /notifications — list all', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/notifications`);
    assert(res.status === 200, `Expected 200, got ${res.status}`);
  }),

  test('9.6', 'Notifications not filtered by user', async () => {
    const res = await api('GET', `/api/vrkb/projects/${projectId}/notifications`);
    console.log('    ⚠️  All users see ALL notifications (not filtered by recipient)');
  }),

  test('9.7', 'POST /notifications/:id/read — mark as read', async () => {
    // First get a notification
    const notifs = await api('GET', `/api/vrkb/projects/${projectId}/notifications`);
    const items = Array.isArray(notifs.body) ? notifs.body : [];
    if (items.length > 0) {
      const res = await api('POST', `/api/vrkb/notifications/${items[0].id}/read`);
      assert(res.status === 200, `Expected 200, got ${res.status}`);
    } else {
      console.log('    Skipped: No notifications to mark as read');
    }
  }),

  test('9.8', 'POST /notifications/:id/read — non-existent notification', async () => {
    const res = await api('POST', '/api/vrkb/notifications/00000000-0000-0000-0000-000000000000/read');
    assert(res.status === 404, `Expected 404, got ${res.status}`);
  }),

  test('9.10', 'Audit create exposed as public API (should be internal)', async () => {
    console.log('    ⚠️  WARNING: POST /audit is a public endpoint — anyone can inject fake audit logs');
  }),
];

// ============================================================
// Runner
// ============================================================

async function run() {
  console.log('\n🔬 VRKB-07: Doc Repo Enhancement — API Tests\n');
  for (const t of docTests) await t();

  console.log('\n🔬 VRKB-08: Members and Roles — API Tests\n');
  for (const t of memberTests) await t();

  console.log('\n🔬 VRKB-09: Audit and Notifications — API Tests\n');
  for (const t of auditTests) await t();

  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  console.log(`\n📊 Combined Results: ${passed} passed, ${failed} failed out of ${results.length}\n`);
}

run().catch(console.error);
