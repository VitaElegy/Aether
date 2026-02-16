# Aether Editor V3: Polymorphic & Decoupled Architecture

## 1. Executive Summary

This specification redefines the **Aether Editor** as a **Polymorphic Host** capable of loading specialized editors for diverse content types (Markdown, Kanban, Canvas, etc.), unified by a strict **Standard Editor Interface (SEI)**.

### Core Philosophy
1.  **Polymorphic Content**: The system is agnostic to internal data structures (Markdown vs JSON vs Binary).
2.  **Standard Interface (SEI)**: All editors MUST implement `load`, `save`, `export(standard)`, `import(standard)`.
3.  **Hybrid Live Sync**: "Edit" is always live (Auto-Save to Main Record). "Publish" is a Snapshot (Version History).
4.  **UI Decoupling**: Layout handles navigation/metadata; Editor handles content.

---

## 2. The Standard Editor Interface (SEI)

Every specialized editor (e.g., `MarkdownEditor`, `KanbanEditor`, `CanvasEditor`) must implement this TypeScript interface:

```typescript
interface IEditorAdapter {
    // Lifecycle
    mount(element: HTMLElement, context: EditorContext): void;
    unmount(): void;
    
    // Core Data Flow
    // Load raw data from backend (could be string, json, or blob)
    load(content: any): Promise<void>;
    
    // Return current state for Auto-Save
    getValue(): any;
    
    // Events
    onChange(callback: (isDirty: boolean) => void): void;
    
    // Portability (CRITICAL)
    // Must provide a standard representation (usually Markdown or PDF)
    export(format: 'markdown' | 'pdf' | 'json'): Promise<string | Blob>;
    
    // Must accept standard representation to reconstruct state
    import(content: string | Blob, format: 'markdown' | 'json'): Promise<void>;
}
```

### 2.1 The Host Component (`UniversalEditor.vue`)

The host component is responsible for:
1.  **Loading**: Detecting `article.type` (e.g., `markdown`, `kanban`).
2.  **Resolving**: Dynamic Import of the correct Editor Component.
3.  **Binding**: Connecting the SEI methods to the Toolbar and Auto-Save logic.

---

## 3. Data Architecture: Hybrid Live Mode

We eliminate the complex "Draft vs Published" duality in favor of a simpler **Live Head + Snapshots** model.

### 3.1 The "Live Head" (Current State)
-   **Storage**: The `article_details` table stores the **current state** in its `body` field (which can be JSON or String).
-   **Behavior**: Every Auto-Save updates this `body`.
-   **Visibility**:
    -   **Private/Internal**: Readers see the live `body`.
    -   **Public**: Readers see the **Latest Published Version** (Snapshot), NOT the live `body`.

### 3.2 The "Snapshot" (History)
-   **Trigger**: User clicks "Publish" or "Commit".
-   **Action**:
    -   Backend creates a `ContentVersion` record copying the current `body`.
    -   Backend updates `article.public_version_id` to point to this new version.
-   **Benefit**: Editors can work on a "Live Draft" without affecting the public view, but without maintaining a separate "Draft Table".

---

## 4. UI/UX Architecture

### 4.1 Layout Strategy (Fixing the Overlap)
-   **Container**: `flex flex-col h-screen`.
-   **Header**: `relative flex-none h-16`. (Natural Flow, no `absolute`).
-   **Body**: `flex-1 relative overflow-hidden`.
-   **Editor**: `absolute inset-0 overflow-y-auto`.

### 4.2 Metadata Panel (The "Obsidian" Style)
-   **Default**: Hidden. Clean writing interface.
-   **Trigger**: "Info" icon in Header.
-   **Behavior**: A **Floating Pop-over** (Right Side) containing Tags, Category, Stats, and History.

---

## 5. Migration Strategy

### 5.1 Database
-   **Cleanup**: Drop `drafts` table (after merging any pending drafts).
-   **Schema**: Ensure `article_details.body` is flexible (JSONB is preferred, but Text is fine if we serialize).
-   **Versioning**: Add `public_version_id` to `articles` table.

### 5.2 Frontend
-   **Refactor**: Rewrite `EditorView.vue` to implement the Flexbox Layout and `UniversalEditor` host.
-   **Adapter**: Wrap the current Tiptap Editor into a `MarkdownEditorAdapter` implementing SEI.
