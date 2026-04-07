# Active Project State

> **Last Updated**: 2026-04-07
> **Current Focus**: Special KB Maturity — Wave 5 VRKB in progress, Memos/PRKB Portability completed

## Milestone Reached (04-07)

**VRKB 后端模型增强 + Memos/PRKB Portability Provider 补齐**：
- Finding 7 状态机 + 状态转换矩阵 + severity/confidence/owner 字段
- Triage Queue 4 分类 + accept/reject/merge/request-evidence API
- 6/6 KB 现在都有专项 portability provider
- 后端 cargo check 0 errors，前端 npm run build 通过

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

### Wave 5 — VRKB ⏳ (25%)
**04-07 新完成:**
- VRKB-02 (后端): Finding Lifecycle 7 状态枚举, severity/confidence/owner/due_date 字段, 状态转换矩阵
- VRKB-03 (后端+API): Triage Queue — 4 分类队列 + 4 操作端点 + 前端 API 方法
- 平台修复: get_asset trait 方法, storage/service.rs 编译修复

**待做:**
- VRKB-01~04 前端 UI 对接
- VRKB-05~10 (Evidence, Assets Integration, Doc Repo, Members, Audit, Portability)

### Wave 6 — Math ✅ (100%)
- MATH-01~06 全部完成

### Wave 7 — Portability 2.0 ⏳ (40%)
**04-07 新完成:**
- `portability/memos.rs` (203行) — MemosPortabilityProvider
- `portability/prkb.rs` (224行) — PrkbPortabilityProvider
- 6/6 KB 都有专项 provider (English, Assets, VRKB, Memos, PRKB + default fallback for Math)

## Build & Test Status

| Check | Status | Notes |
|-------|--------|-------|
| `npm run build` | ✅ Pass | built in 53.86s, chunk warning (known) |
| `cargo check` | ✅ Pass | 0 errors, 56 warnings (pre-existing) |
| `vitest run` | ✅ 117/122 | 5 pre-existing failures (auth header mocks) |

## Cumulative Stats (through 04-07)

- **Total commits since baseline**: 27 (including merge commits)
- **Total files changed**: ~80
- **Net new code**: ~11,100+ lines
- **New files created this session**: 2 (memos.rs, prkb.rs)
- **Files modified this session**: 12

## Commits (04-07)

| Hash | Description |
|------|-------------|
| `faf79d3` | feat(vrkb): VRKB-02/03 Finding lifecycle, triage queue, backend model enhancements |
| `fa1dc99` | feat(portability): add Memos and PRKB dedicated portability providers |
| `1e6c956` | chore: add @types/dompurify and update build artifacts |

## Next Steps

见 `Aether/doc/special_kb_next_steps_2026-04-01.md`
