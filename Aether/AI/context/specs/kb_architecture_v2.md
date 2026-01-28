# Aether Knowledge Base Protocol V2 (Architecture Specification)

**Status**: FINAL DRAFT
**Version**: 2.1
**Date**: 2026-01-28
**Architect**: System Agent (Elegy)

---

## 1. Executive Summary

This document defines the technical standard for "Special Knowledge Bases" (SKBs) within the Aether platform. It transitions the system from a Document-Centric model to a **Block-First Architecture** with a **Hybrid Governance** model.

The goal is to support high-complexity, domain-specific interfaces (e.g., Math 3D Graphs, Structured Memos) while maintaining commercial-grade stability, security, and cross-platform portability (Tauri/Mobile ready).

---

## 2. Core Architecture Decisions (The 15 Pillars)

### 2.1 Data Model & Storage
1.  **Atomicity**: **Backend Block Table**.
    -   Storage is fully normalized. Creating a document creates rows in a `blocks` table.
    -   **Schema**: `id (UUID)`, `document_id (UUID)`, `type (VARCHAR)`, `revision (INT)`, `payload (JSONB)`.
    -   *Rationale*: Enables cross-document block references (transclusion) and fine-grained history.
2.  **Scope**: **Template First**.
    -   Developers must use standard Block templates where possible. Raw code overrides are a fallback of last resort.
3.  **Assets**: **Lazy Garbage Collection**.
    -   Deleting a block does NOT immediately delete referenced assets (images).
    -   A background `Cleaner Job` runs periodically to sweep orphaned assets.
    -   *Rationale*: Optimizes UI responsiveness and simplifies concurrency control.

### 2.2 Governance & Security
4.  **Governance**: **Hybrid Defense-in-Depth**.
    -   **L1 (Envelope)**: Postgres enforces `id`, `type`, `author` integrity.
    -   **L2 (Validation)**: Rust backend uses **Cached Compiled JSON Schemas** to validate `payload` structure.
    -   **L3 (Quotas)**: Schemas are checked for ReDoS complexity and depth limits upon registration.
5.  **Syntax Mapping**: **Core Dictatorship**.
    -   The Core System defines the canonical mapping of Markdown to Blocks.
    -   Example: `$$...$$` is ALWAYS parsed as a standard `MathBlock`. Plugins cannot override standard syntax.
6.  **Error Handling**: **Quarantine Blocks**.
    -   Frontend wraps every block renderer in `<Suspense>` and `onErrorCaptured`.
    -   A crashing block displays a "Red Box" (Quarantine UI) instead of crashing the entire page.

### 2.3 UX & Navigation
7.  **Navigation Topology**: **Smart Trace**.
    -   "Back" button respects the user's traversal graph (History Stack), not just the file hierarchy.
    -   Supports "Deep Exploration" (A -> B -> C -> Back -> B).
8.  **Multi-Stack State**: **Freeze + Persistence**.
    -   Switching Sidebar Tabs (e.g., Library -> Memo) preserves the **exact state** (scroll, input) of the previous tab.
    -   **Mechanism**: State is serialized to **IndexedDB** on switch. Resurrected on return.
9.  **Search**: **Mandatory Searchable Trait**.
    -   All Block implementations MUST implement a `to_text()` trait in Rust.
    -   Indexing ignores JSON structure and only indexes this purified text.
10. **Context Isolation**: **Strict Sandbox (Default)**.
    -   SKBs (e.g., Math) isolate search/sidebar scope by default.
    -   **Escape Hatch**: UI provides a "Search Global" toggle to break isolation.

### 2.4 Frontend Architecture
11. **Renderer**: **Headless Fallback**.
    -   Core includes lightweight HTML renderers for all standard blocks (e.g., static MathJax).
    -   Heavy interactive renderers (3D Graphs) are loaded only when the specific SKB View is active.
12. **Layout Contract**: **Strict Isolation**.
    -   Plugins have NO control over the Shell (TopBar/Sidebar).
    -   Plugins render strictly within the Canvas `<div>`.
    -   *Rationale*: Prevents UI inconsistency and Z-Index wars.
13. **Dependency Injection**: **Dynamic Lazy Loading**.
    -   Shared libraries (Three.js, D3) are located in `/public/libs/`.
    -   Plugins request them via a Loader Service, ensuring only one copy is loaded memory-wise.

### 2.5 Portability
14. **Degradation**: **Mandatory Text Mirror**.
    -   Every Visual Block schema must include a `summary_text` field.
    -   Used for Mobile/CLI fallback and Accessibility (Screen Readers).

---

## 3. Data Schema Specifications

### 3.1 The Block Table (Postgres)
```sql
CREATE TABLE blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id),
    type VARCHAR(50) NOT NULL, -- e.g., 'text', 'math', 'image'
    revision INT DEFAULT 1,
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    plain_text TEXT GENERATED ALWAYS AS (payload->>'text_mirror') STORED, -- Optimization for search
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
-- Index for document retrieval
CREATE INDEX idx_blocks_doc ON blocks(document_id, ordinal_position);
```

### 3.2 The Schema Registry (Rust Struct)
```rust
struct BlockDefinition {
    type_name: String,
    validation_schema: JSONSchema, // Compiled
    complexity_score: u8,
}

trait Searchable {
    fn to_search_text(&self, payload: &Value) -> String;
}
```

---

## 4. Implementation Guidelines

### 4.1 Frontend: The Quarantine Component
```vue
<!-- BlockWrapper.vue -->
<template>
  <ErrorBoundary>
    <Suspense>
      <component :is="renderer" :data="blockData" />
      <template #fallback>
        <SkeletonLoader />
      </template>
    </Suspense>
    <template #error="{ err }">
      <div class="quarantine-box">
        <p>Render Failed: {{ blockData.type }}</p>
        <button @click="showSource">View Source</button>
      </div>
    </template>
  </ErrorBoundary>
</template>
```

### 4.2 State Persistence (IndexedDB Adapter)
- Use `localforage` for key-value storage.
- **Key**: `session_state:{tab_id}:{route_path}`
- **Value**: JSON Object (ScrollPos, FormInput, CursorPos)
- **Trigger**: `onBeforeRouteLeave` or `onDeactivated`.

---

## 5. Migration Strategy
1.  **Legacy Import**: Convert existing `content` (Markdown string) to `Block[]` using a specialized Migration Script.
2.  **Dual Write**: Temporarily write to both `nodes.content` and `blocks` table during transition.
3.  **Cutover**: Switch read path to `blocks` table once validation passes.

---

## 6. Special Knowledge Bases (Self Space Integration)

### 6.1 Definition
Special Knowledge Bases (SKBs) are instances of `KnowledgeBase` distinguished by their `renderer_id` (e.g., `memo`, `english`, `vrkb`). They are not separate database tables but distinct "Lenses" on the generic Node graph.

### 6.2 Lifecycle & Topology
-   **Mixed Mode**: Supports both **Singleton** (Ticketing) and **Template** (Multiple Memo Spaces) lifecycles.
-   **Discovery**: Managed via the unified `Self Space > Knowledge` list view.
-   **Activation**: Users "Pin" specific SKBs to the Self Space Dock.

### 6.3 Dock Protocol (Pinning)
-   **Storage**: `UserPreferences` (Local Storage / Backend Sync) stores list of `pinned_kb_ids`.
-   **UI Behavior**:
    -   Dock displays icons for Pinned KBs.
    -   **Grouped Expansion**: If multiple KBs of type `memo` are pinned, they group under a single Memo Icon which reveals a sub-menu on click.

### 6.4 Permissions
-   **Strict Inheritance**: Access to the SKB implies access to its children.
-   **Privacy Narrowing**: Child nodes can enforce **stricter** permissions (e.g., Private Doc in Shared KB). The System ReBAC must respect this "Deny-Override".

### 6.5 Navigation & Reactivity
-   **Component Recycling**: The Dock uses `<KeepAlive>` for performance.
-   **Reactive Watchers**: All Module Components MUST implement `watch(() => props.kbId)` to handle switching between Pinned KBs of the same `renderer_id`.
-   **State Isolation**: Switching from "Memo A" to "Memo B" must completely reset local state (loading flags, data arrays) to avoid data bleeding.
