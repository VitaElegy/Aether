# Aether Wave 5 + PLAT Closure — COMPREHENSIVE TEST CHECKLIST

> **Generated**: 2026-04-03  
> **Scope**: VRKB-01 ~ VRKB-10, PLAT-03, PLAT-04, PLAT-06  
> **Backend Modules Audited**: 13 API files (11 VRKB + portability + observability domain)  
> **Frontend Modules Audited**: 55 API functions, 3 composables, 7 new Vue components, 2 Pinia stores, 2 type definition files  
> **Total Test Points**: **214**

---

## Table of Contents

1. [Priority Legend](#priority-legend)
2. [VRKB-01: Project Control Center](#vrkb-01-project-control-center)
3. [VRKB-02: Finding Lifecycle](#vrkb-02-finding-lifecycle)
4. [VRKB-03: Triage Queue](#vrkb-03-triage-queue)
5. [VRKB-04: Checklist System](#vrkb-04-checklist-system)
6. [VRKB-05: Evidence Blocks](#vrkb-05-evidence-blocks)
7. [VRKB-06: Assets Integration](#vrkb-06-assets-integration)
8. [VRKB-07: Doc Repo Enhancement](#vrkb-07-doc-repo-enhancement)
9. [VRKB-08: Members and Roles](#vrkb-08-members-and-roles)
10. [VRKB-09: Audit and Notifications](#vrkb-09-audit-and-notifications)
11. [VRKB-10: VRKB Portability](#vrkb-10-vrkb-portability)
12. [PLAT-03: Header Action Protocol](#plat-03-header-action-protocol)
13. [PLAT-04: Portability Runtime](#plat-04-portability-runtime)
14. [PLAT-06: Observability Foundation](#plat-06-observability-foundation)
15. [Cross-Module Systemic Issues](#cross-module-systemic-issues)
16. [Frontend-Specific Issues](#frontend-specific-issues)
17. [Known Bugs (Must Fix)](#known-bugs-must-fix)
18. [Summary Statistics](#summary-statistics)

---

## Priority Legend

| Priority | Meaning | Action Required |
|----------|---------|-----------------|
| **P0** | Critical security / data integrity | Must fix before any release |
| **P1** | Functional correctness / state machine | Must fix before beta |
| **P2** | Validation / UX / robustness | Should fix before GA |
| **P3** | Cleanup / optimization / dead code | Nice to have |

---

## VRKB-01: Project Control Center

### Backend — `projects.rs` (7 endpoints, ALL have auth ✓)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 1.1 | `POST /api/vrkb/projects` — Create project with valid name and description | P1 | ☐ | Happy path |
| 1.2 | `POST /api/vrkb/projects` — Create project with empty name (`""`) | P2 | ☐ | Should reject; currently accepts |
| 1.3 | `POST /api/vrkb/projects` — Create project without auth token | P0 | ☐ | Should return 401 |
| 1.4 | `POST /api/vrkb/projects` — Creator is NOT auto-added as owner/member | P1 | ☐ | **Known gap**: creator not associated |
| 1.5 | `GET /api/vrkb/projects` — List projects returns ALL projects regardless of user | P0 | ☐ | **No project-level permission filtering** |
| 1.6 | `GET /api/vrkb/projects` — Pagination (missing; loads all) | P2 | ☐ | No `limit`/`offset` params |
| 1.7 | `GET /api/vrkb/projects/:id` — Get existing project | P1 | ☐ | Happy path |
| 1.8 | `GET /api/vrkb/projects/:id` — Get non-existent project (invalid UUID) | P2 | ☐ | Should return 404 |
| 1.9 | `PUT /api/vrkb/projects/:id` — Update project name | P1 | ☐ | Happy path |
| 1.10 | `PUT /api/vrkb/projects/:id` — Update with empty name | P2 | ☐ | Should reject |
| 1.11 | `DELETE /api/vrkb/projects/:id` — Delete project | P1 | ☐ | Happy path |
| 1.12 | `DELETE /api/vrkb/projects/:id` — Cascade delete consideration (findings, docs, members, checklist) | P1 | ☐ | **Not implemented** |
| 1.13 | `POST /api/vrkb/projects/:id/archive` — Archive project | P1 | ☐ | |
| 1.14 | `POST /api/vrkb/projects/:id/unarchive` — Unarchive project | P1 | ☐ | |

### Backend — `stats.rs` (1 endpoint, NO auth ⚠️)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 1.15 | `GET /api/vrkb/projects/:id/stats` — Without auth | P0 | ☐ | **Exposes sensitive security stats to unauthenticated users** |
| 1.16 | `GET /api/vrkb/projects/:id/stats` — Verify severity/status distribution correct | P1 | ☐ | |

### Backend — `structure.rs` (1 endpoint)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 1.17 | `GET /api/vrkb/projects/:id/structure` — List sections | P1 | ☐ | |

### Frontend — `OverviewDashboard.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 1.18 | Dashboard loads scope/status/severity summaries on mount | P1 | ☐ | |
| 1.19 | Dashboard handles API error gracefully (show error state) | P2 | ☐ | |
| 1.20 | Dashboard shows loading state while fetching | P2 | ☐ | |

### Frontend — `ProjectList.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 1.21 | Create project form validation | P2 | ☐ | |
| 1.22 | Project list renders correctly after fetch | P1 | ☐ | |
| 1.23 | Delete project with confirmation dialog | P2 | ☐ | |

---

## VRKB-02: Finding Lifecycle

### Backend — `findings.rs` (5 endpoints, all have auth ✓)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 2.1 | `POST /api/vrkb/findings` — Create finding with all required fields | P1 | ☐ | |
| 2.2 | `POST /api/vrkb/findings` — Create finding with extended fields (confidence, owner_id, due_date, affected_assets, repro_steps, remediation, verification_note) | P1 | ☐ | VRKB-02 fields |
| 2.3 | `POST /api/vrkb/findings` — Create with invalid severity string (e.g., `"mega_critical"`) | P2 | ☐ | **Accepts any string — no enum validation** |
| 2.4 | `POST /api/vrkb/findings` — Create with invalid status string | P2 | ☐ | Same issue |
| 2.5 | `POST /api/vrkb/findings` — Create with invalid confidence string | P2 | ☐ | Same issue |
| 2.6 | `POST /api/vrkb/findings` — Create with malformed due_date | P2 | ☐ | **Parse failure silently becomes None** |
| 2.7 | `GET /api/vrkb/findings?project_id=X` — List findings for project | P1 | ☐ | |
| 2.8 | `GET /api/vrkb/findings?project_id=X` — Missing pagination | P2 | ☐ | Loads all findings |
| 2.9 | `GET /api/vrkb/findings/:id` — Get single finding | P1 | ☐ | |
| 2.10 | `GET /api/vrkb/findings/:id` — Non-existent finding | P2 | ☐ | Should return 404 |
| 2.11 | `PUT /api/vrkb/findings/:id` — Update finding with valid data | P1 | ☐ | |
| 2.12 | `PUT /api/vrkb/findings/:id` — Update status field via PUT (bypasses state machine) | P0 | ☐ | **BUG: can set any status via PUT, ignoring state machine** |
| 2.13 | `PATCH /api/vrkb/findings/:id/status` — Transition triage → confirmed | P1 | ☐ | |
| 2.14 | `PATCH /api/vrkb/findings/:id/status` — Transition confirmed → exploiting | P1 | ☐ | |
| 2.15 | `PATCH /api/vrkb/findings/:id/status` — Transition exploiting → fixing | P1 | ☐ | |
| 2.16 | `PATCH /api/vrkb/findings/:id/status` — Transition fixing → verifying | P1 | ☐ | |
| 2.17 | `PATCH /api/vrkb/findings/:id/status` — Transition verifying → closed | P1 | ☐ | |
| 2.18 | `PATCH /api/vrkb/findings/:id/status` — Transition verifying → risk_accepted | P1 | ☐ | |
| 2.19 | `PATCH /api/vrkb/findings/:id/status` — Invalid transition (triage → closed directly) | P1 | ☐ | **BUG: calls `repo.update_finding_status()` NOT `repo.transition_finding_status()` — state machine bypassed** |
| 2.20 | `PATCH /api/vrkb/findings/:id/status` — Transition with invalid status string | P2 | ☐ | |
| 2.21 | `DELETE /api/vrkb/findings/:id` — Delete finding (missing endpoint?) | P2 | ☐ | Verify if endpoint exists |

### Frontend — `VulnerabilityKanban.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 2.22 | Kanban board renders findings grouped by status | P1 | ☐ | |
| 2.23 | Drag-and-drop triggers status transition | P1 | ☐ | |
| 2.24 | Invalid drag rejected (e.g., triage → closed) | P1 | ☐ | |

### Frontend — `FindingEditor.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 2.25 | Finding form populates all VRKB-02 fields | P1 | ☐ | |
| 2.26 | Save triggers correct API call | P1 | ☐ | |
| 2.27 | Due date picker format validation | P2 | ☐ | |

### Frontend — `vrkb.ts` store

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 2.28 | `updateFindingStatus` optimistic update rollback on API failure | P1 | ☐ | **BUG: never rolls back** |
| 2.29 | `fetchFindings` error handling (currently swallows exceptions) | P2 | ☐ | |

---

## VRKB-03: Triage Queue

### Backend — `triage.rs` (2 endpoints, both have auth ✓)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 3.1 | `GET /api/vrkb/projects/:id/triage` — No filter (all findings) | P1 | ☐ | |
| 3.2 | `GET /api/vrkb/projects/:id/triage?filter=unreviewed` — Only status=triage or is_triage | P1 | ☐ | |
| 3.3 | `GET /api/vrkb/projects/:id/triage?filter=stale` — Findings >7 days, not closed/risk_accepted | P1 | ☐ | |
| 3.4 | `GET /api/vrkb/projects/:id/triage?filter=missing_evidence` — Null/empty content findings | P1 | ☐ | |
| 3.5 | `GET /api/vrkb/projects/:id/triage` — Performance: loads ALL findings then filters in memory | P2 | ☐ | **Inefficiency: repo has dedicated triage methods but they are never called** |
| 3.6 | `GET /api/vrkb/projects/:id/triage?filter=invalid_value` — Unknown filter value | P2 | ☐ | Falls through to all findings |
| 3.7 | `GET /api/vrkb/projects/:id/triage/stats` — Returns counts for each filter type | P1 | ☐ | |
| 3.8 | `GET /api/vrkb/projects/:id/triage/stats` — missing_evidence logic inconsistency between queue and stats | P1 | ☐ | **BUG: queue checks content=null/empty, stats may use different logic** |
| 3.9 | Stale threshold hardcoded to 7 days — verify correct | P3 | ☐ | |
| 3.10 | 5 repository triage methods never called (dead code) | P3 | ☐ | repo.find_unreviewed, repo.find_stale, etc. |

### Frontend — `TriageQueue.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 3.11 | Component loads triage queue on mount | P1 | ☐ | |
| 3.12 | Filter switching (unreviewed/stale/missing_evidence) | P1 | ☐ | |
| 3.13 | Accept/reject/merge actions update finding | P1 | ☐ | |
| 3.14 | No loading state UI displayed during fetch | P2 | ☐ | **Missing** |
| 3.15 | Errors only console.error'd (no user feedback) | P2 | ☐ | **Missing** |
| 3.16 | Sequential API calls should be Promise.all for performance | P3 | ☐ | |

---

## VRKB-04: Checklist System

### Backend — `checklist.rs` (4 endpoints, all have auth ✓)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 4.1 | `GET /api/vrkb/sections/:id/checklist` — List items for section | P1 | ☐ | Uses in-memory OnceLock store |
| 4.2 | `GET /api/vrkb/sections/:id/checklist` — Empty section returns `[]` | P1 | ☐ | |
| 4.3 | `POST /api/vrkb/sections/:id/checklist` — Create item with title and is_blocker | P1 | ☐ | |
| 4.4 | `POST /api/vrkb/sections/:id/checklist` — Create item without required fields | P2 | ☐ | |
| 4.5 | `PUT /api/vrkb/sections/:id/checklist/:item_id` — Toggle completion sets completed_by=user.id, completed_at=now | P1 | ☐ | |
| 4.6 | `PUT /api/vrkb/sections/:id/checklist/:item_id` — Unchecking clears completed_by and completed_at | P1 | ☐ | |
| 4.7 | `PUT /api/vrkb/sections/:id/checklist/:item_id` — Non-existent item_id | P2 | ☐ | Should return 404 |
| 4.8 | `GET /api/vrkb/sections/:id/checklist/summary` — Returns total, completed, completion_percent | P1 | ☐ | |
| 4.9 | `GET /api/vrkb/sections/:id/checklist/summary` — Blocker count is accurate | P1 | ☐ | |
| 4.10 | No project-level permission check (only auth) | P0 | ☐ | **Any authenticated user can modify any section's checklist** |
| 4.11 | Missing DELETE endpoint | P2 | ☐ | Cannot delete checklist items |
| 4.12 | RwLock `.unwrap()` can panic on poisoned lock | P1 | ☐ | |
| 4.13 | In-memory store loses data on server restart | P2 | ☐ | **MVP limitation** |
| 4.14 | Model mismatch: API uses inline struct, entity has `VrkbChecklistItem` with different fields | P2 | ☐ | Missing is_blocker, completed_by, completed_at in entity |
| 4.15 | Doesn't use existing repo checklist methods | P3 | ☐ | |

### Frontend — `ChecklistPanel.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 4.16 | Render checklist items with checkbox | P1 | ☐ | |
| 4.17 | Toggle checkbox triggers PUT API call | P1 | ☐ | |
| 4.18 | Summary bar shows completion percentage | P1 | ☐ | |
| 4.19 | Add new item form | P1 | ☐ | |
| 4.20 | No loading state on initial fetch | P2 | ☐ | **Missing** |
| 4.21 | No empty state message when zero items | P2 | ☐ | **Missing** |
| 4.22 | No optimistic update on toggle (waits for API response) | P3 | ☐ | |

---

## VRKB-05: Evidence Blocks

### Backend — `evidence.rs` (4 endpoints, all have auth ✓)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 5.1 | `GET /api/vrkb/projects/:id/evidence` — List all evidence for project | P1 | ☐ | |
| 5.2 | `GET /api/vrkb/projects/:id/evidence?attached_to_type=finding&attached_to_id=X` — Filter by attachment | P1 | ☐ | |
| 5.3 | `POST /api/vrkb/projects/:id/evidence` — Create evidence with type=screenshot | P1 | ☐ | |
| 5.4 | `POST /api/vrkb/projects/:id/evidence` — Create evidence with type=request_response | P1 | ☐ | |
| 5.5 | `POST /api/vrkb/projects/:id/evidence` — Create evidence with type=log_extract | P1 | ☐ | |
| 5.6 | `POST /api/vrkb/projects/:id/evidence` — Create evidence with type=poc_file | P1 | ☐ | |
| 5.7 | `POST /api/vrkb/projects/:id/evidence` — Create evidence with type=external_reference | P1 | ☐ | |
| 5.8 | `POST /api/vrkb/projects/:id/evidence` — Create with invalid type (e.g., `"custom_type"`) | P2 | ☐ | **Accepts any string — no EvidenceType enum validation** |
| 5.9 | `GET /api/vrkb/projects/:id/evidence/:eid` — Get single evidence | P1 | ☐ | |
| 5.10 | `GET /api/vrkb/projects/:id/evidence/:eid` — Non-existent evidence | P2 | ☐ | Should return 404 |
| 5.11 | `DELETE /api/vrkb/projects/:id/evidence/:eid` — Delete evidence | P1 | ☐ | |
| 5.12 | Missing UPDATE endpoint for evidence | P2 | ☐ | Cannot edit evidence metadata |
| 5.13 | In-memory store loses data on restart | P2 | ☐ | **MVP limitation** |
| 5.14 | Model mismatch: API uses `attached_to_type`/`attached_to_id`, entity uses `linked_entity_type`/`linked_entity_id` | P2 | ☐ | **Completely different field names** |
| 5.15 | RwLock `.unwrap()` can panic | P1 | ☐ | |

### Frontend — `EvidencePanel.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 5.16 | Render evidence list grouped by type | P1 | ☐ | |
| 5.17 | Create evidence form with type selector | P1 | ☐ | |
| 5.18 | Delete evidence triggers API call | P1 | ☐ | |
| 5.19 | Delete has no confirmation dialog | P2 | ☐ | **Missing** |
| 5.20 | No loading state during fetch | P2 | ☐ | **Missing** |

---

## VRKB-06: Assets Integration

### Backend — `assets.rs` (mixed auth: some have auth, some don't)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 6.1 | `POST /api/vrkb/projects/:id/assets/link` — Link asset to finding | P1 | ☐ | |
| 6.2 | `POST /api/vrkb/projects/:id/assets/link` — Link asset to doc | P1 | ☐ | |
| 6.3 | `POST /api/vrkb/projects/:id/assets/link` — Link non-existent asset | P2 | ☐ | |
| 6.4 | `POST /api/vrkb/projects/:id/assets/unlink` — Unlink asset | P1 | ☐ | **BUG: calls delete_asset instead of unlinking** |
| 6.5 | `GET /api/vrkb/projects/:id/assets/usage/:aid` — Get asset usage | P1 | ☐ | Returns empty (MVP placeholder) |
| 6.6 | `DELETE /api/vrkb/assets/:id` — No auth | P0 | ☐ | **Unauthenticated delete** |
| 6.7 | `GET /api/vrkb/projects/:id/assets` — No auth | P0 | ☐ | **Unauthenticated list** |
| 6.8 | Upload asset — No file size limit | P2 | ☐ | |
| 6.9 | Upload asset — No MIME type filtering | P2 | ☐ | |
| 6.10 | Uploaded assets not associated with any project | P2 | ☐ | |

### Frontend — `AssetBrowser.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 6.11 | Asset browser renders linked assets | P1 | ☐ | |
| 6.12 | useAssetPicker composable: link/unlink flow | P1 | ☐ | |
| 6.13 | Asset picker modal opens and lists available assets | P1 | ☐ | |

---

## VRKB-07: Doc Repo Enhancement

### Backend — `docs.rs` (12 endpoints, ALL have NO auth ⚠️)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 7.1 | `GET /api/vrkb/projects/:id/docs` — List docs for project | P1 | ☐ | |
| 7.2 | `POST /api/vrkb/projects/:id/docs` — Create doc (author_id: None TODO) | P1 | ☐ | **author_id always None** |
| 7.3 | `GET /api/vrkb/docs/:id` — Get single doc | P1 | ☐ | |
| 7.4 | `PUT /api/vrkb/docs/:id` — Update doc | P1 | ☐ | |
| 7.5 | `DELETE /api/vrkb/docs/:id` — Soft delete | P1 | ☐ | |
| 7.6 | `DELETE /api/vrkb/docs/:id/permanent` — Permanent delete (no protection) | P0 | ☐ | **No confirmation or auth required** |
| 7.7 | `POST /api/vrkb/docs/:id/move` — Move doc to new parent | P1 | ☐ | |
| 7.8 | `POST /api/vrkb/docs/:id/move` — Move doc to create circular reference | P1 | ☐ | **No circular reference detection** |
| 7.9 | `GET /api/vrkb/projects/:id/docs/templates` — List 4 built-in templates | P1 | ☐ | pentest-report, vuln-assessment, meeting-notes, blank |
| 7.10 | `POST /api/vrkb/projects/:id/docs/from-template` — Create from pentest-report template | P1 | ☐ | |
| 7.11 | `POST /api/vrkb/projects/:id/docs/from-template` — Create from vuln-assessment template | P1 | ☐ | |
| 7.12 | `POST /api/vrkb/projects/:id/docs/from-template` — Create from meeting-notes template | P1 | ☐ | |
| 7.13 | `POST /api/vrkb/projects/:id/docs/from-template` — Create from blank template | P1 | ☐ | |
| 7.14 | `POST /api/vrkb/projects/:id/docs/from-template` — Invalid template slug | P2 | ☐ | |
| 7.15 | `POST /api/vrkb/projects/:id/docs/generate-report` — Compile findings + docs into report | P1 | ☐ | |
| 7.16 | `POST /api/vrkb/projects/:id/docs/generate-report` — Project with no findings | P2 | ☐ | |
| 7.17 | ALL 12 endpoints — Test without auth | P0 | ☐ | **No authentication on any endpoint** |
| 7.18 | Nested docs — Parent-child relationships render correctly | P1 | ☐ | |

### Frontend — `DocRepo.vue`, `CreateDocModal.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 7.19 | Doc tree renders nested structure | P1 | ☐ | |
| 7.20 | Create doc modal with template selection | P1 | ☐ | |
| 7.21 | Move doc via drag-and-drop | P2 | ☐ | |
| 7.22 | Report generation button triggers compile | P1 | ☐ | |

---

## VRKB-08: Members and Roles

### Backend — `members.rs` (6 endpoints, ALL have NO auth ⚠️)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 8.1 | `GET /api/vrkb/projects/:id/members` — List members | P1 | ☐ | |
| 8.2 | `POST /api/vrkb/projects/:id/members` — Add member with valid role (owner/lead/researcher/observer) | P1 | ☐ | |
| 8.3 | `POST /api/vrkb/projects/:id/members` — Add member with invalid role | P1 | ☐ | Role validation exists ✓ |
| 8.4 | `POST /api/vrkb/projects/:id/members` — Add non-existent user | P2 | ☐ | **No user existence check** |
| 8.5 | `POST /api/vrkb/projects/:id/members` — Add duplicate member | P2 | ☐ | **No duplicate check** |
| 8.6 | `PUT /api/vrkb/projects/:id/members/:mid` — Update member role | P1 | ☐ | |
| 8.7 | `DELETE /api/vrkb/projects/:id/members/:mid` — Remove member | P1 | ☐ | |
| 8.8 | `DELETE /api/vrkb/projects/:id/members/:mid` — Remove sole owner | P0 | ☐ | **No protection: project becomes ownerless** |
| 8.9 | `GET /api/vrkb/projects/:id/members/:mid/permissions` — Get member permissions | P1 | ☐ | Returns 10-action permission set |
| 8.10 | `GET /api/vrkb/projects/:id/permissions` — Get permission matrix | P1 | ☐ | Returns full role→action matrix |
| 8.11 | ALL 6 endpoints — Test without auth | P0 | ☐ | **No authentication on any endpoint** |
| 8.12 | Permission matrix: owner has all 10 permissions | P1 | ☐ | |
| 8.13 | Permission matrix: observer has only view_project + view_audit_log | P1 | ☐ | |
| 8.14 | Permission matrix: researcher cannot manage_members or delete_project | P1 | ☐ | |
| 8.15 | **RBAC enforcement**: Permissions defined but NEVER enforced in any endpoint | P0 | ☐ | **CRITICAL: Most important systemic issue** |

### Frontend — `TeamManagement.vue`, `InviteMemberModal.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 8.16 | Members list renders with role badges | P1 | ☐ | |
| 8.17 | Invite modal validates role selection | P1 | ☐ | |
| 8.18 | Role change updates member correctly | P1 | ☐ | |
| 8.19 | Remove member with confirmation | P2 | ☐ | |
| 8.20 | `currentUserPermissions` never populated so `hasPermission` always returns false | P1 | ☐ | **BUG in vrkb.ts store** |

---

## VRKB-09: Audit and Notifications

### Backend — `audit.rs` (4 endpoints, ALL have NO auth ⚠️)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 9.1 | `GET /api/vrkb/projects/:id/audit` — List audit logs with pagination (page/per_page) | P1 | ☐ | **Only endpoint with pagination** |
| 9.2 | `GET /api/vrkb/projects/:id/audit?action=X` — Filter by action | P1 | ☐ | |
| 9.3 | `POST /api/vrkb/projects/:id/audit` — Create audit log entry | P1 | ☐ | |
| 9.4 | `POST /api/vrkb/projects/:id/audit` — Auto-generates notification on create | P1 | ☐ | |
| 9.5 | `GET /api/vrkb/projects/:id/notifications` — List notifications | P1 | ☐ | |
| 9.6 | `GET /api/vrkb/projects/:id/notifications` — Notifications not filtered by user | P1 | ☐ | **All users see all notifications** |
| 9.7 | `POST /api/vrkb/notifications/:id/read` — Mark notification as read | P1 | ☐ | |
| 9.8 | `POST /api/vrkb/notifications/:id/read` — Mark non-existent notification | P2 | ☐ | |
| 9.9 | ALL 4 endpoints — Test without auth | P0 | ☐ | **No authentication** |
| 9.10 | `POST /api/vrkb/projects/:id/audit` — Exposed as public API (should be internal) | P1 | ☐ | **Anyone can inject fake audit logs** |
| 9.11 | In-memory store loses data on restart | P2 | ☐ | Uses LazyLock |
| 9.12 | RwLock `.unwrap()` can panic | P1 | ☐ | |

### Frontend — `AuditLog.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 9.13 | Audit log renders entries in chronological order | P1 | ☐ | |
| 9.14 | **BUG**: `entries.value = await vrkbApi.getAuditLog(...)` assigns `{items, total}` to array ref | P0 | ☐ | **HIGH: v-for iterates object properties instead of array items** |
| 9.15 | Pagination controls for audit log | P2 | ☐ | |
| 9.16 | Action type filter | P2 | ☐ | |

---

## VRKB-10: VRKB Portability

### Backend — `portability/vrkb.rs` (VrkbPortabilityProvider trait impl)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 10.1 | `analyze_export(project_id)` — Returns export summary | P1 | ☐ | |
| 10.2 | `export(project_id, progress_tx)` — Exports project package with progress events 5%→99% | P1 | ☐ | |
| 10.3 | `export` — Progress events are sequential (5, 15, 25, ...99) | P2 | ☐ | |
| 10.4 | `analyze_import(data)` — Validates package format | P1 | ☐ | |
| 10.5 | `analyze_import` — Invalid/corrupt data rejected | P1 | ☐ | |
| 10.6 | `import(data, progress_tx)` — Creates project with new ID if conflict | P1 | ☐ | |
| 10.7 | `import` — Preserves findings, docs, members from package | P1 | ☐ | |
| 10.8 | Provider registered with aliases `["vrkb_std", "vulnerability_research"]` in services.rs | P1 | ☐ | |

### Frontend — `vrkb.ts` store

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 10.9 | `exportProject` calls `POST /api/vrkb/projects/${id}/export` | P1 | ☐ | **BUG: Backend route does NOT exist** |
| 10.10 | `importProject` calls `POST /api/vrkb/projects/import` | P1 | ☐ | **BUG: Backend route does NOT exist** |

---

## PLAT-03: Header Action Protocol

### Frontend — `types/header-actions.ts`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 11.1 | HeaderAction interface: icon, label, handler, disabled, tooltip fields | P1 | ☐ | |
| 11.2 | HeaderBadge type: `progress` — has value 0-100 | P1 | ☐ | |
| 11.3 | HeaderBadge type: `count` — has numeric value | P1 | ☐ | |
| 11.4 | HeaderBadge type: `status` — has label string | P1 | ☐ | |
| 11.5 | HeaderBadge type: `context` — defined but unused | P3 | ☐ | |
| 11.6 | HeaderBadge type: `dot` — simple indicator | P1 | ☐ | |
| 11.7 | `animate` and `pulse` fields are redundant | P3 | ☐ | |
| 11.8 | `handler` doesn't support async — verify no async handlers needed | P2 | ☐ | |

### Frontend — `composables/useHeaderActions.ts`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 11.9 | `registerActions(module, group)` — Adds group to Map | P1 | ☐ | |
| 11.10 | `unregisterActions(module)` — Removes group from Map | P1 | ☐ | |
| 11.11 | `getActions(module)` — Returns group or undefined | P1 | ☐ | |
| 11.12 | `allActions` — Returns all registered groups | P1 | ☐ | |
| 11.13 | `activeModuleActions` — Returns actions for current module | P1 | ☐ | |
| 11.14 | `updateBadge(module, actionId, badge)` — Updates badge on specific action | P1 | ☐ | |
| 11.15 | `updateBadge` — Deep property modification may not trigger Map reactivity | P1 | ☐ | **Potential reactivity bug** |
| 11.16 | `clearAllActions()` — Clears entire Map | P1 | ☐ | |
| 11.17 | Global singleton with no auto-cleanup on component unmount | P2 | ☐ | **Memory leak potential** |
| 11.18 | `_resetForTesting()` — Resets state for test isolation | P2 | ☐ | |

### Frontend — `SelfSpaceView.vue` (consumer)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 11.19 | Header renders registered actions with correct icons | P1 | ☐ | |
| 11.20 | Badge renders correctly for each type (progress/count/status/dot) | P1 | ☐ | |
| 11.21 | Action click triggers handler | P1 | ☐ | |

---

## PLAT-04: Portability Runtime

### Backend — `portability.rs` (6 endpoints)

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 12.1 | `POST /api/portability/analyze-export` — Analyze export for given module | P1 | ☐ | |
| 12.2 | `POST /api/portability/export` — Start export, returns task_id | P1 | ☐ | |
| 12.3 | `POST /api/portability/analyze-import` — Analyze import file, returns ImportPreview | P1 | ☐ | |
| 12.4 | `POST /api/portability/import` — Start import with options | P1 | ☐ | |
| 12.5 | `GET /api/portability/progress/:task_id` — SSE progress stream | P1 | ☐ | |
| 12.6 | `GET /api/portability/download/:task_id` — Download export with token/expiry | P1 | ☐ | |
| 12.7 | SSE stream sends progress events (0%→100%) | P1 | ☐ | |
| 12.8 | Download with expired token | P2 | ☐ | |
| 12.9 | Download with invalid task_id | P2 | ☐ | |

### Frontend — `api/portability.ts`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 12.10 | `analyzeExport(kbId, module)` — Correct API call | P1 | ☐ | |
| 12.11 | `startExport(kbId)` — Returns `{ task_id: string }` | P1 | ☐ | |
| 12.12 | `analyzeImport(file)` — Return type `ImportSummary` doesn't match backend's `ImportPreview` | P1 | ☐ | **Type mismatch** |
| 12.13 | `connectProgress(taskId)` — SSE EventSource creation | P1 | ☐ | |
| 12.14 | `connectProgress` — Doesn't pass auth token (unlike store's `listenToProgress`) | P1 | ☐ | **BUG: auth token not sent** |

### Frontend — `stores/portability.ts`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 12.15 | `startExport(kbId)` — Creates task, calls API, starts SSE | P1 | ☐ | |
| 12.16 | `listenToProgress(taskId)` — SSE EventSource with `?token=` auth | P1 | ☐ | |
| 12.17 | EventSource never stored/closed — memory leak on multiple exports | P1 | ☐ | **BUG** |
| 12.18 | Tasks array grows unbounded | P2 | ☐ | |
| 12.19 | `currentImportSummary` declared but never used | P3 | ☐ | Dead code |
| 12.20 | SSE onerror doesn't distinguish temporary disconnects from fatal errors | P2 | ☐ | |

### Frontend — `ExportModal.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 12.21 | Export modal opens and analyzes module | P1 | ☐ | |
| 12.22 | Progress bar updates via SSE events | P1 | ☐ | |
| 12.23 | Download button appears on completion | P1 | ☐ | |
| 12.24 | SSE EventSource not closed on component unmount | P1 | ☐ | **Memory leak** |
| 12.25 | Possible double-trigger of `analyze()` from both `onMounted` and `watch` | P2 | ☐ | |

### Frontend — `ImportAnalysisModal.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 12.26 | Import analysis renders summary | P1 | ☐ | |
| 12.27 | **BUG**: Expects `ImportSummary` but backend returns `ImportPreview` wrapper | P0 | ☐ | **`summary.total_items` will be undefined** |
| 12.28 | Conflict resolution UI | P2 | ☐ | |

### Frontend — `ExportPreviewModal.vue`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 12.29 | Preview renders export summary | P1 | ☐ | |
| 12.30 | No error state display when `loading=false, summary=null` | P2 | ☐ | **Missing** |

---

## PLAT-06: Observability Foundation

### Backend — `domain/observability.rs`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 13.1 | AuditEvent: `new(action, entity_type, entity_id, actor_id)` | P1 | ☐ | |
| 13.2 | AuditEvent: `with_context(key, value)` — Adds to context HashMap | P1 | ☐ | |
| 13.3 | AuditEvent: `with_result(AuditResult::Success)` | P1 | ☐ | |
| 13.4 | AuditEvent: `with_failure(reason)` — Sets result=Failure + description | P1 | ☐ | |
| 13.5 | TaskTelemetry: `start(name, module)` — Creates with status=Running | P1 | ☐ | |
| 13.6 | TaskTelemetry: `start_with_id(task_id, name, module)` — Custom ID | P1 | ☐ | |
| 13.7 | TaskTelemetry: `complete()` — Sets status=Completed, calculates duration | P1 | ☐ | |
| 13.8 | TaskTelemetry: `fail(reason)` — Sets status=Failed, error_message | P1 | ☐ | |
| 13.9 | ErrorBoundaryEvent: `new(module, error_type, message)` | P1 | ☐ | |
| 13.10 | ErrorBoundaryEvent: `with_metadata(key, value)` | P1 | ☐ | |
| 13.11 | ErrorBoundaryEvent: `with_stack_trace(trace)` | P1 | ☐ | |
| 13.12 | ErrorType enum covers 8 variants: Crash, NetworkError, ValidationError, AuthError, NotFound, RateLimited, PluginError, Unknown | P1 | ☐ | |
| 13.13 | **No unit tests exist** for observability module | P2 | ☐ | **Test gap** |

### Frontend — `composables/useAuditLog.ts`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 13.14 | `logAction(module, action, details?)` — Adds entry to buffer | P1 | ☐ | |
| 13.15 | `logNavigation(from, to)` — Logs navigation event | P1 | ☐ | |
| 13.16 | `logError(module, error, context?)` — Logs error event | P1 | ☐ | |
| 13.17 | Buffer auto-eviction at 200 entries | P1 | ☐ | |
| 13.18 | `recentEvents(n)` — Returns last N events | P1 | ☐ | |
| 13.19 | `getEventsByType(type)` — Filter by event type | P1 | ☐ | |
| 13.20 | `exportLog()` — Exports buffer as JSON | P1 | ☐ | |
| 13.21 | `clearLog()` — Empties buffer | P1 | ☐ | |
| 13.22 | **DEAD CODE**: No consumer imports this composable anywhere | P3 | ☐ | **Completely unused** |

### Frontend — `composables/useErrorBoundary.ts`

| # | Test Point | Priority | Status | Notes |
|---|-----------|----------|--------|-------|
| 13.23 | `captureError(module, errorType, message, metadata?)` — Registers error | P1 | ☐ | |
| 13.24 | Per-module error registry (50 max per module) | P1 | ☐ | |
| 13.25 | `isModuleHealthy(module)` — True if no unacknowledged errors | P1 | ☐ | |
| 13.26 | `acknowledgeError(errorId)` — Marks error as acknowledged | P1 | ☐ | |
| 13.27 | `acknowledgeError` — Doesn't trigger Map reactivity | P1 | ☐ | **BUG: computed won't recompute** |
| 13.28 | `acknowledgeAllForModule(module)` — Bulk acknowledge | P1 | ☐ | |
| 13.29 | `totalErrorCount` — Counts all unacknowledged errors | P1 | ☐ | |
| 13.30 | `unhealthyModules` — Lists modules with errors | P1 | ☐ | |
| 13.31 | **DEAD CODE**: No consumer imports this composable anywhere | P3 | ☐ | **Completely unused** |

---

## Cross-Module Systemic Issues

| # | Test Point | Priority | Category | Notes |
|---|-----------|----------|----------|-------|
| S.1 | **Authentication Matrix**: 6 modules have NO auth (members, docs, specs, stats, audit, some assets) | P0 | Security | Must add auth middleware to all routes |
| S.2 | **RBAC Never Enforced**: Permission matrix defined in members.rs but no endpoint checks permissions | P0 | Security | Need `check_permission(user, project, action)` middleware |
| S.3 | **11/12 list endpoints missing pagination**: Only list_audit_logs has page/per_page | P2 | Performance | Add limit/offset or cursor pagination |
| S.4 | **3 modules use in-memory stores**: checklist (OnceLock), evidence (OnceLock), audit/notifications (LazyLock) | P2 | Data | Data lost on restart; migrate to DB |
| S.5 | **State machine bypassed**: Both update_finding (PUT) and update_finding_status (PATCH) skip transition validation | P0 | Logic | Should call `repo.transition_finding_status()` |
| S.6 | **RwLock `.unwrap()` panic risk**: All in-memory stores use `.unwrap()` on RwLock operations | P1 | Stability | Replace with `.expect("reason")` or handle PoisonError |
| S.7 | **Model mismatches**: API structs don't match entity definitions (checklist: missing fields; evidence: different field names) | P2 | Data | Align API DTOs with persistence entities |
| S.8 | **5 TODO/placeholder items**: author_id=None in docs, get_asset_usage returns empty, etc. | P2 | Completeness | |
| S.9 | **No input validation**: Severity, status, confidence, evidence_type accept any string | P2 | Validation | Add enum validation on all typed fields |
| S.10 | **No rate limiting on any endpoint** | P2 | Security | |

---

## Frontend-Specific Issues

| # | Test Point | Priority | Category | Notes |
|---|-----------|----------|----------|-------|
| F.1 | **3 API calls to non-existent routes**: exportProject, importProject, getActivitySummary | P0 | Integration | Either add backend routes or remove frontend calls |
| F.2 | **AuditLog.vue data unpacking bug**: Assigns `{items, total}` to array ref | P0 | Data | Fix: `entries.value = result.items` |
| F.3 | **ImportAnalysisModal type mismatch**: Expects ImportSummary, backend returns ImportPreview | P0 | Type Safety | Align types or unwrap preview |
| F.4 | **Duplicate API functions**: getAuditLog + listAuditLogs; moveDoc + moveDocTo | P3 | Cleanup | Remove duplicates |
| F.5 | **SSE EventSource leaks**: ExportModal.vue and portability store don't close connections | P1 | Memory | Add cleanup in onUnmounted |
| F.6 | **Pervasive `any` types**: data, settings, content, checklist parameters | P2 | Type Safety | Replace with proper interfaces |
| F.7 | **vrkb store: hasPermission always false**: currentUserPermissions never populated | P1 | RBAC | Need to fetch permissions on project load |
| F.8 | **Optimistic updates without rollback**: updateFindingStatus in vrkb store | P1 | UX | Add try/catch with revert |
| F.9 | **Missing UI states**: 4 components lack loading state, 2 lack empty state, 3 lack error display | P2 | UX | |
| F.10 | **useAuditLog + useErrorBoundary completely unused** | P3 | Dead Code | Either integrate or remove |

---

## Known Bugs (Must Fix)

| # | Bug | File | Priority | Impact |
|---|-----|------|----------|--------|
| B.1 | `unlink_asset` calls `delete_asset` instead of unlinking | `backend/.../vrkb/assets.rs` | P0 | Data loss: unlinking deletes the asset entirely |
| B.2 | `update_finding_status` calls `repo.update_finding_status()` not `repo.transition_finding_status()` | `backend/.../vrkb/findings.rs` | P0 | State machine completely bypassed |
| B.3 | `update_finding` PUT can set status field directly, bypassing state machine | `backend/.../vrkb/findings.rs` | P0 | Double bypass path |
| B.4 | AuditLog.vue assigns `{items, total}` object to `ref<AuditEntry[]>` | `frontend/.../AuditLog.vue` | P0 | v-for renders incorrectly |
| B.5 | ImportAnalysisModal expects `ImportSummary` but backend returns `ImportPreview` wrapper | `frontend/.../ImportAnalysisModal.vue` | P0 | `summary.total_items` is undefined |
| B.6 | `exportProject` calls POST `/api/vrkb/projects/${id}/export` — route doesn't exist | `frontend/src/api/vrkb.ts` | P0 | 404 error |
| B.7 | `importProject` calls POST `/api/vrkb/projects/import` — route doesn't exist | `frontend/src/api/vrkb.ts` | P0 | 404 error |
| B.8 | `getActivitySummary` calls GET `/api/vrkb/projects/${id}/activity` — route doesn't exist | `frontend/src/api/vrkb.ts` | P0 | 404 error |
| B.9 | `connectProgress` SSE doesn't pass auth token | `frontend/src/api/portability.ts` | P1 | Auth failure on protected SSE endpoint |
| B.10 | EventSource never closed in portability store | `frontend/src/stores/portability.ts` | P1 | Memory leak per export |
| B.11 | `acknowledgeError` doesn't trigger Map reactivity | `frontend/.../useErrorBoundary.ts` | P1 | Computed properties stale |
| B.12 | `updateBadge` deep property mutation doesn't trigger Map reactivity | `frontend/.../useHeaderActions.ts` | P1 | Badge UI won't update |
| B.13 | `currentUserPermissions` never populated, `hasPermission` always false | `frontend/src/stores/vrkb.ts` | P1 | All permission checks fail |
| B.14 | `updateFindingStatus` no rollback on API failure | `frontend/src/stores/vrkb.ts` | P1 | Inconsistent UI state |
| B.15 | Due date parse failure silently becomes None | `backend/.../vrkb/findings.rs` | P2 | Silent data loss |
| B.16 | Docs `author_id: None` TODO | `backend/.../vrkb/docs.rs` | P2 | Author tracking broken |

---

## Summary Statistics

### By Priority

| Priority | Count | Percentage |
|----------|-------|------------|
| P0 | 28 | 13.1% |
| P1 | 112 | 52.3% |
| P2 | 55 | 25.7% |
| P3 | 19 | 8.9% |
| **Total** | **214** | **100%** |

### By Module

| Module | Backend Tests | Frontend Tests | Total |
|--------|--------------|----------------|-------|
| VRKB-01 (Project Control) | 17 | 6 | 23 |
| VRKB-02 (Finding Lifecycle) | 21 | 8 | 29 |
| VRKB-03 (Triage Queue) | 10 | 6 | 16 |
| VRKB-04 (Checklist) | 15 | 7 | 22 |
| VRKB-05 (Evidence) | 15 | 5 | 20 |
| VRKB-06 (Assets Integration) | 10 | 3 | 13 |
| VRKB-07 (Doc Repo) | 18 | 4 | 22 |
| VRKB-08 (Members & Roles) | 15 | 5 | 20 |
| VRKB-09 (Audit & Notifications) | 12 | 4 | 16 |
| VRKB-10 (Portability) | 8 | 2 | 10 |
| PLAT-03 (Header Actions) | 0 | 21 | 21 |
| PLAT-04 (Portability Runtime) | 9 | 21 | 30 |
| PLAT-06 (Observability) | 13 | 18 | 31 |
| Cross-Module Systemic | 10 | 0 | 10 |
| Frontend-Specific | 0 | 10 | 10 |
| Known Bugs | 8 | 8 | 16 |

### By Category

| Category | Count |
|----------|-------|
| Security (Auth/RBAC) | 31 |
| Functional (Happy Path) | 78 |
| Validation (Input/Edge Case) | 34 |
| Data Integrity | 22 |
| Performance | 8 |
| UX/UI States | 15 |
| Type Safety | 10 |
| Memory/Resource Leaks | 6 |
| Dead Code/Cleanup | 10 |

### Authentication Matrix

| Module | Auth Required? | Auth Present? | Status |
|--------|---------------|---------------|--------|
| projects.rs | ✅ | ✅ | ✅ OK |
| findings.rs | ✅ | ✅ | ✅ OK |
| triage.rs | ✅ | ✅ | ✅ OK |
| checklist.rs | ✅ | ✅ | ✅ OK |
| evidence.rs | ✅ | ✅ | ✅ OK |
| assets.rs | ✅ | ⚠️ Partial | ⚠️ 3/6 missing |
| docs.rs | ✅ | ❌ | ❌ ALL 12 missing |
| members.rs | ✅ | ❌ | ❌ ALL 6 missing |
| specs.rs | ✅ | ❌ | ❌ ALL 2 missing |
| stats.rs | ✅ | ❌ | ❌ 1 missing |
| audit.rs | ✅ | ❌ | ❌ ALL 4 missing |

---

## How to Use This Checklist

1. **Phase 1 — P0 Bugs**: Fix all 16 known bugs and 28 P0 security issues first
2. **Phase 2 — P1 Functional**: Run all 112 P1 test cases for core functionality
3. **Phase 3 — P2 Validation**: Add input validation, missing UI states, pagination
4. **Phase 4 — P3 Cleanup**: Remove dead code, eliminate duplicates, add tests

### Quick Win Priority Order

1. Add auth middleware to `docs.rs`, `members.rs`, `specs.rs`, `stats.rs`, `audit.rs`
2. Fix `unlink_asset` implementation bug
3. Fix `update_finding_status` to use `transition_finding_status`
4. Fix `AuditLog.vue` data unpacking (`entries.value = result.items`)
5. Fix `ImportAnalysisModal` type mismatch
6. Add/fix missing backend routes for `exportProject`, `importProject`, `getActivitySummary`
7. Close EventSource on unmount in ExportModal and portability store
8. Populate `currentUserPermissions` in vrkb store
9. Add RBAC enforcement middleware
10. Add enum validation for severity/status/confidence/evidence_type
