# 协议：创建新的专用知识库 (SKB) - V2

**版本**: 2.0
**适用范围**: 新增知识库类型 (例如: 看板 Kanban, 白板 Whiteboard, 表格 Sheet)

---

## 1. 概述
在 Aether V2 中添加一个新的 SKB 是一个受控过程，必须遵循 **Block-First Architecture (区块优先架构)**。你不能仅仅“写一些 Vue代码”就完事。你必须实现特定的接口ÿ以保证**可搜索性**、**稳定性 (隔离机制)** 和 **持久化**。

---

## 2. 后端实现 (Rust)

### 步骤 2.1: 定义 Block Schema
你必须为你的 Block 定义 JSON Schema。这用于数据**校验**和**配额计算**。

**位置**: `backend/src/domain/blocks/schemas/{your_kb}.rs` (建议位置)

```rust
// 示例: 看板卡片 Block
pub fn get_kanban_card_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "status": { "type": "string", "enum": ["todo", "doing", "done"] },
            "assignee": { "type": "string" },
            "due_date": { "type": "string", "format": "date-time" }
        },
        "required": ["status"]
    })
}
```

### 步骤 2.2: 实现 `SearchableBlock`
你 **必须** 实现 `SearchableBlock` trait。将复杂的 JSON 数据清洗为纯文本，以便 Meilisearch 进行索引。

**接口定义**: `backend/src/domain/blocks/models.rs`
```rust
impl SearchableBlock for KanbanBlock {
    fn to_search_text(&self, payload: &Value) -> String {
        // 仅提取人类可读的文本。忽略 ID、颜色、定位坐标等杂讯。
        format!("{} {}", 
            payload["title"].as_str().unwrap_or(""), 
            payload["description"].as_str().unwrap_or("")
        )
    }
}
```

### 步骤 2.3: 在 `SchemaRegistry` 中注册
在 `backend/src/main.rs` (或启动逻辑中)，注册你的 Block 类型并指定复杂度分数。
```rust
registry.register("kanban_card", schema, 10)?; // 10 = 低复杂度
```

---

## 3. 前端实现 (Vue 3)

### 步骤 3.1: 渲染器组件 (Renderer)
创建你的可视化组件。它 **必须** 接受标准的 `block` prop。

**位置**: `frontend/src/renderers/{YourKB}/`
```typescript
interface Props {
  block: Block; // 标准 Block 类型
}
```

### 步骤 3.2: 接入隔离层 (Quarantine)
**不要** 直接在页面中使用你的渲染器。必须通过 `BlockDispatcher` 注册，该分发器会将其包裹在 `<BlockWrapper>` 中。

```typescript
// frontend/src/renderers/registry.ts
registerRenderer('kanban_card', () => import('./KanbanCard.vue')); // 懒加载!
```

### 步骤 3.3: 降级支持 (Text Mirror)
确保每当内容变更时，你的 UI 会同步更新 Block payload 中的 `text_mirror` 字段，这样移动端或 CLI 用户可以看到有意义的文本内容。

---

## 4. 状态持久化 (打造“线性”体验)

### 步骤 4.1: 定义持久化 Key
如果你的 KB 有特定的视图状态（如滚动位置、筛选器）：

```typescript
// frontend/src/services/session.ts
const PERSISTENCE_KEY = `kb_state:kanban:${kb_id}`;
```

### 步骤 4.2: 实现序列化 (Serializer)
在 `onBeforeRouteLeave` 中，将状态保存到 IndexedDB。
```typescript
onBeforeRouteLeave((to, from) => {
    sessionService.save(PERSISTENCE_KEY, {
        scrollX: window.scrollX,
        filter: activeFilter.value
    });
});
```

### 步骤 4.3: 实现恢复 (Hydrator)
组件挂载时 (onMounted)，恢复状态。
```typescript
onMounted(async () => {
    const state = await sessionService.restore(PERSISTENCE_KEY);
    if (state) {
        window.scrollTo(state.scrollX, 0);
    }
});
```

---

## 5. PR 审查清单

- [ ] **后端**: Schema 是否已定义且有效?
- [ ] **后端**: `to_search_text` 是否纯净 (无 JSON 噪音)?
- [ ] **前端**: 渲染器是否懒加载?
- [ ] **前端**: 是否启用了 Quarantine 隔离包裹?
- [ ] **持久化**: 刷新浏览器 (F5) 后状态是否保留?
- [ ] **移动端**: `text_mirror` 是否已正确填充?
