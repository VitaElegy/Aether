# Aether 特殊知识库世界级功能 Backlog

状态: 提案
日期: 2026-03-19
作者: Codex

## 1. 文档目标

这份文档不是阶段标题，也不是抽象愿景，而是把特殊知识库后续建设拆到功能层、交互层、接口层和测试层，作为后续真正实施的总 backlog。

判定标准只有一个: 各个特殊知识库必须从“有原型”升级为“协议统一、能力闭环、导入导出完整、可测试、可演进”的成熟系统。

## 2. 世界级完成定义

一个特殊知识库只有同时满足下列条件，才可以被认为接近世界级:

1. 身份唯一:
   - `renderer_id`
   - `plugin.id`
   - `layout.id`
   - `dashboard.id`
   - `portability.provider_id`
   - `route namespace`
   - `test fixture id`
   必须一一对应，不能继续依赖 alias 和 fallback 兜底。
2. 功能闭环:
   - 创建 / 发现
   - 列表 / 检索
   - 编辑 / 操作
   - 专项视图
   - 导入 / 导出
   - 权限 / 引用
   - 历史 / 审计
   - 测试
3. 质量闭环:
   - 单元测试
   - 集成测试
   - E2E happy path
   - round-trip portability 测试
4. 产品闭环:
   - 错误可解释
   - 长任务有进度
   - 数据可迁移
   - 模块间协同稳定
   - 性能与权限可观测

## 3. 总执行顺序

### Phase 0. 协议层与质量底座

目标: 清掉当前所有特殊知识库的最大技术债。

#### 0.1 KB Identity Registry

功能项:

- 建立统一 `special_kb_registry`:
  - 每个 KB 定义 `renderer_id`
  - 绑定唯一 `plugin`
  - 绑定唯一 `layout`
  - 绑定唯一 `portability provider`
  - 绑定唯一 `shell behavior`
  - 绑定唯一 capability set
- 启动时执行 registry 完整性校验:
  - 缺 plugin 时报错
  - 缺 layout 时报错
  - 缺 portability provider 时报错
  - 发现重复 `renderer_id` 时报错
- 把当前散落在 `main.ts` 的别名注册迁移到单点映射表
- 把 `useSelfSpaceOrchestrator.ts` 中 legacy 解析改成:
  - 只接受 registry 中存在的 renderer
  - 对历史 renderer id 提供显式 migration table
  - migration 完成后记录 telemetry

后端改造:

- 新建 KB 协议描述结构:
  - `renderer_id`
  - `provider_id`
  - `export_formats`
  - `import_formats`
  - `singleton`
  - `supports_assets`
  - `supports_audit_log`
- 在 bootstrap 阶段校验 provider 是否完整注册

前端改造:

- 新建 registry 源文件，替换 `Object.assign({}, Plugin, { id: 'alias' })`
- `resolvePlugin` 改成:
  - 严格模式
  - migration 模式
  - loud failure 模式

测试:

- registry completeness test
- duplicate renderer test
- old renderer migration test
- shell 启动失败 test

验收:

- 删除主要 alias 后系统仍能正常启动
- 任一 KB 都能从一个唯一 renderer id 启动

#### 0.2 Self Space Shell 收敛

功能项:

- 区分 `singleton app` 和 `multi-instance app`
- 完善 running / pinned / active / minimized / crashed 状态机
- header action 注入协议:
  - title
  - primary action
  - context action
  - progress badge
- deep link 规则统一:
  - `space/:kbId`
  - `space/:kbId/:subview`
  - `space/:kbId/:subview/:entityId`
- session restore:
  - 刷新后恢复最近运行实例
  - 恢复 active tab / filter / scroll intent
- error boundary:
  - 单个 app 崩溃不拖垮 Shell
  - 崩溃实例可重启

测试:

- singleton 去重 test
- deep link reopen test
- crash isolation test
- session restore test

验收:

- 任何特殊 KB 切换时不发生路由抖动
- 崩溃 app 被替换为错误面板，其他 app 正常运行

#### 0.3 测试与 CI 基线

功能项:

- 后端 fixture factory:
  - user
  - kb
  - article
  - asset
  - paper
  - finding
- 前端 test harness:
  - auth mock
  - store reset helper
  - API mock router
  - shell launcher helper
- Playwright smoke matrix:
  - Shell 启动
  - Assets 上传
  - PRKB feed 创建
  - Memos 创建
  - Vocabulary 保存
- portability round-trip harness:
  - export -> import -> compare

验收:

- `cargo test`
- `npm run test:unit`
- `npm run test:e2e`
- `npm run build`
  全绿

### Phase 1. Portability 2.0

目标: 把“有导入导出框架”升级为“每个特殊 KB 有专项导入导出协议”。

#### 1.1 统一 Export Manifest

功能项:

- 导出清单预览:
  - 记录条目数量
  - 记录资源数量
  - 记录依赖模块
  - 记录格式版本
- 支持格式:
  - `json`
  - `markdown bundle`
  - `binary bundle`
- 导出前分析:
  - 是否含有二进制资源
  - 是否含有跨 KB 引用
  - 是否含有不可导出字段

#### 1.2 Import Workflow

功能项:

- 导入策略:
  - create new
  - merge into existing
  - replace target
- 冲突预览:
  - title 冲突
  - slug 冲突
  - external id 冲突
  - asset hash 冲突
- 导入任务进度:
  - prepare
  - validate
  - extract
  - transform
  - write
  - finalize
- 失败回滚:
  - 部分失败时写入错误报告
  - 可重试未完成部分

#### 1.3 Provider 专项化

功能项:

- English provider:
  - 文章
  - 词汇
  - 例句
  - sentence anchor
- Memos provider:
  - memos
  - tag config
  - view prefs
  - channel config
- PRKB provider:
  - feeds
  - inbox items
  - library papers
  - notes
  - local pdf refs
- VRKB provider:
  - projects
  - findings
  - docs
  - assets
  - members
  - specs
- Math provider:
  - theorem graph
  - manuscript
  - proof nodes
  - references
- Assets provider:
  - metadata
  - binaries
  - usage edges
  - permission hints

测试:

- 每种 KB 1 组 export preview test
- 每种 KB 1 组 import merge test
- 每种 KB 1 组 round-trip test

验收:

- 所有特殊 KB 都可独立导出
- 所有特殊 KB 都可重新导入且不丢核心结构

## 4. 各特殊知识库功能级建设

### 4.1 Assets KB

当前定位问题:

- 现状更像上传页，不是资源中枢
- 缺类型化资产模型
- 缺 usage graph
- 缺 picker mode
- 缺上下文访问解释

#### 4.1.1 资产类型系统

功能项:

- 把资产拆成正式类型:
  - `file_asset`
  - `image_asset`
  - `pdf_asset`
  - `domain_asset`
  - `ip_asset`
  - `credential_stub`
  - `snippet_asset`
- 每种类型定义专属 metadata schema:
  - image: width / height / dominant colors / exif summary
  - pdf: page_count / text_extract_status / source_url
  - domain: fqdn / registrar / tags / risk_note
  - ip: cidr / provider / environment / exposure_note
  - credential_stub: label / scope / rotation_due_at / linked_assets
- 后端提供 schema validator
- 前端 detail drawer 根据类型切换字段区块

测试:

- schema validation test
- asset type rendering test
- unsupported type rejection test

验收:

- 上传或创建任一资产后，系统能显示结构化字段而不是只显示文件名

#### 4.1.2 资源接入能力

功能项:

- 上传方式:
  - 本地拖拽上传
  - 文件选择上传
  - 粘贴截图上传
  - URL 导入
- 上传阶段反馈:
  - hashing
  - uploading
  - analyzing
  - complete
- 失败处理:
  - 文件过大
  - mime 不支持
  - hash 冲突
  - 存储失败
- 去重策略:
  - 内容 hash 去重
  - 已存在则提示:
    - 复用现有资源
    - 强制新版本

测试:

- upload progress test
- duplicate hash test
- url import test

验收:

- 用户能从桌面拖拽、剪贴板和 URL 三种路径进入资产库

#### 4.1.3 资产工作台

功能项:

- 视图模式:
  - Grid
  - Table
  - Detail
  - Usage
- 过滤器:
  - type
  - tag
  - used / unused
  - owner
  - created_at
  - linked KB
- 排序:
  - newest
  - largest
  - most used
  - recently referenced
- 批量动作:
  - tag
  - move to archive
  - export selection
  - copy asset link

测试:

- grid/table toggle test
- filter and sort test
- bulk action test

验收:

- Assets 页面不再只是静态缩略图墙，而是可检索的资产控制台

#### 4.1.4 Usage Graph 与 Picker Mode

功能项:

- usage graph:
  - 记录资产被哪些文章、memo、finding、paper note 引用
  - 支持反查 `Used In`
  - 支持点击跳转到引用实体
- picker mode:
  - markdown editor 可打开资产选择器
  - VRKB finding 可插入资产
  - PRKB note 可附加本地 PDF / image
  - Memos compose bar 可插入 asset chip
- 统一链接语法:
  - `[[asset:uuid]]`
  - `![caption]([[asset:uuid]])`

测试:

- usage edge creation test
- usage tab render test
- picker selection test
- markdown link round-trip test

验收:

- 任一模块插入资产后，Assets 中可以反查引用位置

#### 4.1.5 权限与审计

功能项:

- contextual access:
  - 直接所有者可读
  - 非所有者必须通过 context article / finding / doc 引用访问
- 权限解释 UI:
  - why can view
  - why denied
  - referenced by which context
- 审计:
  - upload
  - replace
  - delete
  - share via context

测试:

- owner access test
- context access test
- denied access test
- audit log creation test

验收:

- 任一用户访问私有资产时，系统可以解释访问原因，而不是只返回 403

### 4.2 English / Vocabulary KB

当前定位问题:

- 当前已经是最强模块，但 English 还没成为真正独立协议
- 词汇、文章分析、例句、句子锚点之间仍然存在边界不清的问题

#### 4.2.1 English App Identity 收敛

功能项:

- 把 English 拆成正式 app identity:
  - `english`
  - `vocabulary`
  - `article-analysis`
  三者的关系由 capability 描述，不再靠 alias
- Shell 层支持同 KB 多工作模式:
  - reading mode
  - vocabulary mode
  - article analysis mode
- routing:
  - `space/:kbId/articles`
  - `space/:kbId/vocabulary`
  - `space/:kbId/article/:articleId`

测试:

- renderer identity test
- route resolution test
- tab restore test

验收:

- English 相关功能不再依赖 `english_v1_std` 等兼容 id

#### 4.2.2 文章阅读与分析工作流

功能项:

- article inbox / library:
  - 最近阅读
  - 待读
  - 已分析
  - 已归档
- 阅读器能力:
  - 句子 hover
  - 单词选择
  - 上下文高亮
  - 原文 / 翻译切换
  - 段落折叠
- 文章分析状态:
  - pending
  - analyzing
  - analyzed
  - failed
- 文章级操作:
  - 重新分析
  - 导出分析结果
  - 生成词汇候选清单

测试:

- article status transition test
- sentence selection test
- reanalyze command test

验收:

- 用户从一篇文章中可以完整完成“阅读 -> 选词 -> 保存 -> 回看上下文”

#### 4.2.3 词汇库与例句系统

功能项:

- 词汇实体补全字段:
  - lemma
  - root
  - phonetic
  - level
  - tags
  - mastery status
  - query_count
  - importance
- 例句系统:
  - 一个词可挂多个例句
  - 例句可以关联 article id
  - 例句可以关联 `global_sentence_id`
  - 可标记主例句
- 词汇视图:
  - all
  - recent
  - starred
  - difficult
  - no examples
- 词汇操作:
  - 批量加标签
  - 批量标记重要
  - 合并重复词
  - 恢复误删词

测试:

- multi-example persistence test
- merge duplicate words test
- batch important toggle test

验收:

- 一个词拥有多个上下文例句，且能稳定回到来源文章

#### 4.2.4 Sentence Anchoring 2.0

功能项:

- sentence parser 输出:
  - source sentence hash
  - normalized hash
  - global sentence id
  - article-local sentence id
  - fuzzy match diagnostics
- 文章更新后执行 anchor repair:
  - exact
  - normalized
  - fuzzy
  - unresolved
- unresolved queue:
  - 展示失配例句
  - 支持人工重新绑定

测试:

- typo edit anchor stability test
- sentence moved anchor repair test
- unresolved repair flow test

验收:

- 轻微改文不会导致例句全部丢锚

#### 4.2.5 Vocabulary Intelligence

功能项:

- query pipeline:
  - dictionary lookup
  - lemma normalize
  - inflection resolve
  - local match merge
- 展示层:
  - multiple definitions
  - word family
  - phrases
  - collocations
  - related words
- 学习层:
  - important queue
  - recently queried queue
  - no example queue
  - custom collections

测试:

- fuzzy suggestion test
- lemma normalize test
- local/remote merge test

验收:

- 词汇搜索结果不只是单次查询结果，而是可沉淀的知识对象

#### 4.2.6 English Portability

功能项:

- 导出:
  - vocabulary csv
  - vocabulary json
  - article analysis json
  - markdown notebook bundle
- 导入:
  - merge by lemma
  - merge by word + sentence anchor
  - import conflict preview
- 与 Anki 类导入格式兼容:
  - word
  - meaning
  - example
  - note

测试:

- csv export test
- import merge test
- article + vocab round-trip test

验收:

- English KB 可以独立迁移，不依赖整库备份

### 4.3 Memos KB

当前定位问题:

- 组件数量已经够多，但系统闭环不够强
- 目前更像多个 memo 视图的集合，而不是统一的碎片工作流

#### 4.3.1 Stream 主工作台

功能项:

- stream 卡片信息密度统一:
  - title
  - excerpt
  - tags
  - channel
  - status
  - updated_at
  - linked entities
- stream 操作:
  - quick archive
  - pin
  - snooze
  - convert to note
  - convert to task
- stream 分组:
  - today
  - yesterday
  - this week
  - pinned

测试:

- stream grouping test
- stream quick action test

验收:

- 用户在 stream 中可以完成大多数日常整理动作，不必频繁进入大编辑器

#### 4.3.2 Compose Bar 与 Editor 双模

功能项:

- compose bar:
  - instant text capture
  - slash command
  - quick tag
  - attach asset
  - paste URL card
- full editor:
  - markdown edit
  - checklist
  - linked reference picker
  - reminder / due date
  - channel assignment
- 模式切换:
  - quick capture -> expand editor
  - editor save 后回流 stream

测试:

- compose quick create test
- expand editor state handoff test
- asset chip insert test

验收:

- 从输入到保存的路径可以覆盖“1 秒速记”和“深度整理”两种场景

#### 4.3.3 Smart Dock 与 Saved Views

功能项:

- 左侧 Smart Dock:
  - inbox
  - saved views
  - pinned tags
  - channels
  - stale memos
  - no-tag memos
- saved views:
  - 保存 filter 条件
  - 保存排序方式
  - 保存视图模式
- 一键切换:
  - stream
  - masonry
  - kanban
  - timeline
  - calendar

测试:

- saved view persistence test
- pinned tag toggle test
- dock filter test

验收:

- 常用过滤不再靠手动重复点击，而是可以持久保存

#### 4.3.4 组织与批处理

功能项:

- 批量操作:
  - tag
  - channel
  - archive
  - delete
  - status change
- memo 合并与拆分:
  - 合并多个碎片
  - 从一条 memo 拆出子 memo
- 引用网络:
  - 引用 article
  - 引用 paper
  - 引用 finding
  - 引用 asset
- 反链面板:
  - show linked by
  - show mentioned in

测试:

- bulk update api test
- memo merge test
- backlinks render test

验收:

- Memos 能从“收集碎片”升级为“整理知识入口”

#### 4.3.5 日程与节律

功能项:

- 时间属性:
  - created_at
  - updated_at
  - scheduled_at
  - due_at
  - snoozed_until
- calendar 能力:
  - date drop
  - reschedule
  - overdue highlight
- review 队列:
  - stale memos
  - unresolved memos
  - due today

测试:

- calendar move test
- stale queue test
- overdue highlight test

验收:

- Memo 不再是纯静态卡片，而是能进入回顾和计划流程

#### 4.3.6 Memos Portability

功能项:

- 导出:
  - markdown notebook
  - json bundle
  - daily archive bundle
- 导入:
  - tag merge
  - channel merge
  - duplicate title + body detection

测试:

- markdown export test
- duplicate detection import test

验收:

- 用户可以独立迁移 memo 体系，不破坏标签和视图配置

### 4.4 PRKB

当前定位问题:

- 后端 schema 和抓取方向已经开始成熟
- 前端主工作台仍偏薄
- 研究工作流还没闭环

#### 4.4.1 Feed 管理中心

功能项:

- feed 列表字段补全:
  - type
  - source url
  - health status
  - last fetched
  - total fetched
  - parse errors
- feed 操作:
  - enable / disable
  - manual fetch
  - fetch selected
  - backfill last 30 days
  - test parser
- preset marketplace:
  - conference rss
  - blog rss
  - arxiv category
  - custom atom

测试:

- create/delete feed test
- selected feed refresh test
- parser error render test

验收:

- Feed 不是只存一条 URL，而是可诊断、可维护、可选择性刷新

#### 4.4.2 Inbox Triage 工作流

功能项:

- inbox 状态:
  - new
  - read
  - saved
  - skipped
  - trashed
- 批量 triage:
  - save selected
  - mark read
  - trash
  - add to collection
- 快速操作:
  - one-key save
  - one-key trash
  - mark priority
  - add note
- triage sidebar:
  - unread only
  - by source
  - by publication
  - by recency

测试:

- inbox state transition test
- bulk triage test
- unread filter test

验收:

- 用户能高效处理研究流入，不必一篇篇手动操作

#### 4.4.3 Library 与 Rich Detail

功能项:

- library rich card:
  - title
  - abstract excerpt
  - authors
  - venue
  - year
  - tags
  - signals
  - read state
  - note count
- detail drawer:
  - full abstract
  - source links
  - author list
  - publication info
  - user note
  - local pdf status
  - related papers
- 操作:
  - attach PDF
  - open source
  - copy citation
  - export bib entry

测试:

- library detail render test
- pdf attach test
- citation copy test

验收:

- 保存后的 paper 不只是卡片，而是研究对象详情页

#### 4.4.4 Search / Facet / Query DSL

功能项:

- facet:
  - venue
  - publication
  - author
  - year
  - source type
  - state
  - has pdf
  - tagged
- query syntax:
  - `author:`
  - `venue:`
  - `year:`
  - `tag:`
  - `state:`
  - free text
- saved searches:
  - 保存条件
  - 保存排序
  - 保存显示列

测试:

- facet combination test
- query parser test
- saved search persistence test

验收:

- PRKB 能像专业研究台一样快速收敛结果，而不是只能粗放翻卡片

#### 4.4.5 研究资产与信号系统

功能项:

- signal pipeline:
  - feed freshness
  - venue score
  - author recurrence
  - citation placeholder
  - custom importance
- paper collection:
  - watchlist
  - reading queue
  - archive
  - topic collections
- local PDF lifecycle:
  - not attached
  - queued
  - downloaded
  - indexed
  - failed

测试:

- signal refresh test
- collection move test
- pdf lifecycle test

验收:

- PRKB 不只是信息抓取，而是可组织、可排序、可深读

#### 4.4.6 Export / Portability

功能项:

- 导出:
  - bibtex
  - json
  - markdown research digest
  - collection export
- 导入:
  - bibtex import
  - paper dedupe by external_id / doi / title
  - merge tags and notes

测试:

- bibtex export test
- bibtex import dedupe test
- collection round-trip test

验收:

- 用户能把自己的研究资料带进带出，而不是被锁在系统里

### 4.5 VRKB

当前定位问题:

- 已经有项目、Finding、Doc、Asset、Member、Spec 的骨架
- 但还不是成熟的安全研究协作平台

#### 4.5.1 Project Control Center

功能项:

- project overview:
  - scope summary
  - status summary
  - open findings
  - severity distribution
  - linked assets
  - checklist completion
- project metadata:
  - client
  - target
  - start/end dates
  - methodology
  - repo links
  - environment tags
- project templates:
  - web app
  - mobile app
  - infra
  - protocol

测试:

- create project test
- overview stats test
- template default data test

验收:

- 新项目创建后会立刻获得结构化工作台，而不是空白容器

#### 4.5.2 Finding 生命周期

功能项:

- finding 状态机:
  - triage
  - confirmed
  - exploiting
  - fixing
  - verifying
  - closed
  - risk accepted
- finding 字段:
  - severity
  - confidence
  - affected assets
  - owner
  - due date
  - reproduction steps
  - remediation
- finding 操作:
  - move across status
  - assign owner
  - promote from triage
  - clone finding
  - archive false positive

测试:

- finding state machine test
- assign owner test
- false positive archive test

验收:

- VRKB 里的 finding 能完整经历从 triage 到关闭的专业生命周期

#### 4.5.3 Triage Queue 与 Checklists

功能项:

- triage queue:
  - unreviewed items
  - duplicate suspects
  - stale findings
  - missing evidence
- checklist system:
  - per section checklist
  - per methodology checklist
  - completion percent
  - blocking items
- queue 动作:
  - accept
  - reject
  - merge duplicate
  - request more evidence

测试:

- triage queue filter test
- checklist completion test
- duplicate merge test

验收:

- 项目负责人可以从统一队列审阅发现，而不是散落在多个页面手动追踪

#### 4.5.4 Evidence 与 Asset 联动

功能项:

- evidence block:
  - screenshot
  - request/response
  - log extract
  - poc file
  - external reference
- evidence 绑定:
  - finding
  - doc section
  - project asset
- 与 Assets 集成:
  - 直接选择已有 asset
  - 上传并自动归档到 Assets
  - 反查 evidence 使用位置

测试:

- evidence attach test
- asset reuse test
- usage reverse lookup test

验收:

- VRKB 里的证据不再是孤立文件，而是系统级可追踪资源

#### 4.5.5 Doc Repo 与报告流

功能项:

- 文档树:
  - root docs
  - nested sections
  - move doc
  - doc template
- 报告工作流:
  - executive summary
  - methodology
  - findings appendix
  - asset appendix
- 导出:
  - markdown bundle
  - html preview
  - json report package

测试:

- doc move test
- report export test
- section template test

验收:

- VRKB 能生成稳定的项目文档输出，而不是只有编辑器原始内容

#### 4.5.6 协作、通知与审计

功能项:

- 成员角色:
  - owner
  - lead
  - researcher
  - observer
- 审计日志:
  - finding changed
  - member added
  - doc updated
  - asset linked
- 通知:
  - assignment
  - due soon
  - finding reopened
- webhook:
  - finding changed
  - project completed
  - new high severity finding

测试:

- role permission test
- audit log test
- webhook payload test

验收:

- VRKB 的协作行为可追踪、可回放、可联动外部系统

### 4.6 Math KB

当前定位问题:

- 现在主要是 dashboard 和阅读外观
- 距离数学知识系统还差对象模型和操作闭环

#### 4.6.1 数学对象模型

功能项:

- 正式节点类型:
  - theorem
  - lemma
  - definition
  - proposition
  - corollary
  - proof
  - example
  - problem
  - note
- 关系类型:
  - depends_on
  - proves
  - uses_definition
  - generalizes
  - special_case_of
- 后端校验:
  - proof 需要 target
  - theorem 可挂多个 proof
  - circular dependency 检测

测试:

- node schema test
- relation validation test
- circular dependency test

验收:

- Math KB 的内容不再只是文档块，而是形式化对象网络

#### 4.6.2 Graph Workspace

功能项:

- graph 视图:
  - dependency graph
  - local neighborhood
  - unresolved prerequisites
- graph 操作:
  - add node
  - link node
  - inspect relation
  - mark proof incomplete
- graph 辅助:
  - highlight blockers
  - collapse branches
  - search node by symbol

测试:

- graph edge creation test
- blocker highlight test
- node search test

验收:

- 用户能把定理之间的依赖关系直接作为主工作界面操作

#### 4.6.3 Manuscript / Archive / Workspace 三视图

功能项:

- manuscript:
  - 面向写作
  - 文档顺序
  - 节结构
- archive:
  - 面向阅读
  - 历史版本
  - published snapshot
- workspace:
  - 面向推导
  - graph + editor 联动

测试:

- view switch test
- selected node sync test
- snapshot restore test

验收:

- Math KB 同时支持写作、阅读、推导三种工作模式

#### 4.6.4 LaTeX 与公式引用能力

功能项:

- 公式块:
  - inline
  - block
  - aligned
- 引用:
  - theorem ref
  - definition ref
  - equation ref
- 校验:
  - unresolved reference
  - duplicate label
  - missing symbol definition

测试:

- formula render test
- unresolved ref test
- duplicate label test

验收:

- 数学文本的引用错误在编辑时即可暴露

#### 4.6.5 Math Portability

功能项:

- 导出:
  - json graph
  - markdown manuscript
  - latex package
- 导入:
  - graph merge
  - label collision handling

测试:

- latex export test
- graph round-trip test

验收:

- Math KB 可以作为独立知识系统迁移

## 5. 横向平台功能

### 5.1 权限模型升级

功能项:

- 统一 capability ACL:
  - read
  - write
  - manage
  - export
  - share
- context-derived permission:
  - asset via doc
  - asset via finding
  - paper via collection
- 权限解释 API:
  - 返回 allow/deny 理由
  - 返回上下文链路

测试:

- cross-kb permission inheritance test
- permission explanation test

### 5.2 审计与观测

功能项:

- structured event log:
  - actor
  - target
  - action
  - context
  - result
- long task tracing:
  - import
  - export
  - fetch
  - analyze
  - upload
- UI telemetry:
  - shell crash
  - parser failure
  - unresolved import conflict

测试:

- event log schema test
- progress event sequence test

### 5.3 性能工程

功能项:

- 分块:
  - English analyzer chunk
  - VRKB workspace chunk
  - PRKB detail chunk
  - Math workspace chunk
- 数据懒加载:
  - details on demand
  - facets on demand
  - evidence on demand
- 虚拟列表:
  - memos stream
  - paper list
  - assets table

测试:

- lazy load regression test
- large list render test

## 6. 功能级实施顺序

### Sprint A. 协议和测试

交付:

- special_kb_registry
- shell state machine
- strict plugin resolution
- portability test harness
- CI 全绿

### Sprint B. Assets Foundation

交付:

- typed asset schemas
- asset detail drawer
- upload pipeline
- usage tab
- picker mode

### Sprint C. English Stabilization

交付:

- english identity 收敛
- sentence anchoring 2.0
- multi-example vocab system
- english portability

### Sprint D. Memos Workflow

交付:

- stream 操作强化
- compose/editor 双模
- saved views
- bulk operations
- memo portability

### Sprint E. PRKB Professionalization

交付:

- feed control center
- inbox triage
- rich library detail
- facet + query dsl
- bibtex portability

### Sprint F. VRKB Collaboration

交付:

- finding lifecycle
- triage queue
- evidence blocks
- report export
- audit + webhook

### Sprint G. Math Formalization

交付:

- formal math object model
- dependency graph workspace
- manuscript/archive/workspace modes
- latex export

### Sprint H. Polishing

交付:

- performance pass
- observability pass
- docs and release pass

## 7. 代码量与工作量预估

这是结构化增量，不是凑行数。

预估新增代码量:

- 协议层和 shell: 1.5k - 2.2k
- Portability 2.0: 1.8k - 2.6k
- Assets KB: 1.8k - 2.4k
- English / Vocabulary: 1.4k - 2.0k
- Memos: 1.3k - 1.9k
- PRKB: 1.6k - 2.3k
- VRKB: 1.9k - 2.7k
- Math: 1.3k - 1.9k
- 测试增量: 2.5k - 3.8k

总计:

- 13.1k - 19.8k 行

## 8. 完成标准

只有满足以下结果，才算这轮成熟化结束:

1. 所有特殊 KB 拥有唯一协议身份。
2. 所有特殊 KB 拥有专项 portability provider。
3. 所有特殊 KB 至少拥有 1 条完整 E2E 路径。
4. 所有特殊 KB 至少拥有 1 组 round-trip 导入导出测试。
5. Assets、English、Memos、PRKB、VRKB、Math 的主工作流都形成闭环。
6. Shell、权限、审计、性能和错误观测形成系统级支撑。

如果以上任一项没有完成，就还不能叫“完善”，更不能叫“世界级”。
