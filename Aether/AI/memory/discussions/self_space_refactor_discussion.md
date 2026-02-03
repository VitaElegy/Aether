# Self Space 架构重构深度讨论记录

> **日期**: 2026-01-30
> **参与者**: 用户 + Senior Systems Architect (AI)
> **目标**: 解决 self_space 渲染逻辑问题并设计改进方案

---

## 📋 问题背景

用户报告的核心问题：
1. 进入特殊知识库时 `self_space` 显示空白
2. 特殊知识库的渲染逻辑不仅没有正常工作，还影响了其他程序

## 🔍 深度讨论（5 个问题）

---

### Q1: Dock 数据流向与去重逻辑

**问题本质**：`dockItems` 同时承担 UI 渲染和组件解析两个职责，导致数据流向不清晰。

**调研结论**：
| 产品               | 模式                                                                |
| ------------------ | ------------------------------------------------------------------- |
| macOS Dock         | 分区设计（Pinned 左 \| Running 右），永不重复，运行状态用小圆点标识 |
| Windows 11 Taskbar | 合并策略，可配置 Always/When Full/Never                             |

**用户选择**：✅ **macOS Dock 模式**

**核心规则**：
- Pinned 区域的应用永远不出现在 Running 区域
- Pinned 应用正在运行时，在其图标下方添加运行指示点
- 非 Pinned 应用打开后进入 Running 区域；关闭后立即消失

---

### Q2: 视觉状态指示

**问题**：如何区分"Pinned 但运行中"与"Pinned 但未运行"？

**选项**：
- A: 图标下方小圆点（macOS 风格）
- B: 微光/高亮环（更 Fancy）
- C: 边框颜色变化
- D: 不做区分

**用户选择**：✅ **A + B 组合**（小圆点 + 微光环）

---

### Q3: 插件加载时机

**问题**：特殊知识库 Dashboard 组件的注册时机问题。

**调研结论**：
| 产品          | 模式                                      |
| ------------- | ----------------------------------------- |
| VS Code       | Activation Events（声明式触发条件）       |
| Obsidian      | 可配置懒加载（Lazy Plugin Loader）        |
| Vue/React SPA | `defineAsyncComponent()` + 路由级代码分割 |

**用户选择**：✅ **混合模式 + VS Code 风格 Activation Events**

**核心规则**：
- **核心插件**（Library, Admin）：启动时全量注册
- **特殊知识库插件**（Math, English, Memos...）：首次访问时懒加载
- 加载过程中显示 Loading 骨架屏

---

### Q4: 状态同步架构

**问题**：`SelfSpaceView.vue` 职责过重，状态分散在多个 Store 中。

**调研结论**：
| 模式                       | 优点                   | 缺点                 |
| -------------------------- | ---------------------- | -------------------- |
| 中央调度器（Orchestrator） | 清晰流程、集中错误处理 | 潜在瓶颈、灵活性降低 |
| 事件驱动（Event Bus）      | 高度解耦、可扩展       | 调试困难、事件泛滥   |
| 混合模式                   | 平衡两者优点           | 需要明确边界         |

**用户选择**：✅ **混合模式（Orchestrator + 局部事件）**

**核心架构**：
```
useSelfSpaceOrchestrator()  ← 核心协调器
├─ switchToKb(id)           // 核心协调入口
├─ dockItems                 // 派生状态
└─ currentComponent          // 渲染决策

事件广播（仅用于扩展点）
└─ 插件可订阅 kb:activated 等事件
```

---

### Q5: 错误边界与降级策略

**问题**：当前错误处理粗糙（alert 弹窗），无恢复选项，错误类型未区分。

**调研结论**：
| 来源    | 最佳实践                                             |
| ------- | ---------------------------------------------------- |
| Vue 3   | 分层错误边界（全局→功能→组件）、有意义的 Fallback UI |
| React   | Suspense + Error Boundary 分离、SafeComponent 模式   |
| VS Code | Extension Bisect、Telemetry、版本兼容检查            |

**用户选择**：✅ **三层防御架构**

**架构设计**：
```
Layer 1: Global Error Handler (app.config.errorHandler)
├─ 捕获所有未处理错误
├─ 发送到日志服务
└─ 显示全局 Toast 通知

Layer 2: KB-Level Error Boundary (SelfSpaceView)
├─ 包裹每个 KB 组件
├─ 捕获渲染/生命周期错误
└─ 显示 BrokenState.vue + 重试/返回选项

Layer 3: Async Loading Fallback
├─ 懒加载时显示骨架屏
├─ 加载超时提示
└─ 加载失败触发 Layer 2
```

---

## 📊 决策汇总

| 问题      | 决策                          |
| --------- | ----------------------------- |
| Dock 模式 | macOS 风格分区设计            |
| 视觉指示  | 小圆点 + 微光环               |
| 插件加载  | 混合模式（核心启动 + 懒加载） |
| 状态架构  | Orchestrator + 局部事件       |
| 错误边界  | 三层防御架构                  |

---

## 📁 相关文档

- **重构规范**: [self_space_refactor_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/self_space_refactor_spec.md)
- **AI 执行提示词**: [self_space_refactor_prompt.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/prompts/self_space_refactor_prompt.md)
