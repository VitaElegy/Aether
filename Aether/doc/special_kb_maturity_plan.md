# Aether 特殊知识库成熟化总规划

状态: 提案
日期: 2026-03-19
作者: Codex

补充说明:

- 本文档是总规划和阶段规划。
- 功能颗粒度版 backlog 见 `Aether/doc/special_kb_feature_backlog.md`。
- 主实施计划见 `Aether/doc/special_kb_detailed_execution_plan_2026-03-19.md`。

## 1. 目标定义

本规划的目标不是“继续往现有模块上堆功能”，而是把 Aether 目前所有特殊知识库统一提升到可长期演进、可测试、可迁移、可导入导出、具备明确边界和稳定交互协议的成熟系统。

本轮成熟化工作的硬目标:

1. 特殊知识库形成统一协议，彻底消除大量别名、临时 fallback、模糊映射。
2. 每个特殊知识库都具备完整的最小闭环:
   - 创建/发现
   - 列表/检索
   - 编辑/操作
   - 专项视图
   - 导入/导出
   - 权限/引用
   - 测试
3. 前后端都可以稳定构建，后端测试必须恢复为绿色。
4. 新增代码规模不少于 10,000 行，但必须是结构化增量，不接受“为了凑行数”的无效堆砌。

## 2. 已审阅材料

本规划基于以下真实代码和规范，而不是抽象设想:

- 总体架构:
  - `Aether/ARCHITECTURE.md`
  - `Aether/AI/context/specs/self_space_architecture_v2.md`
  - `Aether/AI/context/specs/kb_architecture_v3.md`
  - `Aether/AI/context/specs/unified_data_model.md`
- 专项规格:
  - `Aether/AI/context/specs/asset_kb_spec_v1.md`
  - `Aether/AI/context/specs/vr_kb_spec.md`
  - `Aether/AI/context/specs/prkb_v2_spec.md`
  - `Aether/AI/context/specs/memos_v2_spec_draft.md`
  - `Aether/AI/context/specs/data_portability_spec_v1.md`
  - `Aether/AI/context/specs/kb_backup_protocol_v1.md`
  - `Aether/AI/context/specs/paper_v1_spec.md`
  - `Aether/AI/memory/discussions/2026-01-18-English-KB-Design.md`
- 关键实现位置:
  - `Aether/frontend/src/main.ts`
  - `Aether/frontend/src/composables/useSelfSpaceOrchestrator.ts`
  - `Aether/frontend/src/views/SelfSpaceView.vue`
  - `Aether/frontend/src/registries/read_layout_registry.ts`
  - `Aether/backend/src/infrastructure/bootstrap/services.rs`
  - `Aether/backend/src/interface/api/*`
  - `Aether/backend/src/infrastructure/persistence/repositories/*`

## 3. 当前现状结论

### 3.1 总体判断

Aether 当前已经不是“空壳 demo”，而是一个中等复杂度的知识操作系统雏形。它已经具备:

- 多知识库 renderer/plugin 体系
- Self Space 壳层
- 通用内容模型
- 多个特殊 KB 原型
- 备份和可移植性框架
- 一定规模的仓储层和 API 层

但距离“成熟系统”还存在四类关键缺口:

1. 协议层不统一
2. 模块完成度严重不均
3. 可移植性和专项能力多数仍停留在 generic fallback
4. 测试体系不足，且现有测试已出现回归失败

### 3.2 与设计理念最冲突的问题

当前代码与规范之间最大的矛盾，不在 UI，而在身份协议。

规范要求:

- `renderer_id` 严格对齐
- 禁止 fuzzy alias
- 路由和插件注册必须单一真源

当前实现实际存在:

- `main.ts` 中大量 `Object.assign({}, Plugin, { id: 'alias' })`
- `useSelfSpaceOrchestrator.ts` 中 legacy renderer 解析
- `read_app_state.ts` 中 alias 思路
- `services.rs` 中 portability provider 别名兜底

这会直接带来:

- KB 身份不稳定
- 导入导出协议难收敛
- 插件边界不清
- 测试很难精确定义“这个 KB 到底是哪一种”

结论: 第一优先级不是继续加单个模块 UI，而是先做“特殊 KB 统一协议层重构”。

## 4. 模块成熟度盘点

### 4.1 基础壳层 / 协议层

成熟度: 中

已有:

- Self Space 壳层
- Orchestrator
- Dock / Running / Pinned 的基本机制
- Plugin Store
- Layout Registry

缺口:

- 仍有大量 alias 与 legacy 兼容路径
- `renderer_id -> plugin.id -> provider.id -> layout.id` 没有统一注册表
- 壳层和具体 app 的能力声明还不够强
- 缺少系统级集成测试

### 4.2 English / Vocabulary

成熟度: 中高

已有:

- VocabularyModule 规模较大
- English sentence anchoring 设计已存在
- 字典查询与上下文绑定思路明确
- 有较完整的 UI 交互

缺口:

- 仍通过 alias 复用 VocabularyPlugin / ArticleAnalysisPlugin
- English 作为独立 KB 的协议边界不清
- 测试回归已破坏 `VocabularyExample` 初始化
- 缺少完整导出导入验收

### 4.3 Memos

成熟度: 中

已有:

- Stream / Masonry / Kanban / Timeline / Calendar 组件群
- 用户设置和 workflow 基础
- Markdown 编辑器接入

缺口:

- 仍缺少项目上下文、引用网络、批量操作 API 完善
- 组件存在但缺少系统级稳定性验证
- 移动交互与拖拽策略仍偏原型

### 4.4 PRKB

成熟度: 中低

已有:

- Feed / Inbox / Library
- ArXiv / RSS ingestion
- authors / venues / signals 方向已进入仓储层

缺口:

- 前端主视图过薄
- Facet / Search Syntax / State Machine 还不完整
- Deep crawl / signal refresh / PDF lifecycle 未完成
- 规范目标远高于现实现状

### 4.5 VRKB

成熟度: 中

已有:

- Project / Findings / Sections / Docs / Members / Specs / Assets API
- 相对完整的模块目录
- 项目与看板基础存在

缺口:

- 审计轨迹、TODO 系统、Webhook、Repo intelligence 尚未闭环
- 成员权限与协作粒度还不够强
- 文档库、资产库、发现流之间缺少统一工作流

### 4.6 Assets KB

成熟度: 低

已有:

- `AssetManager`
- 上传与取回接口
- 自动创建 “My Assets” KB
- 基础前端页面

缺口:

- 视图仍接近简单文件墙
- 没有结构化资产类型管理
- 没有 usage graph / picker mode / contextual access UI
- 没有资产引用审计与冲突处理

### 4.7 Math

成熟度: 中低

已有:

- 数学专用 dashboard
- Manuscript / Archive 两种阅读布局
- 图结构和函数渲染方向

缺口:

- Dashboard 仍偏展示层
- Graph 与 theorem / axiom 的编辑闭环不完整
- 数学 KB 的专项导入导出尚未实现
- 缺少正式的“数学对象模型 + 引理依赖 + 演算工作台”

## 5. 基线验证结果

### 5.1 当前可通过项

- 前端构建通过:
  - `npm run build` in `Aether/frontend`

### 5.2 当前失败项

- 后端测试失败:
  - `cargo test` in `Aether/backend`
  - 失败原因: `VocabularyExample` 新增 `global_sentence_id` 字段后，`vocab_test.rs` 中构造体初始化未同步

结论:

- 当前项目“可以构建”，但还不能称作“完整可测试”
- 第一阶段必须先修复测试基线与最小 CI 入口

## 6. 总体实施策略

本项目不建议一次性无序大改。应该拆成 8 个阶段，每个阶段可提交、可验证、可回滚。

### 阶段 0: 稳定基线与统一协议

目标:

- 修复所有现存测试回归
- 清理特殊 KB 身份协议
- 建立单一注册表

必须完成:

1. 新建统一 KB registry:
   - `renderer_id`
   - `plugin.id`
   - `layout.id`
   - `dashboard.id`
   - `portability.provider_id`
   - `capabilities`
2. 移除 `main.ts` 中大部分 alias 注册
3. 移除 orchestrator 中 legacy fuzzy resolution
4. 引入正式的 capability schema
5. 修复 `cargo test`
6. 增加前后端 smoke test

新增代码预算:

- 1,200 至 1,800 行

### 阶段 1: Portability 2.0

目标:

让特殊 KB 真正拥有“专项导入导出”，而不是统统回退到 DefaultProvider。

必须完成:

1. Provider 抽象增强:
   - preview
   - validate
   - import policies
   - conflict resolution
   - progress channel
2. 针对下列 KB 实现专用 provider:
   - English/Vocabulary
   - Memos
   - PRKB
   - VRKB
   - Math
   - Assets
3. 统一导出 artifact 规范:
   - manifest
   - meta
   - content
   - assets
   - diagnostics
4. 前端 Import/Export 对话框统一升级
5. 导入预览与冲突策略可视化

新增代码预算:

- 1,500 至 2,200 行

### 阶段 2: Assets KB 正式化

目标:

把 Assets 从“上传页”提升为系统级资源中心。

必须完成:

1. 结构化资产 schema:
   - image_asset
   - file_asset
   - ip_asset
   - domain_asset
   - credential_stub
2. 资产详情页:
   - metadata
   - references
   - preview
   - download / copy embed
3. picker mode
4. contextual access UI
5. 使用位置反查
6. 与 VRKB / 编辑器 / 普通文章联动

新增代码预算:

- 1,200 至 1,800 行

### 阶段 3: PRKB V2 全面落地

目标:

让 PRKB 从“收 feed 的 inbox”变成真正的研究工作台。

必须完成:

1. Workflow state machine:
   - Inbox
   - Screening
   - Reading
   - Archived
2. Faceted Search
3. Search Syntax Parser
4. Paper detail drawer / reader panel
5. PDF local lifecycle
6. signals refresh jobs
7. deep crawl / historical backfill
8. export:
   - markdown digest
   - bibtex
   - json package

新增代码预算:

- 1,800 至 2,500 行

### 阶段 4: VRKB 升级到审计协作平台

目标:

让 VRKB 成为真正的安全研究元层，而不是单一 project board。

必须完成:

1. Finding lifecycle 完整化
2. Section checklist + evidence trace
3. TODO 系统
4. Team permission matrix
5. repo intelligence panel
6. webhook / notification interface
7. audit timeline
8. docs / findings / assets 联动

新增代码预算:

- 1,800 至 2,400 行

### 阶段 5: Memos V2 收敛为高频捕捉引擎

目标:

把当前已有组件群收敛成稳定的一致体验。

必须完成:

1. Stream 为主视图正式化
2. ComposeBar + Modal Editor 双模闭环
3. pinned channels / saved searches
4. 批量操作 API
5. memo -> article / kb 引用转换
6. 日历 / 时间线 / Kanban 一致的状态模型

新增代码预算:

- 1,000 至 1,500 行

### 阶段 6: English / Vocabulary 专项收口

目标:

把 English 从“通过别名拼出来的能力组合”提升为协议上独立、交互上稳定的语言 KB。

必须完成:

1. English KB 专属 plugin 与 capability
2. article analysis / sentence selection / vocab save 统一流程
3. example / sentence / global sentence 关系修复
4. English portability provider
5. regression test 补齐

新增代码预算:

- 900 至 1,400 行

### 阶段 7: Math KB 完整化

目标:

让 Math KB 从“好看的阅读与树图”升级为可操作的数学知识系统。

必须完成:

1. theorem / axiom / definition / proof 的正式对象模型
2. dependency graph editor
3. theorem index / relation inspector
4. manuscript / archive / graph 的三视图联动
5. math portability provider

新增代码预算:

- 1,400 至 2,000 行

## 7. 代码增量预算

按照上面的范围，合理新增代码量区间大致为:

- 最低: 10,800 行
- 中位: 13,500 行
- 高位: 15,600 行

这满足“新增至少 1w 行代码”的要求，而且仍然保持结构化。

## 8. 推荐的目录级改造

### 8.1 前端

新增建议目录:

- `frontend/src/kb/registry/`
- `frontend/src/kb/contracts/`
- `frontend/src/kb/runtime/`
- `frontend/src/features/assets/`
- `frontend/src/features/prkb/`
- `frontend/src/features/vrkb/`
- `frontend/src/features/memos/`
- `frontend/src/features/english/`
- `frontend/src/features/math/`
- `frontend/src/tests/e2e/`
- `frontend/src/tests/component/`

### 8.2 后端

新增建议目录:

- `backend/src/application/kb_runtime/`
- `backend/src/application/portability/`
- `backend/src/application/assets/`
- `backend/src/application/prkb/`
- `backend/src/application/vrkb/`
- `backend/src/application/memos/`
- `backend/src/application/english/`
- `backend/src/application/math/`
- `backend/src/interface/api/contracts/`
- `backend/tests/integration/`

## 9. 测试策略

### 9.1 后端测试

必须建立:

1. repository tests
2. service tests
3. API integration tests
4. portability round-trip tests
5. fixture-based KB snapshot tests

覆盖重点:

- vocabulary sentence anchoring
- asset contextual authorization
- prkb feed ingest / dedupe
- vrkb project / finding lifecycle
- memo workflow / settings persistence
- math graph consistency
- backup / import / export round-trip

### 9.2 前端测试

当前没有正式测试脚本，这是明显缺口。

必须建立:

1. Vitest + Vue Test Utils
2. Playwright E2E
3. fixture-driven mock server

覆盖重点:

- Self Space 启动与 KB 切换
- Import/Export flows
- Assets picker
- PRKB screening workflow
- VRKB finding lifecycle
- Memo stream capture
- English selection -> save vocab
- Math graph navigation

### 9.3 CI 基线

建议标准:

- backend:
  - `cargo fmt --check`
  - `cargo clippy`
  - `cargo test`
- frontend:
  - `npm run build`
  - `npm run test:unit`
  - `npm run test:e2e`

## 10. 验收标准

一轮成熟化完成后，必须满足以下条件:

1. 任意特殊 KB 的 `renderer_id` 不再依赖 alias 猜测。
2. 每个特殊 KB 都有:
   - 独立 plugin 声明
   - 独立 capability 描述
   - 独立 portability provider
   - 独立 smoke test
3. `cargo test` 全绿。
4. 前端拥有正式测试脚本而不是只靠 `build`。
5. Self Space 路由、Dock、Open App、Pinned App 在刷新后行为一致。
6. 导入/导出可 round-trip，且不会破坏现有数据。
7. 特殊 KB 之间的引用关系能被追踪。

## 11. 风险清单

### 高风险

1. renderer / plugin / layout / provider 身份不统一
2. Portability provider 大面积 fallback
3. 测试缺失导致改动越大越不敢重构

### 中风险

1. PRKB 和 VRKB 业务逻辑增长过快，若无应用层会继续堆在 repository / API
2. Assets 若不尽快中心化，会在多个模块里继续重复实现上传与引用逻辑

### 低风险

1. UI 细节与视觉风格优化
2. 文案与局部交互 polish

## 12. 推荐执行顺序

严格建议按以下顺序推进:

1. Phase 0: 协议 + 测试基线
2. Phase 1: Portability 2.0
3. Phase 2: Assets KB
4. Phase 3: PRKB V2
5. Phase 4: VRKB
6. Phase 5: Memos
7. Phase 6: English / Vocabulary
8. Phase 7: Math

原因:

- Assets 和 Portability 是很多特殊 KB 的共用基础设施
- PRKB / VRKB 是目前最具业务价值但完成度不均的模块
- English / Memos / Math 虽然已有较强前端表现，但协议和测试仍需在基础设施稳定后统一收口

## 13. 我作为代码专家的最终判断

如果目标是“做到我认为成熟”，那么当前最正确的做法不是随机挑模块加功能，而是:

1. 先把特殊 KB 统一为同一种系统协议
2. 再把 portability 和 assets 提升为平台级基础设施
3. 最后对 PRKB / VRKB / Memos / English / Math 做纵深补全

这是唯一能在新增 1w+ 行代码的同时，仍然保持系统可维护、可验证、可扩展的路径。

如果直接跳过协议层去堆功能，代码量会增长得更快，但成熟度不会真正提高，后续维护成本会急剧上升。
