# Aether 特殊知识库 — 下一步计划

> 日期: 2026-04-03 (updated from 04-01)
> 基于: `special_kb_detailed_execution_plan_2026-03-19.md`
> 上次更新: ASSET-05/06/07 全部完成 + ImportModal 完整实现

## 1. 总体进度

```
Wave 0  Platform Closure              ████████████████████░  95%   ← PLAT-02/03/04/06 有残留
Wave 1  Assets Base Layer              ███████████████████░  95%   ✅ ASSET-05/06/07 已完成
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

| 模块 | 审计时 (03-19) | 实施后 (04-03) | 目标 |
|------|---------------|---------------|------|
| Assets | 3.5/10 | 7.5/10 | 8/10 |
| English | 6-7/10 | 8.5/10 | 9/10 |
| Memos | 5.5/10 | 8/10 | 9/10 |
| PRKB | 4.5/10 | 8/10 | 9/10 |
| VRKB | 5.5/10 | 5.5/10 | 9/10 |
| Math | 4.5/10 | 8/10 | 9/10 |

## 2. 下一批次执行计划

### 批次 A — Assets 补完 (ASSET-05~07) ✅ 已完成

优先级: **P0** — 其他模块（尤其 VRKB）依赖 Picker Mode

| 工作包 | 内容 | 状态 |
|--------|------|------|
| ASSET-05 | Picker Mode — editor 斜杠命令 + ComposeBar 附件按钮 + `/asset` 指令 | ✅ 已完成 |
| ASSET-06 | Permission Explanation — AssetPermissionBadge 集成到 MyAssets 详情侧栏 | ✅ 已完成 |
| ASSET-07 | Assets Portability — Export/Import 按钮 + usage_edges 跨 KB 引用搜索 + ImportModal 完整实现 | ✅ 已完成 |

**已完成的具体改动** (04-03):
- `MarkdownEditorAdapter.vue`: 新增 "Asset" 和 "Image Asset" 斜杠命令，通过 `useAssetPicker` 插入 `![name]([[asset:UUID]])`
- `ComposeBar.vue`: 新增 `attachAsset()` + `/asset` 斜杠命令 + Attach 按钮绑定
- `MyAssets.vue`: 集成 `AssetPermissionBadge` + Export/Import 按钮 + `kb_id` 传递
- `portability/assets.rs`: `usage_edges` 从空 stub 升级为全文搜索 `[[asset:UUID]]` 引用
- `api/assets.rs`: `AssetListResponse` 新增 `kb_id` 字段
- `ImportModal.vue`: 从占位符升级为完整 6 步流程（选择文件→分析→预览冲突→导入→完成/错误）

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
Phase 1 (已完成 ✅):
├── ASSET-05 Picker Mode         ✅ editor + ComposeBar 集成
├── ASSET-06 Permission Explain  ✅ MyAssets 侧栏集成
└── ASSET-07 Assets Portability  ✅ Export/Import + usage_edges + ImportModal

Phase 2 (当前 ← ):
└── VRKB-01 ~ VRKB-10           ← 最后一个核心模块, ASSET-05 依赖已解除
    ├── 可考虑并发: VRKB-01~04 (项目/finding/triage/checklist)
    └── 串行: VRKB-05~10 (依赖 assets integration — 已就绪)

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
- 建议下一轮: **VRKB-01~04 可并发, VRKB-05~10 串行**
- VRKB-06 (Assets Integration) 依赖的 ASSET-05 (Picker Mode) 已完成 ✅

## 6. 发布完成判定 (来自原始计划)

只有同时满足以下条件，才可以宣称特殊知识库体系进入成熟阶段:

1. ✅ 六个特殊 KB 都拥有正式对象模型 — **5/6 完成，VRKB 待做**
2. ☐ 六个特殊 KB 都拥有专项 portability provider — **5/6 完成 (Assets 已完成 ✅)**
3. ☐ 六个特殊 KB 都拥有至少 1 条 E2E 主工作流 — **0/6 (无 Playwright)**
4. ☐ 统一 Shell、权限、审计、长任务和错误处理已形成共享底座 — **部分**
5. ✅ `npm run build` + `npm run test:unit` 持续稳定通过
