# Plan: Editor V3 Implementation (Polymorphic & Decoupled)

> **Context**: We are refactoring the editor architecture to support specialized editors under a unified `IEditorAdapter` interface, fixing UI overlaps, and simplifying the state management model.
> **Constraint**: Zero Data Loss. Seamless transition from current "Draft API" to "Versioned Row".

## 1. Backend Foundation (Rust)

- [ ] **Schema Migration**:
    - Add `public_version_id` (UUID, nullable) to `articles` table.
    - Add `draft_content` (JSONB) to `article_details` (optional, for explicit drafts if needed, OR stick to `body` as draft).
    - **Decision**: Use `body` as "Live Draft". Use `content_versions` table for "Published Snapshots".
- [ ] **API Logic Update**:
    - `GET /content/:id`: Return `body` (Live Draft).
    - `GET /content/:id/public`: Return content from `public_version_id` snapshot.
    - `POST /content/:id/publish`: 
        1. Snapshot current `body` to `content_versions`.
        2. Update `public_version_id`.
- [ ] **Cleanup**:
    - Deprecate/Remove `drafts` table and `DraftRepository`.

## 2. Frontend Core (Vue + Adapter Pattern)

- [ ] **Layout Refactor (Fix Overlap)**:
    - Rewrite `EditorView.vue` shell using Flexbox (`relative` header).
    - Remove all absolute positioning hacks for the main container.
- [ ] **The Host Component (`UniversalEditor.vue`)**:
    - Create a wrapper component that accepts `article.type` (default: 'markdown').
    - Dynamically loads `MarkdownEditorAdapter.vue` (wrapping Tiptap).
- [ ] **Metadata Panel**:
    - Create `EditorMetadataPanel.vue` (Floating/Drawer).
    - Move Category, Tags, Status, History controls here.
- [ ] **State Logic**:
    - Update `useContent.ts` to use the new "Live Head" logic (no more `draftApi.get()`).
    - Auto-Save directly to `PUT /content/:id` (debounced).

## 3. Adapter Implementation

- [ ] **Markdown Adapter**:
    - Wrap existing Tiptap editor.
    - Implement `getValue()`, `setValue()`, `export('markdown')`.
- [ ] **Testing**:
    - Verify Import/Export flows.
    - Verify "Edit Live -> Publish -> Public View" consistency.
