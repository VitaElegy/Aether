# Aether 特殊知识库 — 下一步计划

> 日期: 2026-04-01
> 基于: `special_kb_detailed_execution_plan_2026-03-19.md`
> 当前 HEAD: `52f7750` (master)

## 1. 总体进度

```
Wave 0  Platform Closure              ████████████████████░  95%   ← PLAT-02/03/04/06 有残留
Wave 1  Assets Base Layer              ████████████░░░░░░░░  57%   ← ASSET-05/06/07 待做
Wave 2  English / Vocabulary           ████████████████████  100%  ✅
Wave 3  Memos                          ████████████████████  100%  ✅
Wave 4  PRKB                           ████████████████████  100%  ✅
Wave 5  VRKB                           ░░░░░░░░░░░░░░░░░░░   0%   ← 下一优先
Wave 6  Math                           ████████████████████  100%  ✅
Wave 7  Portability 2.0 Full           ░░░░░░░░░░░░░░░░░░░   0%
Wave 8  Observability / Security       ░░░░░░░░░░░░░░░░░░░   0%
Wave 9  Release & Stability            ░░░░░░░░░░░░░░░░░░░   0%
```

**已完成模块成熟度更新**（基于原始审计 → 实施后估算）:

| 模块 | 审计时 (03-19) | 实施后 (04-01) | 目标 |
|------|---------------|---------------|------|
| Assets | 3.5/10 | 5.5/10 | 8/10 |
| English | 6-7/10 | 8.5/10 | 9/10 |
| Memos | 5.5/10 | 8/10 | 9/10 |
| PRKB | 4.5/10 | 8/10 | 9/10 |
| VRKB | 5.5/10 | 5.5/10 | 9/10 |
| Math | 4.5/10 | 8/10 | 9/10 |

## 2. 下一批次执行计划

### 批次 A — Assets 补完 (ASSET-05~07)

优先级: **P0** — 其他模块（尤其 VRKB）依赖 Picker Mode

| 工作包 | 内容 | 预计规模 |
|--------|------|---------|
| ASSET-05 | Picker Mode — modal/split-view picker, search, recent, upload from picker | ~400 LOC |
| ASSET-06 | Permission Explanation — allowed + reason_code + context_chain | ~300 LOC |
| ASSET-07 | Assets Portability — metadata + binary + usage edges round-trip | ~500 LOC |

**接入点**: Markdown editor, Memos compose, VRKB evidence, PRKB note/PDF

### 批次 B — VRKB 协作平台化 (Wave 5, VRKB-01~10)

优先级: **P0** — 唯一未实施的核心模块

| 工作包 | 内容 | 预计规模 |
|--------|------|---------|
| VRKB-01 | Project Control Center — scope/status/severity/checklist/timeline | ~300 LOC |
| VRKB-02 | Finding Lifecycle — triage→confirmed→exploiting→fixing→verifying→closed | ~500 LOC |
| VRKB-03 | Triage Queue — unreviewed, duplicates, stale, missing evidence | ~300 LOC |
| VRKB-04 | Checklist System — section/methodology completion, blocker items | ~200 LOC |
| VRKB-05 | Evidence Blocks — screenshot, request_response, log_extract, poc_file | ~400 LOC |
| VRKB-06 | Assets Integration — select/upload/reverse lookup/link | ~300 LOC |
| VRKB-07 | Doc Repo — nested docs, templates, report sections, appendix | ~400 LOC |
| VRKB-08 | Members and Roles — owner/lead/researcher/observer, permission matrix | ~300 LOC |
| VRKB-09 | Audit and Notifications — events, webhooks, assignment/due/reopen | ~400 LOC |
| VRKB-10 | VRKB Portability — project package round-trip | ~300 LOC |

**预计总量**: 1,800-2,400 LOC (前后端合计)

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
| Wave 7 | 为 6 个特殊 KB 补齐专项 portability provider + round-trip test |
| Wave 8 | 长任务、权限、导入导出、错误与性能指标纳入可观测 |
| Wave 9 | 文档、发布、回归套件、迁移说明 |

## 3. 建议执行顺序

```
Phase 1 (即时):
├── ASSET-05 Picker Mode         ← 为 VRKB-06 解除依赖
├── ASSET-06 Permission Explain
└── ASSET-07 Assets Portability

Phase 2 (紧接):
└── VRKB-01 ~ VRKB-10           ← 最后一个核心模块
    ├── 可考虑并发: VRKB-01~04 (项目/finding/triage/checklist)
    └── 串行: VRKB-05~10 (依赖 assets integration)

Phase 3 (稳定化):
├── PLAT-02/03/04/06 补完
├── Wave 7 Portability 2.0
├── Wave 8 Observability
└── Wave 9 Release
```

## 4. 已知技术债务

| 类别 | 描述 | 优先级 |
|------|------|--------|
| Build | 前端 chunk > 500kB warning (index-DlqeBf2t.js: 2,500kB) | P2 — code-split special KB modules |
| Test | `portability.test.ts` 1 个 auth header 测试失败 | P1 — axios mock 未传 headers |
| Test | 后端 cargo test 需在非 Windows 控制策略环境验证 | P1 |
| Shell | PLAT-02: minimized/crashed state 未实现 | P2 |
| Shell | `admin_system` 硬编码假设已通过 registry 正当化，但仍需文档化 | P3 |
| English | analyzer 动态/静态混合 import warning | P3 |

## 5. 并发执行策略

上一轮 4 Agent 并发的经验证明 worktree 隔离模式可行:
- 模块间文件重叠极少（仅 `models.rs` 有 1 处 auto-merge）
- 建议下一轮: **ASSET-05~07 串行执行（影响面广）+ VRKB 独立 worktree**
- VRKB-06 (Assets Integration) 需要等 ASSET-05 (Picker Mode) 完成

## 6. 发布完成判定 (来自原始计划)

只有同时满足以下条件，才可以宣称特殊知识库体系进入成熟阶段:

1. ✅ 六个特殊 KB 都拥有正式对象模型 — **5/6 完成，VRKB 待做**
2. ☐ 六个特殊 KB 都拥有专项 portability provider — **4/6 完成**
3. ☐ 六个特殊 KB 都拥有至少 1 条 E2E 主工作流 — **0/6 (无 Playwright)**
4. ☐ 统一 Shell、权限、审计、长任务和错误处理已形成共享底座 — **部分**
5. ✅ `npm run build` + `npm run test:unit` 持续稳定通过
