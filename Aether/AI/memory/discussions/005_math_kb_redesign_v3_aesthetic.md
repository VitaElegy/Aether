# Math Knowledge Base Redesign V3: Aesthetic Overhaul

**Date**: 2026-01-15
**Topic**: Math Knowledge Base Aesthetic Redesign
**Participants**: User, Agent (Senior Systems Architect)

## Context
The user feels the current Math KB interface lacks "beauty" (aesthetic appeal), although the reading page is acceptable. We are initiating a step-by-step Q&A process to define the new design language.

## Discussion Log

### Q1: 视觉隐喻与核心氛围 (Visual Metaphor & Vibe)
**Agent**:
目前的界面您觉得不够美观。为了更好地理解您的审美偏好，您希望这个新的“数学空间”传达出什么样的**核心视觉隐喻**？

*   **高科技指挥中心 (Command Center)**：深色模式、高密度数据、赛博朋克或科幻感，强调精密与力量。
*   **静谧的书房 (Scholar's Desk)**：浅色/护眼色、极简主义、强调排版与留白，像是在一张干净的纸上书写，强调专注与优雅。
*   **抽象艺术画廊 (Art Gallery)**：磨砂玻璃、悬浮元素、流体动画，强调灵动与现代感。
*   **或者您有特定的参考对象**：(例如：“像 Notion 但是更数学”，“像某个游戏的 UI”等)。

请描述一下，当您进入这个页面时，希望获得什么样的**感觉**？

**User**: 
静谧的书房 (Scholar's Desk)。
风格：极简、优雅、大面积留白、强调衬线字体排版。
感觉：像是在一张昂贵的信纸上书写，没有任何干扰，只有纯粹的数学。

### Q2: 布局与空间划分 (Layout Structure)
**Agent**:
既然选择了“静谧书房”风格，布局应当最大化阅读体验。您希望核心的**空间结构**是怎样的？

*   **极简单栏 (The Scroll / The Manuscript)**
    *   **形态**：屏幕中央只有正文，像一份长卷轴或手稿。
    *   **交互**：导航/目录/图谱默认全部**隐藏**。只有当鼠标移动到屏幕边缘，或点击特定按钮时，它们才优雅地浮现。
    *   **优点**：沉浸感最强，符合“纯粹数学”的感觉。

*   **经典双栏 (The Textbook)**
    *   **形态**：左侧保留一个极简的“目录/知识树”侧边栏，右侧是正文。
    *   **交互**：侧边栏始终可见（或可折叠），方便随时跳转。
    *   **设计细节**：侧边栏会去掉所有复杂的背景和框线，仅保留文字，像书页边缘的索引。

*   **三栏工作台 (The Workbench)**
    *   **形态**：左侧目录，中间正文，右侧是“辅助区”（显示当前定理的局部图谱、引用、备注）。
    *   **优点**：信息密度高，不用来回跳转，但可能会稍微打破“极简”的感觉。

您倾向于哪种布局？

**User**: 
1+2+3 综合。以 **1 (极简单栏)** 为主常态，只有在需要时才展开 2 (目录) 和 3 (辅助栏)。

### Q3: 交互触发与过渡 (Interaction & Triggers)
**Agent**:
非常棒的“渐进式披露” (Progressive Disclosure) 策略。为了保持常态的“静谧”，我们需要确定这些面板**如何出现**以及**何时出现**。

1.  **左侧目录 (TOC)**：
    *   **触发方式**：
        *   A. **鼠标悬停 (Hover)**：鼠标移到屏幕最左侧边缘时自动滑出。
        *   B. **菜单按钮 (Toggle)**：左上角始终保留一个极简的图标 (如 `≡` 或 `Index`)，点击展开。
    *   **出现方式**：
        *   **悬浮 (Overlay)**：盖在正文上方（像从抽屉里拿出来的纸盖在桌上）。
        *   **推挤 (Push)**：将正文平滑推向右侧（保留阅读位置）。

2.  **右侧辅助栏 (Auxiliary/Graph)**：
    *   **触发场景**：
        *   A. **主动点击**：只有点击正文中的某个“定理引用”或“关键词”时，右侧栏才弹出来显示详情。
        *   B. **双击/快捷键**：双击空白处或按 `Cmd+I` 唤出通用工具栏。
    *   **内容**：当没有选中特定内容时，右侧应当是空白/隐藏，还是显示当前文章的“全局图谱”？

请告诉我您理想的交互流。

**User**: 
左侧：保留一个极淡的衬线体 "Index" 或章节序号在左上角，点击展开。默认完全隐藏。
右侧：辅助栏显示组件预览（如函数定义、定理、文章链接）。点击链接时在右侧预览详细信息，可进一步跳转。
过渡：先试试 **Push (推挤)** 效果。

## 3. Decisions Summary (设计决策总结)

Based on the discussion, we have defined the **"Scholar's Desk V3"** Design Language:

### 3.1 Core Metaphor
*   **Vibe**: **Scholar's Desk (静谧书房)**.
*   **Key Elements**: Expensive Paper (Cream/Off-white), Serif Typography (Noto Serif/Computer Modern), "Ink" Colors, Silence (No clutter).

### 3.2 Layout Architecture: "The Hybrid Manuscript"
*   **Default State**: **Single Column (Manuscript)**. Centered text, maximum breathing room.
*   **Left Panel (Navigation)**:
    *   **Trigger**: Explicit Click on a faint "Index" / Chapter Number label in top-left.
    *   **Behavior**: **Push**. Shifts the manuscript right to reveal the structure.
*   **Right Panel (Context)**:
    *   **Trigger**: Click on **Semantic Links** (Theorems, Functions, Refs) in the text.
    *   **Content**: **Contextual Preview Components** (Mini-Graph, Function Definition, Theorem Statement).
    *   **Behavior**: **Push**. Shifts manuscript left.

### 3.3 Typography & Color
*   **Font**: Serif for Body and Headers (e.g., *Noto Serif SC*, *Times New Roman* style).
*   **Background**: Not pure white (`#ffffff`), but "Paper White" (e.g., `#fdfbf7` or `#faf9f6`).
*   **Accent**: "Scholar Blue/Red" (Muted, ink-like), no neon colors.

---
**Status**: Ready for Implementation Planning.




