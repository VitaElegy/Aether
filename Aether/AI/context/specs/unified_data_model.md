# Aether Unified Data Model (UObject) & Editor Architecture

## 1. Core Philosophy: The "UObject" Pattern

To prevent fragmentation and ensure robust extensibility, Aether adopts a **Unified Object Model** inspired by Game Engine architecture (e.g., Unreal's `UObject` or Unity's `GameObject + Component`).

### 1.1 The Single Source of Truth
Instead of creating separate tables for every new KB type (`math_details`, `kanban_details`, `canvas_details`), we enforce a **Single Content Table Strategy**:

-   **Base Entity (`nodes`)**: Stores identity, hierarchy, permissions, and `type`.
-   **Content Entity (`article_details`)**: Stores the **Payload Body** in a polymorphic format (JSON/Text).

### 1.2 Polymorphic Payload
The `body` field in `article_details` is treated as a **Variant**:

| Node Type | Body Content (Storage) | Editor Interface |
| :--- | :--- | :--- |
| `markdown` | String (Markdown) | `MarkdownEditorAdapter` |
| `kanban` | JSON `{ columns: [...] }` | `KanbanEditorAdapter` |
| `whiteboard`| JSON `{ elements: [...] }` | `ExcalidrawAdapter` |
| `asset` | JSON `{ file_path: "..." }` | `AssetPreviewAdapter` |

**Benefit**: All system-level operations (Backup, Restore, Versioning, Permissions, Search) work identically for ALL types without custom logic.

---

## 2. Editor Architecture V3 (Polymorphic Host)

The Frontend Editor is no longer a specific "Page", but a **Generic Host**.

### 2.1 The Host: `UniversalEditor.vue`
-   **Responsibility**:
    1.  Fetch `Node` + `Body`.
    2.  Check `node.type`.
    3.  Load the registered **Adapter Component**.
    4.  Provide standard `save()`, `autoSave()`, `publish()` hooks.
-   **Layout**:
    -   **Flexbox Shell**: Navigation (Relative) + Editor Body (Flex-1) to prevent overlap.
    -   **Floating Panel**: Metadata/Settings are popped over, never squeezing the content.

### 2.2 The Interface: `IEditorAdapter`
Every specialized editor MUST implement:

```typescript
interface IEditorAdapter {
    // Core: Mount & Load Data
    load(content: any): Promise<void>;
    
    // Core: Return current state for Auto-Save
    getValue(): any;
    
    // Interop: Export to Standard Format (Markdown/PDF)
    // CRITICAL: Even a Kanban board must be able to render a Markdown list representation!
    export(format: 'markdown' | 'json'): Promise<string | Blob>;
    
    // Interop: Import from Standard Format
    import(content: any, format: 'markdown' | 'json'): Promise<void>;
}
```

---

## 3. Data Flow: Hybrid Live Mode

To solve the "Draft vs Published" confusion without complex "Shadow Tables":

### 3.1 Live Head (Editing)
-   **Action**: User edits in `UniversalEditor`.
-   **Storage**: `article_details.body` is updated **continuously** (Debounced Auto-Save).
-   **State**: This is the "Working Copy". It is **Live**.

### 3.2 Public Snapshot (Viewing)
-   **Action**: User clicks "Publish".
-   **Storage**:
    1.  Backend creates a `ContentVersion` snapshot of current `body`.
    2.  Backend updates `nodes.public_version_id` to point to this snapshot.
-   **Read View**:
    -   **Author**: Sees `body` (Working Copy).
    -   **Reader**: Sees content from `public_version_id` (Stable Snapshot).

---

## 4. Implementation Plan

### Phase 1: Backend Schema (The UObject)
- [ ] **Migration**: Add `public_version_id` to `nodes` (or `article_details`).
- [ ] **Refactor**: Ensure `article_details.body` can handle JSON (it is already `Json` type in SeaORM).
- [ ] **Cleanup**: Deprecate `drafts` table.

### Phase 2: Frontend Host
- [ ] **Layout**: Implement Flexbox Shell.
-   **Host**: Create `UniversalEditor.vue` replacing `EditorView.vue`.
-   **Registry**: Implement `EditorRegistry` to map `type -> component`.

### Phase 3: Adapters
- [ ] **Markdown**: Port existing Tiptap editor to `MarkdownAdapter`.
- [ ] **Kanban/Memo**: (Future) Implement adapters for new types.
