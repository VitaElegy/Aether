# 讨论：Self Space 便签 (Memos) 知识库需求
日期: 2026-01-25
主题: 深入讨论 Self Space "便签" (Memos) 功能的具体需求。

## 背景
用户希望实现 Self Space 的 "Memos" 模块。这对应路线图中的 "Phase 3"。
我们将进行 10 个问题的访谈来明确需求。

## 问答记录

### 问题 1: 核心视觉隐喻 (Core Metaphor)
**提问**: 我们正在设计 **Memos (便签)** 模块。
**用户打开这个模块时，首选的"核心视图" (Hero View) 应该是什么样的？**
请描述您理想中的默认视图以及它给用户的*感觉*。

**选项参考**:
*   **时间轴/日志 (Timeline/Journal)**: 类似日记或朋友圈的时间流？
*   **看板 (Kanban)**: 任务导向（待办 vs 已办）？
*   **空间网格 (Spatial Grid)**: 二维的便利贴墙（类似 Apple Freeform）？
*   **列表/收件箱 (List/Inbox)**: 高密度的条目列表（类似 Apple Notes/Outlook）？

**回答**: 用户希望 **全部支持**。四个视图（时间轴、看板、空间网格、列表）都要有，并且可以合理切换。

### 问题 2: 容器与组织结构 (Container & Structure)
**提问**: 既然我们要支持"空间网格"和"看板"，这涉及到了数据的组织方式。请问便签的**归属关系**是怎样的？

*   **模式 A（全局池 - 类似 Apple Notes/Google Keep）**: 所有便签都在一个大池子里。
    *   "网格视图"只是对这些便签的一种自动可视化排列（或者只记录一次性的位置）。
    *   没有"文件夹"或"画板"的概念，只有标签 (Tags) 来筛选。
*   **模式 B（画板容器 - 类似 Trello/Miro）**: 用户必须先创建一个**"画板" (Board)** 或 **"页面"**，然后在里面创建便签。
    *   每个便签属于特定的画板。
    *   空间位置是画板的属性。
    *   不同画板里的便签是隔离的。

    *   不同画板里的便签是隔离的。

您更倾向于哪种？（我们的架构需要决定是 `Memo belongs_to Board` 还是 `Memo belongs_to User`）

**回答**: 用户选择 **模式 B (画板)**，但特别强调**便签可以自由迁移/移动到其他画板**。

### 问题 3: 编辑体验 (Editor Experience)
**提问**: 便签需要既"轻量"又"结构化"。
1.  **交互模式**: 当我点击一张便签卡片时，它应该如何打开？
    *   **弹窗 (Modal)**: 在当前页面上层浮现一个对话框。（保持上下文，最常见）
    *   **侧边栏 (Side Drawer)**: 从右侧滑出。（适合内容较长，可以参照其他便签写东西）
    *   **原位展开 (Inline Expand)**: 卡片变大，直接在板上编辑。（最流畅，但不仅适合复杂内容）
2.  **内容丰富度**: 便签的内容支持程度应该是？
    *   **全功能 (Full Article)**: 和文章一样，支持 Markdown、代码块、数学公式、图片上传。（开发成本低，复用现有编辑器）
    *   **轻量级 (Lightweight)**: 仅支持纯文本 + 简单的清单/加粗。（启动更快，更纯粹）


**回答**: 
1.  **交互**: **原位展开 (Inline Expand)** - 类似 Notion 看板，直接变大编辑。
2.  **内容**: **全功能 (Full)** - 必须支持 Markdown/代码/Block，但**感觉要轻量**。
    *   *User Note*: 明确引用了 "Block-first" 设计 (参考 `AI/specs/kb_parser_interface.md`)。
    *   *User Note*: 强调要"特别方便的记录" (Convenient Recording)。

### 问题 4: 分类与检索 (Taxonomy & Retrieval)
**提问**: 为了实现"特别方便"的查找和整理，除了"画板"这个物理容器外，您需要哪些维度的分类？

*   **视觉编码 (Color Coding)**: 像传统便利贴一样，通过**颜色**来区分（红=紧急，黄=日常）？这是最快、最直观的。
*   **语义标签 (Tags)**: 像 Evernote/Notion 一样，打 **#标签**？（更灵活，但输入有阻力）
*   **双向链接 (Bi-directional Links)**: 既然基于 Block-first，是否支持在便签里写 `[[引用]]` 其他便签？

**核心矛盾**: "方便记录"通常意味着减少分类负担。您希望默认是**基于颜色**的简单分类，还是**基于标签**的精密分类？

**回答**: 默认只用颜色 (Color-First)，偶尔用标签 (Tags-Secondary)。
*   这确认了我们需要一个显眼的颜色选择器 (Color Picker) 作为一级交互。

### 问题 5: 卡片视觉密度 (Visual Density)
**提问**: 在 **看板 (Board)** 和 **网格 (Grid)** 视图中，未展开的便签卡片应该显示多少内容？

*   **极简模式 (Minimal)**: 只显示**标题** (如果有) 和 **颜色**。正文完全隐藏，必须点击展开才能看。
    *   *优点*: 极其整洁，适合以此作为"目录"。
    *   *缺点*: 必须点击才能知道里面写了什么。
*   **摘要模式 (Snippet)**: 显示标题 + **前 3 行正文** + 如果有图片显示略缩图。
    *   *优点*: 信息即时可见，不需要频繁点击。
    *   *缺点*: 界面会显得比较杂乱 (Information Overload)。
*   **自适应模式 (Auto-Adaptive)**: 
    *   字少的便签：直接显示**全部全文**（所见即所得）。
    *   字多的便签：截断显示前几行。

    *   字多的便签：截断显示前几行。

您希望默认的视觉密度是哪一种？

**回答**: **摘要模式 (Rich Preview)**。
*   显示：标题 + 前3-5行 + 第一张图缩略图。
*   感觉：像 **Pinterest** 瀑布流。

### 问题 6: 与其他模块的集成 (Integration)
**提问**: 便签通常是"想法的种子"。当种子长大后，应该发生什么？

1.  **升级路径 (Promotion)**:
    *   **一键升级**: 支持将便签"升级"为正式的 **Article** (文章知识库)？
    *   *逻辑*: 升级后，原便签是消失（转化），还是保留一个链接指向新文章？
2.  **反向链接 (Backlinks)**:
    *   在写正式文章 (Article) 时，是否需要一个侧边栏能**把便签拖进来**作为素材？

简而言之：便签是**完全独立**的小世界，还是长文章的**原材料工厂**？

**回答**: **原材料工厂**。
*   **反向引用**: 支持写文章时把便签**拖拽**进去作为素材。
*   **升级逻辑**: 支持升级为文章，但**保留**原便签，并将其变身为一个指向新文章的**引用/链接**（不删除历史）。

### 问题 7: 快速录入 (Quick Capture)
**提问**: 便签的核心在于"快"。您希望如何录入新便签？

*   **常规模式 (Standard)**: 必须打开 App，进入画板，点击 "+" 号。
*   **全局浮窗 (Global Panel)**: 类似 **Apple Quick Note** 或 **Raycast**。无论在哪（即使在看其他文章），按一个快捷键（如 `Cmd+Shift+M`）就能弹出一个小框快速输入？
*   **引用录入 (Selection to Memo)**: 在阅读文章时，选中一段话 -> 右键 -> "保存到便签"？

**您希望我们做到多"快"？**（全局浮窗开发成本较高，但体验最好）

**回答**: **全功能上下文录入**。
*   用户强调需要**"强大的上下文能力"**。
*   这意味着我们需要实现：
    1.  **全局快捷键浮窗** (Global Quick Note).
    2.  **阅读选区菜单** (Text Selection -> Save to Memo).

### 问题 8: 智能增强 (Intelligence)
**提问**: 便签积累多了容易乱。为了保持"方便"，您是否需要 **AI** 介入来辅助整理？

*   **零智能 (Manual)**: 完全手动，保持纯粹极速。
*   **被动辅助 (Passive)** - *推荐*: 
    *   录入时，AI 自动**分析语义**并**推荐标签** (Auto-Tagging)？
    *   在"摘要视图"中，显示的不是截取的前三行，而是 AI 生成的**一句话总结**？
*   **主动连接 (Active Recall)**: 
    *   "写这张便签时，系统提示：'这也和之前的便签 X 有关'..." (需要向量数据库支持)

您希望 AI 在便签模块中扮演多重的角色？

**回答**: **零智能 (Manual)**，但**预留接口**。
*   目前保持纯粹、极速。
*   架构上需要预留 `analysis_status` 或 `ai_tags` 字段，方便未来接入。

### 问题 9: 迁移与导入 (Migration & Import)
**提问**: 用户通常已经积累了大量碎片化信息。您最希望支持从哪里导入数据？

*   **纯文本/Markdown**: 比如粘贴一大段文本，自动按空行分割成多个便签？
*   **专属格式**: 是否需要专门针对 **Flomo (浮墨)**、**Apple Notes** 或 **Notion** 的导入器？
*   **不导入**: 这是一个全新的开始，不希望被旧数据污染？

(如果我们支持"粘贴即导入"，会让新用户更容易上手)

**回答**: **全部支持**。
*   **Smart Paste (Priority High)**: 必须支持粘贴一大段文本，自动按空行分割生成便签（场景：微信记录）。
*   **Importer**: 也需要专门的导入器。
*   **Fresh Start**: 当然也支持全新开始。

### 问题 10: 空间自由度 (Spatial Freedom)
**提问**: 这是最后一个问题，决定了底层的渲染技术。
您之前提到了 "Pinterest 瀑布流" (Q5) 和 "Apple Freeform 空间网格" (Q1)。这也是两种完全不同的技术路线：

*   **流式布局 (Masonry/Flow)**: 类似 **Pinterest/Google Keep**。
    *   卡片**自动排列**，不能随意重叠或放在空白处。
    *   只能上下滚动，不能缩放(Zoom)。
    *   *技术*: 标准 HTML/CSS。开发快，但"看板感"较弱。
*   **无限画布 (Infinite Canvas)**: 类似 **Heptabase/Miro/Apple Freeform**。
    *   一张无限大的桌子。您可以把便签放在**任意位置**（X, Y），可以堆叠，可以连线。
    *   支持**无级缩放** (Zoom In/Out)。
    *   *技术*: 需要 Canvas/SVG 引擎。开发难度大，但极具"思考感"和"Wow"效果。

为了达到您心中"画板"的效果，**无限画布**是必须的吗？还是普通的**自动排列**就够了？

**回答**: **流式布局 (Masonry)** 更好。
*   不需要无限画布。
*   新增需求：**日历集成** (希望在日历上直接看便签)。
*   新增指令：再问 3 个问题。

---

### 问题 11: 日历交互模式 (Calendar Integration)
**提问**: 您希望能"在日历上直接看见当日便签"。请问具体的**展示形态**应该是？

*   **日历作为筛选器 (Date Picker)**: 左边是一个小日历，点击 "1月25日"，右边的瀑布流就只显示那一天的便签？
*   **全屏日历视图 (Full Calendar View)**: 类似 Outlook/Google Calendar。屏幕变成大日历格，每个格子(Day Cell)里面真的**塞满了当日的便签卡片**？
*   **时间轴侧栏 (Journal Sidebar)**: 屏幕旁边有一条 Time-line，显示"今天、昨天、上周"的锚点？

(全屏日历视图信息密度极大，而筛选器模式比较干净，您倾向于哪种？)

**回答**: **全部支持**。
*   用户希望"这三个都有"，由我来合理安排位置（Layout Management）。
*   *设计点*: 我们需要一个 `CalendarViewMode` (Side-Filter vs Full-Grid vs Journal).

### 问题 12: 待办任务属性 (Task Features)
**提问**: 便签经常用来记"待办事项"。这一块的功能深度如何？

*   **纯文本 (Markdown Only)**: 仅仅是Markdown里的 `- [ ]` 语法。勾选没有任何系统副作用，纯视觉。
*   **结构化任务 (Structured Task)**:
    *   便签本身可以被标记为 **"待办" (ToDo)** 状态？
    *   需要设置 **截止日期 (Due Date)** 吗？
    *   需要 **在此提醒我 (Reminders)** 吗？

(如果您选了"结构化任务"，那么日历视图里可能就需要显示"红色的过期任务")

**回答**: **重型 GTD (Actionable)**。
*   用户明确选择 "重型 GTD"。
*   这意味我们需要完整的：Priority (P0-P3), Due Date (Time-specific), Reminders, Status (Todo/Doing/Done), Assignee (Optional).
*   *架构影响*: Memo 实体需要扩展大量的 Task 字段。

### 问题 13: 移动端与响应式 (Mobile Experience)
**提问**: 这是最后一道设计题。作为一个 GTD + 快速记录工具，**手机端的体验**至关重要。

*   **完全响应式 (Full Responsive)**: 手机浏览器打开就是完美的 App 体验（包括手势操作）。
*   **桌面优先 (Desktop First)**: 手机端只需要一个"极其简陋"的输入框（只负责快速录入），不强求在手机上做复杂的画板整理？

(如果您希望手机端能完美操作"画板/看板"，开发成本会显著增加，因为拖拽在手机上很难做)

**回答**: **完全响应式桌面 (Full Power)**。
*   手机端必须是全功能的 PWA 体验。
*   支持完整的看板、瀑布流查看。
*   **必须支持** Touch Drag & Drop (虽然成本高)。

---

## 需求汇总 (Requirements Synthesis)

经过 13 轮深入讨论，我们明确了 **Self Space: Memos (便签)** 模块的最终形态：

### 1. 核心架构 (Core Architecture)
*   **数据模型**: `Memo` 实体。
    *   **容器**: 属于 `Project` (画板)，但支持一键 `Move` 到其他画板。
    *   **内容**: **Block-First** 架构（复用 Article 的 Block 系统），但 UI 上通过 "Inline Expand" 呈现轻量感。
*   **空间形态**: **Masonry (瀑布流)** + **Kanban (看板)**。拒绝无限画布 (Canvas)，坚持流式布局以保证手机端兼容性和开发稳健性。

### 2. 视图与交互 (Views & Interaction)
*   **多视图切换**: 
    1.  **Timeline**: 时间轴/日志流。
    2.  **Kanban**: 待办看板 (Pending/Done)。
    3.  **Grid/Masonry**: 默认的 Pinterest 瀑布流。
    4.  **List**: 高密度列表。
*   **日历集成**: 全功能日历套件 (Side Picker + Full Month Grid + Journal Stream)。
*   **卡片密度**: **Rich Preview** (标题 + 3行正文 + 缩略图)。
*   **编辑模式**: **Inline Expand** (原位展开编辑)，支持 Markdown/Block。

### 3. 分类与 GTD (Taxonomy & Tasks)
*   **分类**: **Color-First** (颜色优先，红黄绿蓝) + **Tags-Secondary** (标签辅助)。
*   **GTD**: **Heavy (重型)**。
    *   字段: Status (Todo/Done), Priority (P0-P3), Due Date, Reminder.
    *   表现: 过期任务在日历显红，看板视图根据 Status 分组。

### 4. 智能与录入 (Input & AI)
*   **快速录入**:
    *   **Global Quick Note**: 全局快捷键唤起（模态框）。
    *   **Smart Paste**: 粘贴长文本 -> 自动按空行分割为多张便签。
    *   **Context Menu**: 选中文本 -> "Save to Memo"。
*   **AI**: **Manual (暂不介入)**，但数据库预留接口 (`ai_tags`, `analysis_status`)。

### 5. 移动端 (Mobile)
*   **策略**: **Full Responsive Web App**。
*   **要求**: 手机端必须完整支持看板拖拽、瀑布流浏览，不做阉割版。

### 6. 生态集成 (Ecosystem)
*   **Promotion**: 便签 -> 升级为文章 (Article)。**保留**原便签作为 Reference Link。
*   **Reference**: 写文章时，侧边栏支持**拖拽**便签进入文章作为素材。













## Implementation Log (2026-01-25)

**Status**: Phase 1 Implemented
**Spec Reference**: [Memos Module Spec](../specs/memos_module.md)

### What was built
1.  **Backend Core**:
    *    Entity created with full GTD fields (Priority, Status, Color, Dates).
    *   Repository layer implemented with Class Table Inheritance ( + ).
    *   API Endpoints (, , , ) fully functional.

2.  **Frontend Module**:
    *   **Masonry View**: Default waterfall layout for browsing.
    *   **Kanban View**: Status-based columns with Drag & Drop.
    *   **Editor**: Inline-style modal for quick capture.

3.  **Cross-Module Integration**:
    *   Added **Memos Sidebar** to the main Article Editor ().
    *   Supports dragging a memo from sidebar directly into the article as text/reference.

### Next Steps / Remaining
*   **Calendar View**: Placeholder exists, needs implementation.
*   **Mobile Optimization**: Kanban drag-and-drop is currently HTML5-based (Desktop optimized). Needs touch polyfill or library for mobile.
*   **Smart Paste**: Bulk import feature logic needs to be added to Backend Service.
