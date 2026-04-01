# Active Project State

> **Last Updated**: 2026-04-01
> **Current Focus**: Special KB Maturity — Wave 2/3/4/6 Complete, Preparing Wave 5 + Remaining Assets

## Milestone Reached

**4 模块并发开发完成并合并到 master** — 使用独立 git worktree 并行实现 Wave 2 (English), Wave 3 (Memos), Wave 4 (PRKB), Wave 6 (Math)，零冲突合并，前端编译通过。

## Completed Waves (截至 52f7750)

### Wave 0 — Platform Closure ✅ (95%)
- PLAT-01: Renderer Canonicalization — 前后端 normalize 收口完成
- PLAT-02: Shell State Machine — active/running/pinned 完成，minimized/crashed 待补
- PLAT-03: Test Baseline — 116 frontend tests, build stable
- PLAT-05: Test Infrastructure — fixtures, helpers 基本就绪
- 缺口: PLAT-03/04/06 的深度功能（Header Action, Portability Runtime, Observability Foundation）

### Wave 1 — Assets Base Layer ✅ (ASSET-01~04)
- Commit: `11946b3`
- Typed Asset Schema, Upload Pipeline, Asset Console, Usage Graph
- 缺口: ASSET-05 (Picker Mode), ASSET-06 (Permission Explanation), ASSET-07 (Assets Portability)

### Wave 2 — English / Vocabulary ✅ (ENG-01~07)
- 7 commits: `337c0fe` → `0ac3225`
- 12 files, +2,309/-168 lines
- 交付:
  - ENG-01: Identity split — canonical ID 收口, capability map, tab modes
  - ENG-02: Article Workspace — AnalysisStatus 状态机 (Pending→Analyzing→Analyzed|Failed→Archived)
  - ENG-03: Vocabulary Object — lemma, CEFR level, mastery, batch ops
  - ENG-04: Example System 2.0 — multi-example, primary example, search
  - ENG-05: Sentence Anchoring 2.0 — 3-tier repair (exact→normalized→fuzzy→unresolved)
  - ENG-06: Search Intelligence — query pipeline, family words, collocations
  - ENG-07: Portability 2.0 — CSV/JSON/Markdown/Analysis export, Anki import

### Wave 3 — Memos ✅ (MEMO-01~07)
- 1 commit: `a8cb160`
- 15 files, +2,846/-572 lines
- 交付:
  - MEMO-01: Stream Core — card model, quick actions (archive/pin/snooze/convert)
  - MEMO-02: Compose/Editor — slash commands, inline tags, URL paste, channel
  - MEMO-03: Saved Views — saved_view, pinned_tag, channel, review_queue
  - MEMO-04: Bulk Ops — bulk tag/channel/archive/delete, merge, split
  - MEMO-05: Backlinks — mention picker, backlink panel, linked entities
  - MEMO-06: Rhythm/Review — scheduled_at, due_at, snoozed_until, review queues
  - MEMO-07: Portability — JSON/Markdown/Daily Archive export, duplicate import

### Wave 4 — PRKB ✅ (PRKB-01~08)
- 2 commits: `64cf457`, `321e20f`
- 23 files, +3,494/-767 lines
- 交付:
  - PRKB-01: Feed Control — health status, diagnostics, enable/disable, test parser
  - PRKB-02: Inbox Triage — state machine (new→read→saved→skipped→trashed), priority, notes
  - PRKB-03: Library Detail — drawer with tags, signals, PDF status, citation copy
  - PRKB-04: Search/Facet/DSL — `author:`, `venue:`, `year:`, `state:`, `tag:` query syntax
  - PRKB-05: Collections — watchlist, reading_queue, archive, topic_collection
  - PRKB-06: PDF Lifecycle — not_attached→queued→downloaded→indexed→failed
  - PRKB-07: Signals — feed freshness, venue tier, author recurrence, custom importance
  - PRKB-08: Portability — BibTeX export/import, JSON, Markdown digest
  - 新增 15 API endpoints, migration script, 6 new Vue components

### Wave 6 — Math ✅ (MATH-01~06)
- 1 commit: `1c9c5df`
- 16 files, +3,504/-338 lines
- 交付:
  - MATH-01: Formal Object Model — 9 node types, 5 relation types
  - MATH-02: Graph Semantics — dependency graph, cycle detection, unresolved prerequisites
  - MATH-03: Workspace Mode — add/remove node/relation, inspect, mark incomplete proof
  - MATH-04: Manuscript/Archive/Workspace — 3 mode formal switch
  - MATH-05: Formula & References — theorem/definition refs, equation labels, validation
  - MATH-06: Portability — JSON graph, Markdown manuscript, LaTeX package export/import
  - 新增 13 API endpoints, MathService with in-memory graph, 13 unit tests

## Build & Test Status

| Check | Status | Notes |
|-------|--------|-------|
| `npm run build` | ✅ Pass | 28.73s, chunk warning (known, non-blocking) |
| `npx vitest run` | ✅ 103/104 | 1 pre-existing failure in portability auth header test |
| `cargo check` | ⚠️ Blocked | Windows app control policy, not a code issue |
| `cargo test` (math domain) | ✅ Pass | 13/13 math domain tests |

## Cumulative Stats

- **Total commits since baseline**: 16 (including 3 merge commits)
- **Total files changed**: 65
- **Lines added**: +12,153
- **Lines removed**: -1,845
- **Net new code**: ~10,308 lines

## Merge Fix Log (合并时修复的问题)

1. `memos.test.ts` — 补充 `linked_entities: []` 默认值
2. `special_kb_registry.ts` — 补充完整 Capabilities 系统、Introspection API、Validation
3. `LibraryDetailDrawer.vue` — 修复 v-model on prop, 合并重复 defineEmits

## Next Steps

见 `Aether/doc/special_kb_next_steps_2026-04-01.md`
