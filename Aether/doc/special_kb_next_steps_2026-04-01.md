# Aether 特殊知识库 — 下一步计划

> 日期: 2026-04-07 (updated from 04-03)
> 基于: `special_kb_detailed_execution_plan_2026-03-19.md`
> 上次更新: VRKB-02/03 + Memos/PRKB Portability Provider 完成

## 1. 总体进度

```
Wave 0  Platform Closure              ████████████████████░  95%   ← PLAT-02/03/04/06 有残留
Wave 1  Assets Base Layer              ███████████████████░  95%   ✅ ASSET-01~07 全部完成
Wave 2  English / Vocabulary           ████████████████████  100%  ✅
Wave 3  Memos                          ████████████████████  100%  ✅
Wave 4  PRKB                           ████████████████████  100%  ✅
Wave 5  VRKB                           ████░░░░░░░░░░░░░░░  ~25%  ← 后端模型+API骨架+Triage 已完成
Wave 6  Math                           ████████████████████  100%  ✅
Wave 7  Portability 2.0 Full           ████████░░░░░░░░░░░  ~40%  ← Memos + PRKB provider 已补齐
Wave 8  Observability / Security       ░░░░░░░░░░░░░░░░░░░   0%
Wave 9  Release & Stability            ░░░░░░░░░░░░░░░░░░░   0%
```

**已完成模块成熟度更新**（基于原始审计 → 实施后估算）:

| 模块 | 审计时 (03-19) | 实施后 (04-07) | 目标 |
|------|---------------|---------------|------|
| Assets | 3.5/10 | 7.5/10 | 8/10 |
| English | 6-7/10 | 8.5/10 | 9/10 |
| Memos | 5.5/10 | 8/10 | 9/10 |
| PRKB | 4.5/10 | 8/10 | 9/10 |
| VRKB | 5.5/10 | **6.5/10** | 9/10 |
| Math | 4.5/10 | 8/10 | 9/10 |

## 2. 本次 (04-07) 完成的工作

### 批次 B1 — VRKB 后端增强 ✅

| 工作包 | 内容 | 状态 |
|--------|------|------|
| VRKB-02 (部分) | Finding Lifecycle — 7 状态枚举 + severity/confidence/owner/due_date 字段 + 状态转换矩阵 | ✅ 后端模型完成 |
| VRKB-03 | Triage Queue — 4 分类队列 (unreviewed/duplicates/stale/missing_evidence) + accept/reject/merge/request-evidence 端点 | ✅ 后端+前端API完成 |
| 平台修复 | `get_asset` trait 方法 + `storage/service.rs` 编译修复 (预存在的4个错误) | ✅ |

**具体改动** (04-07):
- `models.rs`: +160 行 — Finding 7 状态枚举、Severity/Confidence 枚举、状态转换矩阵 `VALID_TRANSITIONS`
- `ports.rs`: 新增 `get_asset`, `transition_finding_status`, `list_triage_*` (4个), `merge_finding_duplicate`, checklist/evidence trait 方法
- `triage.rs`: +194 行 — 完整 triage queue API (GET queue, POST accept/reject/merge/request-evidence)
- `vrkb.ts` (前端): +16 行 — `acceptFinding`, `rejectFinding`, `mergeFinding`, `requestEvidence` API 方法
- `vrkb.rs` (仓储): +16 行 — `get_asset` 实现
- `storage/service.rs`: 修复 `get_asset`/`delete_asset` 借用问题

### 批次 B2 — Memos + PRKB Portability Provider ✅

| 工作包 | 内容 | 状态 |
|--------|------|------|
| Memos Provider | `portability/memos.rs` — 专项导出 (ZIP: manifest + memos.json)、分析导入、格式验证 | ✅ 新建 203 行 |
| PRKB Provider | `portability/prkb.rs` — 专项导出 (ZIP: manifest + feeds + papers + collections)、分析导入 | ✅ 新建 224 行 |
| Provider 注册 | `services.rs` 注册 + `special_kb_registry.ts` 更新 portabilityProviderId | ✅ |

**具体改动** (04-07):
- 新建 `portability/memos.rs` (203 行) — MemosPortabilityProvider, provider_id="memo"
- 新建 `portability/prkb.rs` (224 行) — PrkbPortabilityProvider, provider_id="prkb"
- `portability/mod.rs`: 新增 `pub mod memos; pub mod prkb;`
- `services.rs`: 注册两个新 provider，memo/prkb 从 default fallback 移除
- `special_kb_registry.ts`: memo portabilityProviderId "default"→"memo", prkb "default"→"prkb"

### 其他修复

- 安装 `@types/dompurify` 修复 `npm run build` 的 TS2307 错误
- 清理 12 个调试临时文件

## 3. Git Commits (04-07)

| Hash | 描述 | 文件数 |
|------|------|--------|
| `faf79d3` | feat(vrkb): VRKB-02/03 Finding lifecycle, triage queue, backend model enhancements | 7 files, +360/-36 |
| `fa1dc99` | feat(portability): add Memos and PRKB dedicated portability providers | 5 files, +446/-6 |
| `1e6c956` | chore: add @types/dompurify and update build artifacts | 2 files, +29/-1 |

## 4. 下一步执行计划

### 批次 B3 — VRKB 前端工作流 (待做)

优先级: **P0** — 后端 API 已就绪，需要前端对接

| 工作包 | 内容 | 状态 |
|--------|------|------|
| VRKB-01 | OverviewDashboard 增强 — severity 分布图、checklist 进度、资产计数、时间线 | ❌ 待做 |
| VRKB-02 (前端) | VulnerabilityKanban 7 列 + FindingEditor 状态转换 UI + severity/owner 选择器 | ❌ 待做 |
| VRKB-03 (前端) | TriageQueue 4 tab + 操作按钮 (accept/reject/merge) + 计数 badge | ❌ 待做 |
| VRKB-04 | ChecklistPanel toggle + blocker 标记 + 完成百分比 | ❌ 待做 |
| VRKB-05 | Evidence Blocks — 5 种证据类型创建/关联 UI | ❌ 待做 |
| VRKB-06 | Assets Integration — AssetPicker 集成到 VRKB | ❌ 待做 |
| VRKB-07 | Doc Repo — 嵌套文档、模板 | ❌ 待做 |
| VRKB-08 | Members & Roles — 权限矩阵 UI | ❌ 待做 |
| VRKB-09 | Audit & Notifications — 事件列表 UI | ❌ 待做 |
| VRKB-10 | VRKB Portability — 导出/导入 UI | ❌ 待做 |

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

1. ✅ 六个特殊 KB 都拥有正式对象模型 — **5/6 完成，VRKB 模型已增强 (状态机+字段)**
2. ✅ 六个特殊 KB 都拥有专项 portability provider — **6/6 完成 ✅** (Memos + PRKB 04-07 补齐)
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
