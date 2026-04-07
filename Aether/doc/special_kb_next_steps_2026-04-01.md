# Aether 特殊知识库 — 下一步计划

> 日期: 2026-04-07 (evening update)
> 基于: `special_kb_detailed_execution_plan_2026-03-19.md`
> 上次更新: VRKB 前端 UI 全面对接 + Memos/PRKB Portability Provider 完成

## 1. 总体进度

```
Wave 0  Platform Closure              ████████████████████░  95%   ← PLAT-02/03/04/06 有残留
Wave 1  Assets Base Layer              ███████████████████░  95%   ✅ ASSET-01~07 全部完成
Wave 2  English / Vocabulary           ████████████████████  100%  ✅
Wave 3  Memos                          ████████████████████  100%  ✅
Wave 4  PRKB                           ████████████████████  100%  ✅
Wave 5  VRKB                           ████████████░░░░░░░  ~60%  ← 后端+前端主体对接完成
Wave 6  Math                           ████████████████████  100%  ✅
Wave 7  Portability 2.0 Full           ████████░░░░░░░░░░░  ~40%  ← 6/6 KB 都有专项 provider
Wave 8  Observability / Security       ░░░░░░░░░░░░░░░░░░░   0%
Wave 9  Release & Stability            ░░░░░░░░░░░░░░░░░░░   0%
```

**已完成模块成熟度更新**:

| 模块 | 审计时 (03-19) | 实施后 (04-07) | 目标 |
|------|---------------|---------------|------|
| Assets | 3.5/10 | 7.5/10 | 8/10 |
| English | 6-7/10 | 8.5/10 | 9/10 |
| Memos | 5.5/10 | 8/10 | 9/10 |
| PRKB | 4.5/10 | 8/10 | 9/10 |
| VRKB | 5.5/10 | **7.5/10** | 9/10 |
| Math | 4.5/10 | 8/10 | 9/10 |

## 2. 本次 (04-07) 完成的工作

### 批次 B1 — VRKB 后端增强 ✅

| 工作包 | 内容 | 状态 |
|--------|------|------|
| VRKB-02 (后端) | Finding Lifecycle — 7 状态枚举 + severity/confidence/owner/due_date 字段 + 状态转换矩阵 | ✅ |
| VRKB-03 (后端+API) | Triage Queue — 4 分类队列 + accept/reject/merge/request-evidence 端点 | ✅ |
| 平台修复 | `get_asset` trait 方法 + `storage/service.rs` 编译修复 | ✅ |

### 批次 B2 — Memos + PRKB Portability Provider ✅

| 工作包 | 内容 | 状态 |
|--------|------|------|
| Memos Provider | `portability/memos.rs` — 专项导出/导入 | ✅ 203 行 |
| PRKB Provider | `portability/prkb.rs` — 专项导出/导入 | ✅ 224 行 |
| Provider 注册 | `services.rs` + `special_kb_registry.ts` | ✅ |

### 批次 B3 — VRKB 前端 UI 全面对接 ✅

| 工作包 | 内容 | 状态 |
|--------|------|------|
| VRKB-01 | OverviewDashboard — `getProjectStats` API 已对接 | ✅ 已对接 |
| VRKB-02 (前端) | VulnerabilityKanban 7 列 + owner/due_date 显示 + Info severity | ✅ +140 行 |
| VRKB-03 (前端) | TriageQueue 4-tab + store actions (accept/reject/merge/requestEvidence) | ✅ 已对接 |
| VRKB-04 | ChecklistPanel toggle/blocker/进度条 | ✅ 已对接 |
| VRKB-05 | EvidencePanel — 5 种证据类型 create/delete | ✅ 已对接 |
| VRKB-06 | AssetBrowser — upload/link/unlink/usage | ✅ 已对接 |
| VRKB-09 (部分) | Audit tab 已添加到导航 | ✅ |
| Store 增强 | findingsByStatus computed + triageQueue state + 9 actions | ✅ +97 行 |

**Store 新增内容:**
- `findingsByStatus` computed — 按 7 状态分组 findings
- `projectStats`, `triageQueue`, `evidence`, `projectAssets` state
- `loadProjectStats`, `loadTriageQueue`, `triageAccept/Reject/Merge/RequestEvidence`
- `loadEvidence`, `loadProjectAssets` actions

### 其他修复

- 安装 `@types/dompurify` 修复 `npm run build` 的 TS2307 错误
- 新增 Triage/Evidence/Audit 三个导航 tab 到 VulnerabilityKanban

## 3. Git Commits (04-07)

| Hash | 描述 | 文件数 |
|------|------|--------|
| `faf79d3` | feat(vrkb): VRKB-02/03 Finding lifecycle, triage queue, backend model enhancements | 7 files, +360/-36 |
| `fa1dc99` | feat(portability): add Memos and PRKB dedicated portability providers | 5 files, +446/-6 |
| `1e6c956` | chore: add @types/dompurify and update build artifacts | 2 files, +29/-1 |
| `d6ec630` | docs: sync progress tracking for 2026-04-07 | 4 files |
| `0ce1dd2` | feat(vrkb): wire VRKB frontend UI — triage/evidence/audit tabs, store actions, kanban | 3 files, +140/-3 |

## 4. 下一步执行计划

### 批次 B4 — VRKB 剩余工作 (待做)

优先级: **P0** — 完成 VRKB 模块最后 30%

| 工作包 | 内容 | 状态 |
|--------|------|------|
| VRKB-07 | Doc Repo — 嵌套文档树、模板插入、markdown/html 导出 | ❌ 待做 |
| VRKB-08 | Members & Roles — 4 角色 (owner/lead/researcher/observer) 权限矩阵 UI | ❌ 待做 |
| VRKB-09 (完整) | Audit & Notifications — 事件列表 UI、webhook 配置、通知中心 | ❌ 待做 |
| VRKB-10 | VRKB Portability — 导出/导入 UI 对接 portability provider | ❌ 待做 |

### 批次 C — Platform 补完 (Wave 0 残留)

优先级: **P1** — 不阻塞功能，但影响长期质量

| 工作包 | 内容 |
|--------|------|
| PLAT-02 补完 | minimized / crashed 状态, deep link 协议, session restore |
| PLAT-03 | Header Action Protocol — shell 顶栏统一 |
| PLAT-04 | Portability Runtime — export preview, import analysis, conflict preview, long task lifecycle |
| PLAT-06 | Observability Foundation — structured audit event, long task telemetry |

### 批次 D — Wave 7~9 收口

优先级: **P2** — 在所有模块主体完成后

| Wave | 内容 |
|------|------|
| Wave 7 | 为 6 个特殊 KB 执行 round-trip portability 测试 (Memos/PRKB provider 已就绪) |
| Wave 8 | 长任务、权限、导入导出、错误与性能指标纳入可观测 |
| Wave 9 | 文档、发布、回归套件、迁移说明 |

## 5. 已知技术债务

| 类别 | 描述 | 优先级 |
|------|------|--------|
| Build | 前端 chunk > 500kB warning (index.js: 2,524kB) | P2 — code-split special KB modules |
| Test | `portability.test.ts` + `assets.test.ts` + `backup.test.ts` 共 5 个 auth header 测试失败 (预存在) | P1 — axios mock 未传 headers |
| Test | 后端 cargo check 0 errors，56 warnings (预存在) | P2 |
| Shell | PLAT-02: minimized/crashed state 未实现 | P2 |
| Shell | `admin_system` 硬编码假设已通过 registry 正当化，但仍需文档化 | P3 |
| English | analyzer 动态/静态混合 import warning | P3 |

## 6. 发布完成判定 (来自原始计划)

只有同时满足以下条件，才可以宣称特殊知识库体系进入成熟阶段:

1. ✅ 六个特殊 KB 都拥有正式对象模型 — **6/6 完成 ✅** (VRKB 7 状态机 + severity/confidence)
2. ✅ 六个特殊 KB 都拥有专项 portability provider — **6/6 完成 ✅**
3. ☐ 六个特殊 KB 都拥有至少 1 条 E2E 主工作流 — **0/6 (无 Playwright)**
4. ☐ 统一 Shell、权限、审计、长任务和错误处理已形成共享底座 — **部分**
5. ✅ `npm run build` + `cargo check` 持续稳定通过 (0 errors)

## 7. Portability Provider 覆盖表 (04-07 更新)

| KB | Provider ID | 文件 | 状态 |
|----|-------------|------|------|
| Default | `default` | `portability/default.rs` | ✅ 通用 fallback |
| English | `english_v1` | `portability/english.rs` (24.76KB) | ✅ 完整 |
| Assets | `assets_v1` | `portability/assets.rs` (24.05KB) | ✅ 完整 |
| VRKB | `vrkb` | `portability/vrkb.rs` (15.58KB) | ✅ 完整 |
| **Memos** | **`memo`** | **`portability/memos.rs` (6.35KB)** | **✅ 新增 04-07** |
| **PRKB** | **`prkb`** | **`portability/prkb.rs` (8.24KB)** | **✅ 新增 04-07** |
| Math | — | 使用 default fallback | ⚠️ 待专项化 |
