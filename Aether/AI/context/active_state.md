# Active Project State

> **Last Updated**: 2026-04-07 (evening)
> **Current Focus**: Special KB Maturity — VRKB frontend UI wired, all 6 portability providers complete

## Milestone Reached (04-07 Evening)

**VRKB 前端 UI 全面对接后端 API**：
- VulnerabilityKanban: 7 列看板 + owner/due_date 显示 + Info severity
- TriageQueue: 4-tab 分类 + accept/reject/merge/requestEvidence store actions
- EvidencePanel: 5 种证据类型 (已对接 API)
- AssetBrowser: upload/link/unlink/usage (已对接 API)
- ChecklistPanel: toggle/blocker/进度条 (已对接 API)
- OverviewDashboard: getProjectStats API 已对接
- 新增 Triage/Evidence/Audit 三个导航 tab
- Store 新增: findingsByStatus computed、triageQueue state、9 个新 actions

## Completed Waves

### Wave 0 — Platform Closure ✅ (95%)
- PLAT-01~05 完成，PLAT-03/04/06 待补

### Wave 1 — Assets Base Layer ✅ (95%)
- ASSET-01~07 全部完成

### Wave 2 — English / Vocabulary ✅ (100%)
- ENG-01~07 全部完成

### Wave 3 — Memos ✅ (100%)
- MEMO-01~07 全部完成

### Wave 4 — PRKB ✅ (100%)
- PRKB-01~08 全部完成

### Wave 5 — VRKB ⏳ (60%)
**04-07 完成:**
- VRKB-02 (后端+前端): Finding Lifecycle 7 状态 + Kanban 7 列 + owner/due_date + Info severity
- VRKB-03 (后端+前端): Triage Queue — 后端 4 分类 + 前端 4-tab + store actions
- VRKB-01: OverviewDashboard — stats API 已对接
- VRKB-04: ChecklistPanel — 已完整对接 (toggle/blocker/进度条)
- VRKB-05: EvidencePanel — 5 种证据类型已对接
- VRKB-06: AssetBrowser — upload/link/unlink/usage 已对接
- VRKB-09 (部分): Audit tab 已添加到导航
- 平台修复: get_asset trait + storage/service.rs 编译修复

**待做:**
- VRKB-07: Doc Repo (嵌套文档、模板)
- VRKB-08: Members & Roles (4 角色权限矩阵)
- VRKB-09 (完整): Audit 事件列表、webhook 配置
- VRKB-10: VRKB Portability 导出/导入 UI

### Wave 6 — Math ✅ (100%)
- MATH-01~06 全部完成

### Wave 7 — Portability 2.0 ⏳ (40%)
- 6/6 KB 都有专项 provider (English, Assets, VRKB, Memos, PRKB + default for Math)

## Build & Test Status

| Check | Status | Notes |
|-------|--------|-------|
| `npm run build` | ✅ Pass | built in 42.27s, chunk warning (known) |
| `cargo check` | ✅ Pass | 0 errors, 56 warnings (pre-existing) |
| `vitest run` | ✅ 117/122 | 5 pre-existing failures (auth header mocks) |

## Commits (04-07 Full Day)

| Hash | Description |
|------|-------------|
| `faf79d3` | feat(vrkb): VRKB-02/03 Finding lifecycle, triage queue, backend model enhancements |
| `fa1dc99` | feat(portability): add Memos and PRKB dedicated portability providers |
| `1e6c956` | chore: add @types/dompurify and update build artifacts |
| `d6ec630` | docs: sync progress tracking for 2026-04-07 |
| `0ce1dd2` | feat(vrkb): wire VRKB frontend UI — triage/evidence/audit tabs, store actions, kanban owner/due_date |

## Cumulative Stats (through 04-07)

- **Total commits since baseline**: 29
- **Total files changed**: ~85
- **Net new code**: ~12,000+ lines
- **New files created**: 4 (memos.rs, prkb.rs portability providers + triage/models backend)
- **VRKB frontend files modified**: 3 (vrkb.ts store, VulnerabilityKanban.vue, VRFindingEditor.vue)

## Next Steps

见 `Aether/doc/special_kb_next_steps_2026-04-01.md`
