# Aether 特殊知识库异常详细实施计划

状态: 实施中 (Wave 0/1/2/3/4/6 已完成或接近完成, Wave 5/7/8/9 待做)
日期: 2026-03-19 (规划) → 2026-04-01 (进度更新)
作者: Codex

关联文档:

- 总规划: `Aether/doc/special_kb_maturity_plan.md`
- 功能 backlog: `Aether/doc/special_kb_feature_backlog.md`
- 逐模块审计: `Aether/doc/special_kb_module_audit_2026-03-19.md`

## 1. 文档定位

这份文档是后续实施的主计划文档。

它不是愿景文档，也不是模块介绍，而是把后续工作拆解到足以直接执行的颗粒度:

1. 平台工作包
2. 模块工作包
3. 数据模型变更
4. 后端 API 变更
5. 前端交互变更
6. portability 变更
7. 测试矩阵
8. 依赖关系
9. 验收标准
10. 提交顺序建议

目标是让 Aether 的特殊知识库体系从“强原型”升级为“接近世界级的成熟系统”。

## 2. 计划原则

### 2.1 核心原则

- 不允许再靠 alias 和 fallback 堆行为。
- 不允许只有页面，没有对象模型。
- 不允许只有后端结构，没有前端工作流。
- 不允许只有功能，没有导入导出。
- 不允许只有测试框架，没有真实关键路径测试。

### 2.2 优先顺序原则

- 先平台协议，再模块纵深。
- 先资源底座，再上层知识库。
- 先高频工作流，再长尾功能。
- 先闭环，再扩展。

### 2.3 完成定义

一个特殊知识库只有满足以下条件，才算进入“成熟”状态:

1. 具有唯一身份:
   - `renderer_id`
   - `plugin.id`
   - `layout.id`
   - `dashboard.id`
   - `portability.provider_id`
2. 具有完整主工作流:
   - 创建或导入
   - 列表与检索
   - 详情与编辑
   - 专项工作台
   - 导入导出
3. 具有系统级支撑:
   - 权限
   - 审计
   - 长任务进度
   - 错误解释
4. 具有质量闭环:
   - 单测
   - 集成测试
   - E2E
   - round-trip portability test

## 3. 当前基线

截至 2026-03-19，已完成的前置工作:

- 前端已有 special KB registry 雏形:
  - `frontend/src/registries/special_kb_registry.ts`
- 插件解析开始收敛:
  - `frontend/src/stores/plugins.ts`
- Self Space Orchestrator 已接入 canonical renderer 解析:
  - `frontend/src/composables/useSelfSpaceOrchestrator.ts`
- 后端 portability service 已改为 canonical provider + alias map:
  - `backend/src/infrastructure/services/portability_service.rs`
  - `backend/src/infrastructure/bootstrap/services.rs`
- 现有测试基线已经可跑通:
  - `cargo test`
  - `npm run test:unit`
  - `npm test`
  - `npm run build`

当前仍存在的明确基线问题:

- backend `asset_manager.rs` 与 `knowledge_base.rs` 仍硬编码 `assets_v1`
- backend create/update KB 还没有统一 renderer canonical normalize
- frontend `read_layout_registry.ts` 仍缺 registry 完整性校验
- Shell 仍有 `admin_system` 的系统应用硬编码假设
- frontend build 仍有 chunk warning
- English analyzer 仍有动态/静态混合 import warning

## 4. 实施总顺序

总顺序分 10 个执行波次。

### Wave 0. 平台收口

目标:

- 协议统一
- Shell 行为稳定
- portability 基础能力稳定
- 测试底座稳定

### Wave 1. Assets 底座

目标:

- 把 Assets 从上传页升级为资源中枢
- 为后续 Memos / PRKB / VRKB 提供统一资源基础设施

### Wave 2. English / Vocabulary 完整化

目标:

- 把当前最成熟模块提升为真正独立的语言知识系统

### Wave 3. Memos 高成熟化

目标:

- 把多视图 memo 模块收敛成高频知识捕捉与整理工作流

### Wave 4. PRKB 专业研究化

目标:

- 从 feed reader 升级为研究工作台

### Wave 5. VRKB 协作平台化

目标:

- 从带看板的项目模块升级为审计协作平台

### Wave 6. Math 正式化

目标:

- 从数学展示面升级为数学对象与依赖系统

### Wave 7. Portability 2.0 全量化

目标:

- 为六个特殊 KB 补齐专项 provider 和 round-trip 测试

### Wave 8. Observability / Security / Performance

目标:

- 把长任务、权限、导入导出、错误与性能指标纳入可观测范围

### Wave 9. 发布与稳定版收口

目标:

- 文档、发布、回归套件、迁移说明完整

## 5. 平台工作包

本节是所有特殊 KB 共享的工作。

### PLAT-01. Renderer Canonicalization

目标:

- 所有 KB 都由唯一 canonical renderer 驱动

当前缺口:

- backend create/update/list 还未统一 normalize
- assets 与 system 仍残留硬编码

交付:

- backend renderer normalization helper
- frontend renderer normalization helper 继续收口
- migration table 单点化
- legacy telemetry

后端变更:

- 新增 renderer normalize helper
- `create_knowledge_base_handler` 在保存前 normalize `renderer_id`
- `update_knowledge_base_handler` 在更新前 normalize `renderer_id`
- `asset_manager.rs` 与 `knowledge_base.rs` 改为通过 helper 判断 assets renderer

前端变更:

- `special_kb_registry.ts` 补充完整性自检
- 所有 `renderer_id === 'xxx'` 改为 helper 判定

测试:

- backend normalize unit test
- legacy renderer create/update test
- assets auto create canonical renderer test

提交建议:

- `refactor(protocol): canonicalize renderer ids across kb lifecycle`

验收:

- 任何旧 renderer id 写入后都会被收敛为 canonical renderer

### PLAT-02. Shell State Machine

目标:

- 让 Self Space 像 OS，而不是一次性页面切换器

交付:

- active / running / pinned / minimized / crashed 状态机
- singleton / multi-instance 行为统一
- deep link 协议
- session restore 协议
- error boundary 标准化

前端变更:

- `useSelfSpaceOrchestrator.ts`
- `SelfSpaceView.vue`
- `read_app_state.ts`
- `preferences.ts`

测试:

- deep link reopen test
- session restore test
- crash isolation test
- singleton dedupe test

验收:

- 单个模块崩溃不影响其他模块
- 刷新可恢复最近实例

### PLAT-03. Header Action Protocol

目标:

- Shell 顶栏动作协议统一

交付:

- header action def
- plugin injected header actions
- progress badge protocol
- context badge protocol

前端变更:

- `core/plugin.ts`
- `SelfSpaceView.vue`
- 各模块 header action adapters

测试:

- plugin header action render test
- action dispatch test

### PLAT-04. Portability Runtime

目标:

- 统一 export/import runtime，而不是每个模块各写一套流程

交付:

- export preview service
- import analysis service
- conflict preview DTO
- long task lifecycle
- download token / expiry

后端变更:

- `portability_service.rs`
- `interface/api/portability.rs`
- provider trait 扩展

前端变更:

- `ExportModal.vue`
- `ImportModal.vue`
- progress store / event parser

测试:

- progress event order test
- task result test
- expired download test
- import conflict preview render test

### PLAT-05. Test Infrastructure

目标:

- 让后续每个模块新增功能时都能直接落在稳定测试底座上

交付:

- backend fixtures:
  - user fixture
  - kb fixture
  - article fixture
  - asset fixture
  - paper fixture
  - project/finding fixture
- frontend helpers:
  - auth mock
  - plugin registry bootstrap
  - shell mount helper
  - SSE mock helper
- Playwright smoke suite

测试:

- smoke:
  - launch shell
  - open assets
  - open english
  - open memos
  - open prkb
  - open vrkb
  - open math

### PLAT-06. Observability Foundation

目标:

- 后续所有特殊 KB 都能记录:
  - actor
  - target
  - action
  - context
  - result

交付:

- structured audit event schema
- long task telemetry schema
- frontend error boundary event schema

## 6. Assets KB 详细计划

### 6.1 目标状态

Assets 必须从“上传页”升级为“资源中枢”，达到以下目标:

- 支持多种结构化资产类型
- 支持 usage graph
- 支持 picker mode
- 支持上下文权限解释
- 支持专项导入导出
- 成为 PRKB / VRKB / Memos / Markdown 的统一资源层

### 6.2 入口文件

- 前端:
  - `frontend/src/views/apps/MyAssets.vue`
  - `frontend/src/api/assets.ts`
- 后端:
  - `backend/src/interface/api/assets.rs`
  - `backend/src/infrastructure/services/asset_manager.rs`
  - `backend/src/domain/kb/schemas/assets.rs`

### 6.3 工作包

#### ASSET-01. Typed Asset Schema

目标:

- 给资产正式对象模型

交付类型:

- `file_asset`
- `image_asset`
- `pdf_asset`
- `domain_asset`
- `ip_asset`
- `credential_stub`
- `snippet_asset`

数据字段:

- 通用:
  - `asset_id`
  - `type`
  - `title`
  - `hash`
  - `mime_type`
  - `size_bytes`
  - `tags`
  - `created_at`
  - `updated_at`
- image:
  - `width`
  - `height`
  - `dominant_color`
  - `exif_summary`
- pdf:
  - `page_count`
  - `text_extract_status`
  - `source_url`
- domain:
  - `fqdn`
  - `registrar`
  - `environment`
  - `risk_note`
- ip:
  - `cidr`
  - `provider`
  - `environment`
  - `exposure_note`
- credential_stub:
  - `label`
  - `scope`
  - `rotation_due_at`
  - `linked_assets`

后端变更:

- schema registry 注册所有类型
- upload 时根据 mime / import source 推断类型
- 提供 schema validation error

前端变更:

- detail drawer
- type pill
- type-aware metadata panel

测试:

- schema validation unit test
- invalid type payload rejection test
- detail drawer render test

#### ASSET-02. Upload Pipeline

目标:

- 把上传从“黑盒请求”升级成可观测任务

功能:

- local file upload
- drag & drop upload
- paste screenshot upload
- URL import
- hash dedupe
- version reuse prompt

后端变更:

- upload progress events
- URL import endpoint
- duplicate hash lookup

前端变更:

- upload queue drawer
- progress row
- duplicate conflict modal

测试:

- upload progress test
- duplicate hash test
- URL import test
- paste image test

#### ASSET-03. Asset Console

目标:

- 提供完整资产工作台

视图:

- grid
- table
- detail
- usage

过滤器:

- type
- tag
- used / unused
- referenced by KB
- created_at range
- owner

排序:

- newest
- largest
- most used
- recently referenced

前端变更:

- 重写 `MyAssets.vue` 为多视图工作台
- 新增:
  - `AssetsTable.vue`
  - `AssetDetailDrawer.vue`
  - `AssetUsagePanel.vue`
  - `AssetFiltersBar.vue`

测试:

- grid/table toggle test
- sort test
- filter combination test

#### ASSET-04. Usage Graph

目标:

- 记录资产如何被其他模块使用

边:

- asset -> article
- asset -> memo
- asset -> paper note
- asset -> finding
- asset -> vrkb doc

后端变更:

- usage edge table or relation layer
- create/delete usage edge hooks
- reverse lookup API

前端变更:

- usage tab
- "Used In" chips
- jump to source entity

测试:

- usage edge create test
- usage edge delete test
- reverse lookup API test

#### ASSET-05. Picker Mode

目标:

- 跨模块统一选择资产

接入模块:

- Markdown editor
- Memos compose
- VRKB evidence
- PRKB note / PDF attach

功能:

- modal picker
- split-view picker
- search in picker
- recent assets
- upload from picker

测试:

- picker open test
- picker select test
- picker upload and insert test

#### ASSET-06. Permission Explanation

目标:

- 不只判断权限，还要解释权限

输出:

- `allowed: boolean`
- `reason_code`
- `reason_text`
- `context_chain`
- `referenced_by`

测试:

- owner access explanation test
- context access explanation test
- denied explanation test

#### ASSET-07. Assets Portability

目标:

- 资产独立可迁移

导出内容:

- metadata
- binaries
- usage edges
- permission hints

导入策略:

- dedupe by hash
- merge metadata
- restore usage edges

测试:

- assets round-trip test
- binary integrity test
- usage edge round-trip test

### 6.4 Assets 验收门槛

- 用户能上传、检索、筛选、查看详情、反查引用、从其他模块选择资产
- 资产权限可以解释
- 资产可以独立导出导入

## 7. English / Vocabulary 详细计划

### 7.1 目标状态

English 必须从“词汇页 + 分析页的组合”升级为“语言知识工作台”。

### 7.2 入口文件

- 前端:
  - `frontend/src/components/self-space/modules/VocabularyModule.vue`
  - `frontend/src/components/self-space/modules/ArticleAnalysisModule.vue`
- 后端:
  - `backend/src/interface/api/vocabulary.rs`
  - `backend/src/domain/sentence_parser.rs`
  - `backend/src/infrastructure/services/portability/english.rs`

### 7.3 工作包

#### ENG-01. Identity and Capability Split

目标:

- 收口:
  - `english_v1`
  - `vocabulary`
  - `article-analysis`

交付:

- capability map
- tab mode contract
- shell launch rules

测试:

- renderer identity resolution test
- tab restore test
- launch mode test

#### ENG-02. Article Workspace

目标:

- 建立 article inbox / library / analysis state machine

状态:

- pending
- analyzing
- analyzed
- failed
- archived

前端功能:

- article list
- article reader
- analysis status badge
- reanalyze action
- export analysis

后端功能:

- article analysis status field
- reanalyze endpoint
- failure diagnostics

测试:

- article status transition test
- reanalyze test
- failure render test

#### ENG-03. Vocabulary Object Upgrade

目标:

- 把词汇做成正式知识对象

字段:

- lemma
- word
- root
- phonetic
- level
- translation
- tags
- mastery
- query_count
- is_important
- source_kb_id

能力:

- batch tag
- batch importance
- merge duplicates
- archive
- restore

测试:

- duplicate merge test
- batch importance test
- archive/restore test

#### ENG-04. Example System 2.0

目标:

- 让例句成为稳定对象，而不是附加字符串

字段:

- example_id
- sentence
- translation
- note
- image_url
- article_id
- sentence_uuid
- global_sentence_id
- created_at

功能:

- multi-example per word
- primary example
- search examples
- unresolved example queue

测试:

- multi-example persistence test
- example search test
- unresolved example render test

#### ENG-05. Sentence Anchoring 2.0

目标:

- 让文章小改动不破坏句子绑定

parser 输出:

- exact hash
- normalized hash
- article-local sentence id
- global sentence id
- diagnostics

repair 流程:

- exact
- normalized
- fuzzy
- unresolved

前端功能:

- unresolved queue
- rebind sentence
- compare old/new sentence

测试:

- typo stability test
- paragraph move repair test
- unresolved manual bind test

#### ENG-06. Search and Intelligence

目标:

- 建立更强的 query pipeline

流程:

- dictionary lookup
- lemma normalize
- inflection resolve
- local vocab merge
- suggestion ranking

前端功能:

- family words
- collocations
- phrase hints
- custom collections

测试:

- fuzzy suggestion test
- lemma normalize test
- local remote merge test

#### ENG-07. English Portability 2.0

目标:

- 真正支持 English 独立导入导出

导出:

- csv
- json
- markdown bundle
- analysis bundle

导入:

- merge by lemma
- merge by word + anchor
- conflict preview
- Anki-like csv import

测试:

- csv export test
- json import merge test
- article + vocab round-trip test

### 7.4 English 验收门槛

- 从文章阅读、选词、保存例句、回看上下文、修复句子绑定、导出导入形成闭环

## 8. Memos 详细计划

### 8.1 目标状态

Memos 必须成为“高频捕捉 + 整理 + 回顾”系统，而不是多视图笔记集合。

### 8.2 入口文件

- 前端:
  - `frontend/src/components/self-space/modules/memos/MemosModule.vue`
  - `frontend/src/stores/memos.ts`
- 后端:
  - `backend/src/interface/api/memo.rs`
  - `backend/src/infrastructure/persistence/repositories/memo.rs`

### 8.3 工作包

#### MEMO-01. Stream Core

目标:

- stream 成为默认主工作台

卡片字段:

- title
- excerpt
- tags
- channel
- status
- updated_at
- linked entities

快捷操作:

- archive
- pin
- snooze
- convert to task
- convert to note

测试:

- stream grouping test
- stream quick action test

#### MEMO-02. Compose / Editor Contract

目标:

- 快速捕捉和深度编辑是一条连续工作流

compose bar:

- instant capture
- slash commands
- quick tags
- attach asset
- paste URL card

editor:

- markdown
- checklist
- references
- due_at
- reminder_at
- channel

测试:

- quick capture test
- editor handoff test
- asset chip insert test

#### MEMO-03. Saved Views and Dock

目标:

- 把筛选条件提升为正式对象

对象:

- saved_view
- pinned_tag
- channel
- review_queue

功能:

- save current filters
- save sort
- save view mode
- quick switch

测试:

- saved view persistence test
- pinned tag toggle test
- dock queue render test

#### MEMO-04. Organization and Bulk Ops

目标:

- 用户能批量整理碎片

功能:

- bulk tag
- bulk channel
- bulk archive
- bulk delete
- bulk status change
- merge memo
- split memo

后端变更:

- bulk update endpoint
- merge endpoint
- split endpoint

测试:

- bulk update API test
- merge memo test
- split memo test

#### MEMO-05. Backlinks and References

目标:

- memo 融入整个知识系统

引用目标:

- article
- asset
- paper
- finding
- doc

功能:

- mention picker
- backlink panel
- linked entities panel

测试:

- reference insert test
- backlinks render test

#### MEMO-06. Rhythm and Review

目标:

- memo 进入计划与回顾节律

字段:

- scheduled_at
- due_at
- snoozed_until
- reviewed_at

队列:

- due today
- overdue
- stale
- unresolved

测试:

- calendar move test
- overdue queue test
- stale review test

#### MEMO-07. Memos Portability

目标:

- memos 独立可迁移

导出:

- markdown notebook
- json bundle
- daily archive

导入:

- tag merge
- channel merge
- duplicate detection

测试:

- markdown export test
- duplicate import test
- memos round-trip test

### 8.4 Memos 验收门槛

- 用户能从快速输入一路走到批量整理和周期回顾

## 9. PRKB 详细计划

### 9.1 目标状态

PRKB 必须成为研究工作台，而不是订阅列表和卡片仓库。

### 9.2 入口文件

- 前端:
  - `frontend/src/views/prkb/ResearchSpace.vue`
  - `frontend/src/stores/prkb.ts`
- 后端:
  - `backend/src/interface/api/prkb.rs`
  - `backend/src/infrastructure/persistence/repositories/prkb.rs`

### 9.3 工作包

#### PRKB-01. Feed Control Center

目标:

- feed 具有可维护性和可诊断性

字段:

- name
- type
- source url
- health status
- last fetched
- total fetched
- parse errors

动作:

- enable
- disable
- manual fetch
- fetch selected
- test parser
- backfill

测试:

- create/delete feed test
- selected fetch test
- parser diagnostics test

#### PRKB-02. Inbox Triage

目标:

- 让 influx paper 可被高效处理

状态:

- new
- read
- saved
- skipped
- trashed

动作:

- mark read
- save
- skip
- trash
- add note
- mark priority

测试:

- inbox transition test
- bulk triage test
- unread filter test

#### PRKB-03. Library Detail Drawer

目标:

- library card 背后有研究对象详情页

字段:

- title
- authors
- venue
- abstract
- year
- tags
- signals
- read state
- pdf status
- notes

动作:

- attach pdf
- open source
- copy citation
- export bib entry

测试:

- detail drawer render test
- attach pdf test
- citation copy test

#### PRKB-04. Search / Facet / Query DSL

目标:

- 研究资料能被快速收敛

facet:

- venue
- publication
- author
- year
- state
- has pdf
- tagged

dsl:

- `author:`
- `venue:`
- `year:`
- `state:`
- `tag:`

测试:

- facet combination test
- query parser test
- saved search test

#### PRKB-05. Collections and Queues

目标:

- 论文不只是“已保存”，而是进入组织结构

对象:

- watchlist
- reading_queue
- archive
- topic_collection

测试:

- collection move test
- queue reorder test

#### PRKB-06. PDF Lifecycle

目标:

- 本地 PDF 成为正式生命周期对象

状态:

- not_attached
- queued
- downloaded
- indexed
- failed

测试:

- pdf lifecycle test
- failed state retry test

#### PRKB-07. Signals

目标:

- 系统能对 paper 提供基础智能排序信号

信号:

- feed freshness
- venue tier
- author recurrence
- citation placeholder
- custom importance

测试:

- signal refresh test
- ranking order test

#### PRKB-08. PRKB Portability

目标:

- research library 可独立迁移

导出:

- bibtex
- json
- markdown digest
- collection bundle

导入:

- bibtex import
- dedupe by doi / external_id / title
- merge tags / notes

测试:

- bibtex export test
- bibtex import test
- collection round-trip test

### 9.4 PRKB 验收门槛

- 用户能完成:
  - 订阅
  - 抓取
  - triage
  - 保存
  - 深读
  - 导出

## 10. VRKB 详细计划

### 10.1 目标状态

VRKB 必须成为安全研究协作平台，而不是“项目 + finding + 文档”的松散集合。

### 10.2 入口文件

- 前端:
  - `frontend/src/components/self-space/modules/vrkb/VrkbModule.vue`
  - `frontend/src/stores/vrkb.ts`
  - `frontend/src/api/vrkb.ts`
- 后端:
  - `backend/src/interface/api/vrkb/*`
  - `backend/src/infrastructure/persistence/repositories/vrkb.rs`

### 10.3 工作包

#### VRKB-01. Project Control Center

目标:

- 项目创建后立刻进入结构化工作台

区块:

- scope summary
- status summary
- severity distribution
- checklist completion
- linked assets
- timeline summary

测试:

- project overview test
- stats render test

#### VRKB-02. Finding Lifecycle

目标:

- finding 具有专业生命周期

状态:

- triage
- confirmed
- exploiting
- fixing
- verifying
- closed
- risk_accepted

字段:

- severity
- confidence
- owner
- due_date
- affected_assets
- repro_steps
- remediation
- verification_note

测试:

- finding state machine test
- assign owner test
- due date test

#### VRKB-03. Triage Queue

目标:

- 统一审阅入口

队列:

- unreviewed
- duplicate suspects
- stale findings
- missing evidence

动作:

- accept
- reject
- merge duplicate
- request evidence

测试:

- triage queue filter test
- duplicate merge test

#### VRKB-04. Checklist System

目标:

- 把 section 和 methodology 的完成度显式化

对象:

- section checklist
- methodology checklist
- completion state
- blocker items

测试:

- checklist completion test
- blocker render test

#### VRKB-05. Evidence Blocks

目标:

- 证据成为正式对象

类型:

- screenshot
- request_response
- log_extract
- poc_file
- external_reference

功能:

- attach to finding
- attach to doc
- attach to asset

测试:

- evidence attach test
- evidence reorder test

#### VRKB-06. Assets Integration

目标:

- VRKB 资产不再孤立

能力:

- select existing asset
- upload to asset center
- reverse lookup usage
- link to finding / doc / project

测试:

- asset reuse test
- usage reverse lookup test

#### VRKB-07. Doc Repo

目标:

- 项目文档树支持报告工作流

功能:

- nested docs
- move doc
- doc templates
- report sections
- appendix generation

测试:

- doc move test
- template insert test

#### VRKB-08. Members and Roles

目标:

- 协作权限正式化

角色:

- owner
- lead
- researcher
- observer

能力矩阵:

- create finding
- update finding
- change severity
- manage members
- export report

测试:

- role permission matrix test
- forbidden update test

#### VRKB-09. Audit and Notifications

目标:

- 协作行为可追踪

事件:

- finding created
- finding status changed
- evidence added
- doc updated
- member added

通知:

- assignment
- due soon
- reopened finding

webhook:

- finding changed
- project completed
- new high severity finding

测试:

- audit event test
- webhook payload test

#### VRKB-10. VRKB Portability

目标:

- 项目可迁移

导出:

- project package
- findings
- docs
- evidence links
- asset references
- member map

导入:

- recreate project
- remap evidence/assets
- preserve ids mapping

测试:

- vrkb round-trip test
- evidence link round-trip test

### 10.4 VRKB 验收门槛

- 一个真实项目可以完整经历:
  - 创建
  - triage
  - finding lifecycle
  - 证据收集
  - 文档输出
  - 协作追踪

## 11. Math 详细计划

### 11.1 目标状态

Math 必须成为形式化知识系统，而不是数学主题展示页。

### 11.2 入口文件

- 前端:
  - `frontend/src/components/dashboard/MathDashboard.vue`
  - `frontend/src/components/dashboard/MathDashboardV3.vue`
- 后端:
  - `backend/src/domain/graph_service.rs`
  - `backend/src/interface/api/graph.rs`

### 11.3 工作包

#### MATH-01. Formal Object Model

目标:

- 定义数学对象类型和关系类型

节点类型:

- theorem
- lemma
- definition
- proposition
- corollary
- proof
- example
- problem
- note

关系类型:

- depends_on
- proves
- uses_definition
- generalizes
- special_case_of

测试:

- object schema test
- relation validation test

#### MATH-02. Graph Semantics

目标:

- graph 不再只是树，而是有关系语义

能力:

- relation inspector
- dependency graph
- unresolved prerequisites
- circular dependency detection

测试:

- circular dependency test
- unresolved prerequisites test

#### MATH-03. Workspace Mode

目标:

- 图工作台成为主工作模式之一

功能:

- add node
- add relation
- inspect node
- mark incomplete proof
- highlight blockers

测试:

- relation create test
- blocker highlight test

#### MATH-04. Manuscript / Archive / Workspace

目标:

- 三种工作模式正式化

manuscript:

- 写作视图

archive:

- 阅读归档视图

workspace:

- 推导与关系视图

测试:

- mode switch test
- selected node sync test

#### MATH-05. Formula and References

目标:

- 公式和引用错误可被发现

能力:

- theorem refs
- definition refs
- equation labels
- unresolved refs
- duplicate labels

测试:

- formula render test
- unresolved ref test
- duplicate label test

#### MATH-06. Math Portability

目标:

- 数学知识系统独立可迁移

导出:

- json graph
- markdown manuscript
- latex package

导入:

- graph merge
- label collision handling

测试:

- latex export test
- graph round-trip test

### 11.4 Math 验收门槛

- 用户能定义对象、建立关系、查看依赖、写作、导出

## 12. 测试矩阵总表

### 12.1 平台级

- registry completeness
- renderer normalize
- plugin resolution
- shell state machine
- error boundary
- portability runtime

### 12.2 Assets

- asset schema validation
- upload queue
- usage graph
- picker mode
- permission explanation
- assets portability round-trip

### 12.3 English

- article state machine
- vocab object persistence
- example system
- sentence anchor repair
- english portability round-trip

### 12.4 Memos

- stream actions
- compose/editor handoff
- bulk ops
- review queues
- memos portability round-trip

### 12.5 PRKB

- feed control
- inbox triage
- detail drawer
- query parser
- pdf lifecycle
- bibtex portability round-trip

### 12.6 VRKB

- project overview
- finding lifecycle
- triage queue
- evidence blocks
- role matrix
- vrkb portability round-trip

### 12.7 Math

- object schema
- graph semantics
- formula references
- math portability round-trip

## 13. E2E 清单

### 13.1 Assets E2E

- 打开 Assets
- 上传图片
- 查看 detail
- 复制 asset link
- 在 memo 中插入该 link
- 回到 Assets 查看 usage

### 13.2 English E2E

- 打开 English KB
- 打开文章
- 选词并保存
- 添加例句
- 回到词汇列表查看
- 导出 English KB

### 13.3 Memos E2E

- 快速创建 memo
- 展开 editor
- 添加 tag 和 due date
- 切到 kanban
- 批量归档

### 13.4 PRKB E2E

- 创建 feed
- 抓取 papers
- triage inbox
- 保存到 library
- 添加 note
- 导出 bibtex

### 13.5 VRKB E2E

- 创建 project
- 创建 finding
- 上传 evidence
- 关联 asset
- 导出 report package

### 13.6 Math E2E

- 创建 theorem
- 创建 definition
- 建立 depends_on
- 在 manuscript 引用 definition
- 导出 latex package

## 14. 提交顺序建议

### Batch 1. 平台

- `refactor(protocol): canonicalize renderer ids across kb lifecycle`
- `feat(shell): stabilize self space renderer and dock state machine`
- `feat(testing): add shared fixtures and shell test helpers`

### Batch 2. Assets

- `feat(assets): add typed asset schemas and detail drawer`
- `feat(assets): add upload pipeline and duplicate detection`
- `feat(assets): add usage graph and picker mode`
- `feat(portability): add assets portability provider`

### Batch 3. English

- `refactor(english): split english identities and shell modes`
- `feat(vocabulary): add multi-example and batch vocabulary actions`
- `feat(english): add sentence anchoring repair workflow`
- `feat(portability): add english import pipeline`

### Batch 4. Memos

- `feat(memos): formalize compose and editor handoff`
- `feat(memos): add saved views and bulk operations`
- `feat(memos): add review queues and memo references`
- `feat(portability): add memos provider`

### Batch 5. PRKB

- `feat(prkb): add feed control center and triage workflow`
- `feat(prkb): add library detail drawer and query dsl`
- `feat(prkb): add pdf lifecycle and collections`
- `feat(portability): add prkb provider and bibtex support`

### Batch 6. VRKB

- `feat(vrkb): add finding lifecycle and triage queue`
- `feat(vrkb): add evidence blocks and asset integration`
- `feat(vrkb): add doc repo workflow and report export`
- `feat(vrkb): add audit events and webhook support`

### Batch 7. Math

- `feat(math): add formal object model and relation types`
- `feat(math): add graph workspace and dependency validation`
- `feat(math): add manuscript/workspace sync and references`
- `feat(portability): add math provider`

### Batch 8. 收口

- `feat(obs): add structured audit events and task telemetry`
- `perf(frontend): split heavy special kb bundles`
- `docs(release): add migration and testing runbooks`

## 15. 风险与回避策略

### 风险 1. 协议先收不彻底

结果:

- 后续模块继续基于 alias 开发

回避:

- 每个模块开工前先验证 canonical renderer 是否收口

### 风险 2. 数据模型领先于 UI

结果:

- 后端能力丰富但用户无法使用

回避:

- 每个工作包必须同时定义前端与后端交付

### 风险 3. 功能很多但没有闭环

结果:

- 项目越来越大，但成熟度不升

回避:

- 每个模块必须有主工作流 E2E

### 风险 4. portability 长期被推迟

结果:

- 数据仍被锁在系统里

回避:

- 每个模块在进入“完成”前必须补 portability provider

## 16. 发布完成判定

只有同时满足以下条件，才可以对外宣称特殊知识库体系进入成熟阶段:

1. 六个特殊 KB 都拥有正式对象模型。
2. 六个特殊 KB 都拥有专项 portability provider。
3. 六个特殊 KB 都拥有至少 1 条 E2E 主工作流。
4. 统一 Shell、权限、审计、长任务和错误处理已形成共享底座。
5. `cargo test`、`npm run test:unit`、`npm test`、`npm run build` 持续稳定通过。

## 17. 当前建议的下一执行项

按依赖和收益排序，下一步直接执行:

1. `PLAT-01`
2. `ASSET-01`
3. `ASSET-02`
4. `ASSET-03`
5. `ASSET-04`

理由:

- 资产底座越早完成，越能减少 Memos / PRKB / VRKB 后续返工。
