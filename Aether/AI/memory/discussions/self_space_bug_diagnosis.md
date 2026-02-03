# Self Space Bug 诊断报告 - 待修复

> **日期**: 2026-01-30
> **状态**: 🔴 待修复
> **当前症状**: 
>   1. Dock 点击后选中错位（高亮图标与点击图标不一致）
>   2. 特殊知识库白屏，影响其他程序

---

## 🔍 可能的根本原因分析

### 理论 1: dockItems 顺序问题

**问题描述**：
`ModuleSwitcher.vue` 将 `dockItems` 分成 `pinnedModules` 和 `openModules`，但渲染时的索引与原始数组不一致。

**关键代码位置**：
- `frontend/src/components/self-space/ModuleSwitcher.vue` 第 153-154 行
```typescript
const pinnedModules = computed(() => props.modules.filter(m => isPinned(m)));
const openModules = computed(() => props.modules.filter(m => !isPinned(m)));
```

**风险点**：
- 如果 `isPinned()` 检查与 Orchestrator 中的 `pinned` 属性不一致，会导致错误分组
- CSS 动画或样式可能基于错误的索引应用

---

### 理论 2: isActive 与 props.activeModule 不同步

**问题描述**：
Orchestrator 创建的 `DockItem` 包含 `isActive` 属性，但这个值是静态快照，与 `props.activeModule`（响应式）不同步。

**关键代码位置**：
- `frontend/src/composables/useSelfSpaceOrchestrator.ts` 第 205-207 行
```typescript
return {
    // ...
    isRunning: appStore.runningKbIds.has(kb.id),
    isActive: appStore.activeKbId === kb.id  // ❌ 这是 computed 内部的静态快照
};
```

**问题**：`isActive` 在 `dockItems` computed 中被设置，但它不是响应式的引用，而是创建 DockItem 时的快照值。

---

### 理论 3: activeKbId 与 dockItems.id 类型不匹配

**问题描述**：
`activeKbId` 可能是 UUID 字符串，而 `dockItems[].id` 可能被设置为其他格式（如 renderer_id）。

**关键代码位置**：
- `frontend/src/stores/read_app_state.ts` - `activeKbId` 的来源
- `useSelfSpaceOrchestrator.ts` 第 196 行
```typescript
id: kb.id,  // 应该是 KB 的 UUID
```

**检查点**：
1. `appStore.activeKbId` 返回的是什么格式？
2. `dockItems` 中的 `id` 是否与之匹配？
3. 是否存在 `_resolveId()` 别名映射导致的不一致？

---

### 理论 4: 视觉偏移来自 CSS 伪元素

**问题描述**：
`ModuleSwitcher.vue` 使用 CSS 伪元素 (`::after`, `::before`) 来显示运行状态点和光环。这些伪元素可能与实际按钮位置不对应。

**关键代码位置**：
- `frontend/src/components/self-space/ModuleSwitcher.vue` 第 182-210 行
```css
.dock-item.dock-running::after { /* 运行指示点 */ }
.dock-item.dock-active { /* 激活光环 */ }
.dock-item.dock-active.dock-running::before { /* 脉冲动画 */ }
```

**检查点**：
1. 这些样式是否应用到正确的元素上？
2. 是否有多个元素同时具有 `.dock-active` 类？

---

### 理论 5: 特殊知识库 renderer_id 未正确解析

**问题描述**：
特殊 KB（如 Math, English, Memos）的 `renderer_id` 在 `pluginStore.resolvePlugin()` 中找不到对应的插件，导致组件返回 null 或 BrokenState。

**关键代码位置**：
- `frontend/src/stores/plugins.ts` - `resolvePlugin()` 别名映射
- `frontend/src/composables/useSelfSpaceOrchestrator.ts` - `currentComponent` 解析

**检查点**：
1. 数据库中特殊 KB 的 `renderer_id` 字段值是什么？
2. `aliasMap` 是否包含这些 ID？
3. 对应的插件（如 MathDashboard, MemosModule）是否已注册？

---

### 理论 6: 插件未注册（时序问题）

**问题描述**：
插件注册可能发生在 `SelfSpaceView` 挂载之后，导致首次渲染时找不到插件。

**检查点**：
1. 插件在哪里注册？（搜索 `registerPlugin` 调用）
2. 注册是否在 Orchestrator 初始化之前完成？
3. 控制台是否有 `[PluginStore] Registering plugin:` 日志？

---

## 🔧 建议调试步骤

1. **添加详细日志**：
```typescript
// 在 SelfSpaceView.vue 中
watch(() => appStore.activeKbId, (val) => {
    console.log('[DEBUG] activeKbId changed:', val);
});

watch(() => orchestrator.dockItems.value, (items) => {
    console.log('[DEBUG] dockItems:', items.map(i => ({ id: i.id, isActive: i.isActive })));
}, { deep: true });
```

2. **在 ModuleSwitcher 中输出匹配结果**：
```typescript
const handleClick = (mod: any) => {
    console.log('[DEBUG] Clicked mod:', mod.id, 'Current activeModule:', props.activeModule);
    // ...
};
```

3. **检查数据库 renderer_id**：
```sql
SELECT id, title, renderer_id FROM knowledge_bases;
```

---

## 📁 相关文件索引

| 文件                                                    | 职责                                  |
| ------------------------------------------------------- | ------------------------------------- |
| `frontend/src/views/SelfSpaceView.vue`                  | Shell 视图，传递 activeModule 到 Dock |
| `frontend/src/components/self-space/ModuleSwitcher.vue` | Dock UI，处理点击和样式               |
| `frontend/src/composables/useSelfSpaceOrchestrator.ts`  | 核心状态管理，生成 dockItems          |
| `frontend/src/stores/read_app_state.ts`                 | 保存 activeKbId 状态                  |
| `frontend/src/stores/plugins.ts`                        | 插件注册和解析                        |

---

## 🎯 下一个 AI 的修复清单

- [ ] 验证 `dockItems` 中每个 item 的 `id` 格式是否与 `activeKbId` 一致
- [ ] 移除 `DockItem.isActive` 属性，完全依赖 `props.activeModule` 判断
- [ ] 检查 `pinnedModules` 和 `openModules` 分区是否正确
- [ ] 确认所有特殊 KB 的插件已在启动时注册
- [ ] 添加调试日志追踪实际的数据流
