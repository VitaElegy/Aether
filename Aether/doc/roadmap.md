# Aether Roadmap & Status

## 🟢 Phase 1 (Foundation) — COMPLETED

### Core Architecture

- [x] **Project Structure**: Hexagonal Architecture (Domain/Infra/Interface) established.
- [x] **Database**: Postgres integration via SeaORM with auto-migration.
- [x] **Dependency Injection**: `AppState` utilizing `Arc<dyn Trait>` for loose coupling.

### Authentication & Security

- [x] **Secure Storage**: Argon2id password hashing implementation.
- [x] **Token System**: JWT generation and verification service.
- [x] **API**: `/api/auth/login` and `/api/auth/register` endpoints.
- [x] **Frontend Guard**: Vue Router navigation guards for protected routes.

### User Interface (The "Void" Design)

- [x] **Visual Identity**: "Future Minimalist" theme (Dark mode, Glassmorphism).
- [x] **Motion**: Fluid entrance animations using `v-motion`.
- [x] **Login Experience**: Split-screen design with generative abstract art.
- [x] **Dashboard**: Information-dense layout with sidebar navigation and stats.

### Observability

- [x] **Structured Logging**: JSON logging to file + Pretty printing to stdout (`tracing`).
- [x] **Trace Middleware**: Automatic HTTP request tracking.

### Extensibility

- [x] **WASM Host**: Basic `WasmPluginHost` structure integrated into infrastructure.
- [x] **Dynamic Rendering**: Frontend `DynamicRenderer` component for polymorphic content.

---

## 🟡 Special KB Maturity (Phase 1.5) — IN PROGRESS (80%)

> Detailed plan: `special_kb_detailed_execution_plan_2026-03-19.md`
> Current status: `special_kb_next_steps_2026-04-01.md`
> Last updated: 2026-04-07

### Platform Closure (Wave 0) — 95%

- [x] **PLAT-01**: Renderer Canonicalization — frontend + backend normalize, alias map, tests
- [x] **PLAT-02**: Shell State Machine — active/running/pinned (minimized/crashed deferred)
- [x] **PLAT-05**: Test Infrastructure — vitest suite, fixtures, 116 tests
- [ ] **PLAT-03**: Header Action Protocol
- [ ] **PLAT-04**: Portability Runtime (unified export/import)
- [ ] **PLAT-06**: Observability Foundation (audit events, telemetry)

### Assets (Wave 1) — 57%

- [x] **ASSET-01**: Typed Asset Schema (7 types)
- [x] **ASSET-02**: Upload Pipeline (progress, dedupe, URL import)
- [x] **ASSET-03**: Asset Console (grid/table, filters, sort)
- [x] **ASSET-04**: Usage Graph (edge tracking, reverse lookup)
- [ ] **ASSET-05**: Picker Mode (cross-module asset selection)
- [ ] **ASSET-06**: Permission Explanation
- [ ] **ASSET-07**: Assets Portability

### English / Vocabulary (Wave 2) — 100%

- [x] **ENG-01**: Identity & Capability Split
- [x] **ENG-02**: Article Workspace (state machine)
- [x] **ENG-03**: Vocabulary Object Upgrade (lemma, CEFR, mastery, batch ops)
- [x] **ENG-04**: Example System 2.0 (multi-example, primary, search)
- [x] **ENG-05**: Sentence Anchoring 2.0 (3-tier repair)
- [x] **ENG-06**: Search & Intelligence (query pipeline, family, collocations)
- [x] **ENG-07**: English Portability 2.0 (CSV/JSON/Markdown/Anki)

### Memos (Wave 3) — 100%

- [x] **MEMO-01**: Stream Core (card model, quick actions)
- [x] **MEMO-02**: Compose / Editor (slash commands, channels)
- [x] **MEMO-03**: Saved Views & Dock
- [x] **MEMO-04**: Organization & Bulk Ops (merge, split)
- [x] **MEMO-05**: Backlinks & References
- [x] **MEMO-06**: Rhythm & Review (queues, scheduling)
- [x] **MEMO-07**: Memos Portability

### PRKB (Wave 4) — 100%

- [x] **PRKB-01**: Feed Control Center (health, diagnostics)
- [x] **PRKB-02**: Inbox Triage (state machine, priority)
- [x] **PRKB-03**: Library Detail Drawer (tags, signals, citation)
- [x] **PRKB-04**: Search / Facet / Query DSL
- [x] **PRKB-05**: Collections & Queues (watchlist, reading queue)
- [x] **PRKB-06**: PDF Lifecycle (5-state)
- [x] **PRKB-07**: Signals (freshness, tier, recurrence)
- [x] **PRKB-08**: PRKB Portability (BibTeX, JSON, Markdown)

### VRKB (Wave 5) — 25% ← IN PROGRESS

- [x] **VRKB-02 (Backend)**: Finding Lifecycle 7-state model + severity/confidence/owner fields
- [x] **VRKB-03 (Backend+API)**: Triage Queue — 4-category queue + accept/reject/merge/request-evidence
- [ ] **VRKB-01**: Project Control Center
- [ ] **VRKB-02 (Frontend)**: Finding Lifecycle UI (Kanban 7-col, editor state transitions)
- [ ] **VRKB-03 (Frontend)**: Triage Queue UI (4-tab + action buttons)
- [ ] **VRKB-04**: Checklist System
- [ ] **VRKB-05**: Evidence Blocks
- [ ] **VRKB-06**: Assets Integration
- [ ] **VRKB-07**: Doc Repo
- [ ] **VRKB-08**: Members & Roles
- [ ] **VRKB-09**: Audit & Notifications
- [ ] **VRKB-10**: VRKB Portability

### Math (Wave 6) — 100%

- [x] **MATH-01**: Formal Object Model (9 node types, 5 relation types)
- [x] **MATH-02**: Graph Semantics (cycle detection, prerequisites)
- [x] **MATH-03**: Workspace Mode (graph editing, blockers)
- [x] **MATH-04**: Manuscript / Archive / Workspace modes
- [x] **MATH-05**: Formula & References (validation, labels)
- [x] **MATH-06**: Math Portability (JSON/Markdown/LaTeX)

### Closure (Wave 7-9) — 10%

- [x] **Wave 7 (partial)**: Memos + PRKB portability providers added (6/6 KBs now have dedicated providers)
- [ ] **Wave 7**: Round-trip tests for all 6 KBs
- [ ] **Wave 8**: Observability / Security / Performance
- [ ] **Wave 9**: Release & Stability (docs, migration, regression suite)

---

## 🟡 Phase 2 (Content & Interaction) — NEXT STEPS

### Content Management

- [ ] **CRUD API**: Implement `GET/POST/PUT/DELETE` for `/api/content`.
- [ ] **Rich Editor**: Integrate a Markdown/WYSIWYG editor in the frontend.
- [ ] **Tag System**: Implement the logic for tag aggregation and searching.

### Advanced Plugin System

- [ ] **WIT Interface**: Define a standard WIT (Wasm Interface Type) for plugins.
- [ ] **Hot Reloading**: Allow plugins to be uploaded/reloaded without restarting the backend.
- [ ] **Marketplace**: A simple UI to list and enable installed plugins.

### Social Features

- [ ] **Comments**: A threaded comment system (stored as JSONB or separate table).
- [ ] **Reactions**: Simple emoji reactions for posts.

---

## 🔴 Phase 3 (The Singularity) — LONG TERM

- [ ] **Edge Replication**: Support for SQLite replication to edge nodes (Turso/LiteFS).
- [ ] **AI Integration**: Local LLM inference via `rust-bert` for content summarization.
- [ ] **3D Space**: Replace the 2D dashboard with a WebGL-based navigation system.
