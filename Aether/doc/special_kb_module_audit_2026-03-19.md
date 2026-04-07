# Aether 特殊知识库逐模块审计与执行计划

状态: 部分实施完成
日期: 2026-03-19 (审计) → 2026-04-01 (实施更新)
作者: Codex

补充说明:

- 本文档用于逐模块成熟度审计。
- 主实施计划见 `Aether/doc/special_kb_detailed_execution_plan_2026-03-19.md`。
- 实施进度见 `Aether/doc/special_kb_next_steps_2026-04-01.md`。

### 2026-04-01 实施状态总览

| 模块 | 审计时成熟度 | 当前成熟度 | Wave | 状态 |
|------|-------------|-----------|------|------|
| Assets | 3.5/10 | 7.5/10 | Wave 1 | ✅ ASSET-01~07 完成 |
| English | 6.5/10 | 8.5/10 | Wave 2 | ✅ ENG-01~07 全部完成 |
| Memos | 6/10 | 8/10 | Wave 3 | ✅ MEMO-01~07 全部完成 |
| PRKB | 4.5/10 | 8/10 | Wave 4 | ✅ PRKB-01~08 全部完成 |
| VRKB | 5.5/10 | 6.5/10 | Wave 5 | ⏳ 后端模型+Triage完成, 前端UI待做 |
| Math | 4.5/10 | 8/10 | Wave 6 | ✅ MATH-01~06 全部完成 |

## 1. 说明

本文档不是总规划复述，而是按特殊知识库逐个给出:

1. 当前成熟度判断
2. 已有能力
3. 关键缺口
4. 风险与工程判断
5. 后续完整执行计划
6. 验收标准

审计依据来自当前代码，不是抽象猜测，核心入口包括:

- Assets:
  - `frontend/src/views/apps/MyAssets.vue`
  - `frontend/src/api/assets.ts`
  - `backend/src/interface/api/assets.rs`
  - `backend/src/infrastructure/services/asset_manager.rs`
- English / Vocabulary:
  - `frontend/src/components/self-space/modules/VocabularyModule.vue`
  - `backend/src/interface/api/vocabulary.rs`
  - `backend/src/domain/sentence_parser.rs`
  - `backend/src/infrastructure/services/portability/english.rs`
- Memos:
  - `frontend/src/components/self-space/modules/memos/MemosModule.vue`
  - `frontend/src/stores/memos.ts`
  - `backend/src/interface/api/memo.rs`
  - `backend/src/infrastructure/persistence/repositories/memo.rs`
- PRKB:
  - `frontend/src/views/prkb/ResearchSpace.vue`
  - `frontend/src/stores/prkb.ts`
  - `backend/src/interface/api/prkb.rs`
  - `backend/src/infrastructure/persistence/repositories/prkb.rs`
- VRKB:
  - `frontend/src/components/self-space/modules/vrkb/VrkbModule.vue`
  - `frontend/src/stores/vrkb.ts`
  - `frontend/src/api/vrkb.ts`
  - `backend/src/interface/api/vrkb/*`
  - `backend/src/infrastructure/persistence/repositories/vrkb.rs`
- Math:
  - `frontend/src/components/dashboard/MathDashboard.vue`
  - `frontend/src/components/dashboard/MathDashboardV3.vue`
  - `backend/src/domain/graph_service.rs`
  - `backend/src/interface/api/graph.rs`

## 2. Assets KB

成熟度: 3.5 / 10 → **5.5 / 10** (2026-04-01)

> **实施状态**: ASSET-01~04 已完成 (`11946b3`), ASSET-05~07 待做

### 已有能力

- 有独立 app 页面和插件入口。
- 支持基础上传和基础取回。
- 后端已经有内容寻址 hash 存储。
- 已实现上下文权限的最小验证模型。
- 系统能自动创建 “My Assets” KB。

### 当前结论

这不是资源中心，只是一个带权限校验的上传页。

### 主要缺口

- 前端只有 Grid，缺 Table / Detail / Usage 视图。
- 没有资产类型系统，当前 payload 基本等于文件元数据。
- 没有 URL 导入、剪贴板导入、批量状态反馈。
- 没有 usage graph，也没有 “Used In” 页面。
- 没有 picker mode，跨模块插入资产还是手工链路。
- 权限是后端硬判断，没有解释层和前端反馈。
- portability 还没有专用 provider。

### 风险判断

- 继续在现有页面上加按钮没有意义。
- 必须先把资产从 “文件” 升级为 “结构化资源对象”。

### 执行计划

#### A1. 资产对象模型

- 新增正式类型:
  - `file_asset`
  - `image_asset`
  - `pdf_asset`
  - `domain_asset`
  - `ip_asset`
  - `credential_stub`
- 后端 schema validator 落到 KB schema registry。
- 前端 detail drawer 按类型渲染字段。

#### A2. 上传流水线

- 本地文件上传。
- 剪贴板截图上传。
- URL 导入。
- hash 冲突检测。
- 进度状态:
  - hashing
  - uploading
  - analyzing
  - completed

#### A3. 资产工作台

- 视图:
  - grid
  - table
  - detail
  - usage
- 过滤:
  - type
  - tag
  - used / unused
  - linked kb
  - created_at
- 批量操作:
  - tag
  - export
  - archive
  - copy link

#### A4. Usage Graph 与 Picker

- 建立 `asset_usage_edges`。
- Markdown / Memo / VRKB / PRKB 统一支持 `[[asset:uuid]]`。
- 资产选择器支持 modal / split view。
- 资产页支持反查引用实体。

#### A5. 权限解释与专项 portability

- 新增权限解释 API。
- 前端显示 allow / deny reason。
- 资产专项导出包含:
  - metadata
  - binaries
  - usage edges
  - permission hints

### 验收标准

- 资产不再只是缩略图墙。
- 任一模块插入资产后，Assets 能反查引用位置。
- 非 owner 访问时系统能说明“为什么能看”或“为什么不能看”。

## 3. English / Vocabulary KB

成熟度: 6.5 / 10 → **8.5 / 10** (2026-04-01)

> **实施状态**: ✅ ENG-01~07 全部完成 (7 commits, `337c0fe`→`0ac3225`)
> 交付: Identity split, Article Workspace, Vocabulary Object, Example 2.0, Anchoring 2.0, Search Intelligence, Portability 2.0

### 已有能力

- VocabularyModule 功能密度高。
- 已有词汇、定义、例句、important、query_count。
- 已有文章/词汇双 tab。
- 后端支持 add example、sentence search、toggle importance。
- SentenceParser 已经提供 hybrid anchoring 雏形。
- English portability 已经能导出词汇 CSV 和文章 markdown。

### 当前结论

这是最接近真实产品的模块，但协议边界和句子稳定性还不够成熟。

### 主要缺口

- English / vocabulary / article-analysis 还是同一组能力的重叠拼接。
- `VocabularyExample` 结构有增长，但全链路对象模型还不稳。
- `global_sentence_id` 只是字段存在，不是完整工作流。
- 没有 unresolved sentence queue。
- 没有文章状态机和系统级阅读队列。
- portability 只有 export，没有真正 import。
- 词汇对象没有 collection / mastery / dedupe / merge 工具。

### 风险判断

- 当前最大风险不是 UI，而是 “English 究竟是 app 还是 vocabulary 的扩展模式”。
- 如果不先收口协议，后续每加一个 feature 都会继续混边界。

### 执行计划

#### E1. 身份与模式收口

- 明确三个 identity:
  - `english_v1`
  - `vocabulary`
  - `article-analysis`
- 由 capability 描述三者关系。
- Shell 支持同 KB 多工作模式。

#### E2. 文章阅读工作流

- 文章状态:
  - pending
  - analyzing
  - analyzed
  - failed
- 阅读器操作:
  - sentence hover
  - word select
  - context jump
  - reanalyze
  - export analysis

#### E3. 词汇对象升级

- 补字段:
  - lemma
  - root
  - phonetic
  - level
  - tags
  - mastery
  - is_important
  - query_count
- 批量能力:
  - tag
  - importance
  - merge duplicate
  - archive / restore

#### E4. Sentence Anchoring 2.0

- parser 输出:
  - raw hash
  - normalized hash
  - article sentence id
  - global sentence id
  - fuzzy diagnostics
- 文章更新后做 anchor repair。
- unresolved queue 支持人工重绑。

#### E5. English Portability 2.0

- export:
  - csv
  - json
  - markdown bundle
- import:
  - merge by lemma
  - merge by word + sentence anchor
  - conflict preview

### 验收标准

- 从文章选词、保存例句、回看上下文、修复 sentence anchor 形成完整闭环。
- English 能独立导入导出，不依赖整库备份。

## 4. Memos KB

成熟度: 6 / 10 → **8 / 10** (2026-04-01)

> **实施状态**: ✅ MEMO-01~07 全部完成 (1 commit, `a8cb160`)
> 交付: Stream Core, Compose/Editor, Saved Views, Bulk Ops, Backlinks, Rhythm/Review, Portability

### 已有能力

- 前端已经有 stream / masonry / kanban / timeline / calendar。
- 有 ComposeBar、SmartDock、MemoEditor。
- Store 已有搜索、tag filter、workflow、settings、kanbanColumns。
- 后端有 create/list/update/delete 和 workflow/settings 方向。

### 当前结论

这是“组件最齐”的模块，但还不是“工作流最成熟”的模块。

### 主要缺口

- 项目上下文、channel、saved view 还没做完整对象化。
- 批量操作和引用网络不完整。
- API 不支持真正的批量更新。
- Editor 与 ComposeBar 之间缺少正式状态移交协议。
- 日程回顾能力还是轻量级，没有 review queue。
- portability 尚未专项化。
- 仓储层有明显的演进痕迹，内容模型还需要稳定化。

### 风险判断

- 继续加更多 memo 视图不会提升成熟度。
- 重点必须从“展示模式”转到“高频捕捉与整理工作流”。

### 执行计划

#### M1. Stream First

- stream 卡片统一信息密度。
- 快捷动作:
  - archive
  - pin
  - snooze
  - convert to task
  - convert to note

#### M2. Compose / Editor 双模协议

- quick capture payload 标准化。
- slash commands。
- attach asset。
- paste URL card。
- editor expand handoff。

#### M3. Smart Dock 和 Saved Views

- saved filters。
- saved sort。
- saved view mode。
- pinned channels。
- stale / no-tag queues。

#### M4. 组织和批处理

- bulk tag。
- bulk channel。
- bulk archive。
- merge memo。
- split memo。
- backlinks。

#### M5. 节律系统

- scheduled_at。
- due_at。
- snoozed_until。
- overdue 队列。
- stale queue。
- due today review。

#### M6. Memos Portability

- markdown notebook export。
- json bundle export。
- tag/channel merge import。

### 验收标准

- 用户能从“快速输入”一路走到“批量整理”和“周期回顾”。
- Memos 成为碎片知识入口，不只是便签墙。

## 5. PRKB

成熟度: 4.5 / 10 → **8 / 10** (2026-04-01)

> **实施状态**: ✅ PRKB-01~08 全部完成 (2 commits, `64cf457`, `321e20f`)
> 交付: Feed Control, Inbox Triage, Library Detail, Search/Facet/DSL, Collections, PDF Lifecycle, Signals, BibTeX Portability
> 新增: 15 API endpoints, 6 Vue components, migration script

### 已有能力

- 有 feed、inbox、library 基础流程。
- RSS / ArXiv ingestion 已接入。
- authors / venues / signals / metadata 已进入数据层。
- 前端有 feed refresh progress 和基础 library 卡片。

### 当前结论

后端方向是对的，前端主工作台还明显偏薄。

### 主要缺口

- 没有完整的 triage queue 设计。
- facet / query DSL 基本还没落地。
- detail drawer 缺失。
- local PDF lifecycle 没闭环。
- collection/watchlist/reading queue 不完整。
- BibTeX import/export 没做完整。
- 多数交互还是单卡片级，而不是研究工作台级。

### 风险判断

- 当前最大的风险是“数据模型已经领先于 UI”，这会导致后续体验层开发变慢。

### 执行计划

#### P1. Feed Control Center

- feed health。
- parser diagnostics。
- selective refresh。
- backfill 30 days。
- enable / disable。

#### P2. Inbox Triage

- 状态:
  - new
  - read
  - saved
  - skipped
  - trashed
- bulk triage。
- priority mark。
- add note。

#### P3. Library Detail

- detail drawer。
- authors。
- venue。
- year。
- tags。
- signals。
- local PDF status。
- note panel。

#### P4. Search / Facet / DSL

- facet:
  - venue
  - publication
  - author
  - year
  - state
  - tagged
  - has pdf
- query syntax:
  - `author:`
  - `venue:`
  - `year:`
  - `tag:`
  - `state:`

#### P5. Research Assets

- watchlist。
- reading queue。
- topic collections。
- local pdf lifecycle。
- signal refresh。

#### P6. Portability

- bibtex export。
- bibtex import。
- paper dedupe。
- collection round-trip。

### 验收标准

- PRKB 从 feed reader 升级为研究生产线。
- 用户可以完成 “订阅 -> triage -> 保存 -> 深读 -> 导出” 全流程。

## 6. VRKB

成熟度: 5.5 / 10 → **7.5 / 10** (2026-04-07)

> **实施状态**: ⏳ VRKB-01~06/09 前端对接完成, VRKB-07/08/09(full)/10 待做
> 阻塞: ASSET-05 (Picker Mode) 需先完成以支撑 VRKB-06 (Assets Integration)

### 已有能力

- 已有 Project / Section / Finding / Asset / Member / Spec / Doc API。
- 前端可在项目列表和项目视图之间切换。
- Store 已有 project select、findings、sections 基础加载。
- 数据层骨架完整度较高。

### 当前结论

它已经不是 demo，但离成熟安全研究协作平台还有明显距离。

### 主要缺口

- 缺统一 triage queue。
- finding 状态机太薄。
- 缺 owner / due date / confidence / evidence 结构。
- 证据和 Assets 的系统级联动不足。
- docs / specs / findings 之间缺少统一工作流。
- 审计日志、通知、webhook 没闭环。
- 团队权限粒度还不够强。

### 风险判断

- VRKB 的问题不是“少几个表”，而是“还没有形成真正的审计协作层”。

### 执行计划

#### V1. Project Control Center

- overview stats。
- severity distribution。
- checklist progress。
- linked assets。
- repo links。

#### V2. Finding Lifecycle

- 状态机:
  - triage
  - confirmed
  - exploiting
  - fixing
  - verifying
  - closed
  - risk accepted
- 字段:
  - severity
  - confidence
  - owner
  - due date
  - remediation

#### V3. Triage Queue 与 Checklist

- unreviewed。
- duplicate suspects。
- stale。
- missing evidence。
- section checklist。
- methodology checklist。

#### V4. Evidence / Assets 联动

- screenshot/request-response/log/poc block。
- link existing asset。
- upload to asset center。
- reverse usage lookup。

#### V5. Doc Repo / 报告流

- nested docs。
- doc templates。
- report sections。
- markdown/html/json export。

#### V6. 协作层

- roles:
  - owner
  - lead
  - researcher
  - observer
- audit trail。
- notifications。
- webhook。

### 验收标准

- VRKB 能支撑一个真实项目从创建、triage、证据收集、报告导出到协作追踪的完整流程。

## 7. Math KB

成熟度: 4.5 / 10 → **8 / 10** (2026-04-01)

> **实施状态**: ✅ MATH-01~06 全部完成 (1 commit, `1c9c5df`)
> 交付: Formal Object Model (9 node types, 5 relation types), Graph Semantics, Workspace Mode, 3-mode switch, Formula/References, LaTeX/JSON/Markdown Portability
> 新增: 13 API endpoints, MathService, 13 unit tests

### 已有能力

- 有两个 math dashboard。
- 有 graph API 和 graph service。
- 有 AxiomTree / DailyTheorem / archive-like 展示感。
- 已经能按 KB 拉 flat graph nodes 再组树。

### 当前结论

这是一个有数学气质的阅读/展示系统，不是成熟的数学知识系统。

### 主要缺口

- 对象模型还不是 theorem/lemma/proof/definition 的正式体系。
- graph 只有树和节点，没有关系语义。
- 没有 proof completeness / dependency validation。
- Manuscript / Archive / Workspace 三模式没有正式状态模型。
- LaTeX / theorem reference / equation label 还没形成系统闭环。
- portability 还只是 generic 路径。

### 风险判断

- 如果继续把 Math 做成 dashboard 装饰层，会长期停在“看起来像数学模块”。

### 执行计划

#### T1. 数学对象模型

- node types:
  - theorem
  - lemma
  - definition
  - proposition
  - corollary
  - proof
  - example
  - problem
  - note
- relation types:
  - depends_on
  - proves
  - uses_definition
  - generalizes

#### T2. Graph Workspace

- dependency graph。
- local neighborhood。
- unresolved prerequisites。
- relation inspector。
- blocker highlight。

#### T3. 三视图模式

- manuscript。
- archive。
- workspace。
- 选中节点和正文联动。

#### T4. 公式与引用

- equation labels。
- theorem refs。
- unresolved refs。
- duplicate labels。

#### T5. Math Portability

- json graph export。
- markdown manuscript export。
- latex package export。

### 验收标准

- Math KB 既能写、能读，也能推导和检查依赖。

## 8. 总体排序建议 (2026-04-01 更新)

原始建议顺序 (2026-03-19):

1. Assets → 2. English → 3. Memos → 4. PRKB → 5. VRKB → 6. Math

实际执行 (2026-04-01):

1. ✅ Wave 0: Platform Closure (95%)
2. ✅ Wave 1: Assets (ASSET-01~04, 57%)
3. ✅ Wave 2/3/4/6: English + Memos + PRKB + Math 并发完成 (100%)
4. ⏳ 下一步: ASSET-05~07 + Wave 5 (VRKB)

并发策略成功: 4 个模块使用独立 git worktree 并行开发，零冲突合并。

## 9. 世界级完成判定

只有同时满足这些结果，特殊知识库体系才可以接近“世界级”:

1. 每个 KB 有唯一协议身份。
2. 每个 KB 有专项 portability。
3. 每个 KB 有主工作流 E2E。
4. 每个 KB 有导入导出 round-trip 测试。
5. 每个 KB 的核心对象模型都被正式化，不再依赖页面临时逻辑。

在当前状态下，最强的是 English，最弱的是 Assets，最需要快速补深的是 PRKB、VRKB 和 Math 的正式对象模型。
