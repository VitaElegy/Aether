# Self Space: Memos Module (Technical Specification)

**Version**: 1.0 (Implementation Phase 1)
**Date**: 2026-01-25
**Status**: Implemented

## 1. Overview
The **Memos Module** is a high-speed, structural capture system designed to bridge the gap between "fleeting notes" and "structured knowledge". It follows the "Self Space" philosophy of treating user thoughts as first-class citizens in a Linux-kernel-like architecture.

**Key Features**:
- **Dual Mode**: Works as both a quick capture inbox (GTD) and a creative mood board (Pinterest-like).
- **Core Entity**: Inherits from the system-wide `Node` entity via Class Table Inheritance (CTI).
- **Views**: Masonry (Waterfall) for exploration, Kanban for tasks.
- **Integration**: Deeply integrated into the Article Editor via "Reference Sidebar" for drag-and-drop citation.

## 2. Data Architecture

### 2.1 Database Schema (PostgreSQL)

Memos use the **Class Table Inheritance** pattern.
- **Base**: `nodes` table (Shared metadata: UUID, Authorship, Permissions).
- **Detail**: `memo_details` table (Specific fields).

```sql
-- Base Node (Referenced via Foreign Key)
CREATE TABLE nodes (
    id UUID PRIMARY KEY,
    type TEXT NOT NULL, -- 'Memo'
    created_at TIMESTAMPTZ,
    ...
);

-- Memo Detail
CREATE TABLE memo_details (
    id UUID PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    
    -- Context
    project_id UUID,          -- Optional Board context
    
    -- Visual & Status
    color TEXT NOT NULL,      -- 'Yellow', 'Red', 'Green', 'Blue', 'Purple', 'Gray'
    is_pinned BOOLEAN DEFAULT FALSE,
    
    -- Content (Block-First Architecture compatible)
    content JSONB NOT NULL,   -- Stored clearly, often simple text wrapped in JSON for Phase 1
    
    -- GTD Fields
    status TEXT NOT NULL,     -- 'Todo', 'Doing', 'Done', 'Archived'
    priority TEXT NOT NULL,   -- 'P0', 'P1', 'P2', 'P3'
    due_at TIMESTAMPTZ,
    reminder_at TIMESTAMPTZ,
    
    -- Performance Denormalization
    tags JSONB NOT NULL DEFAULT '[]' -- JSON String Array for fast sidebar filtering
);
```

### 2.2 Domain Model (Rust)

Located in `backend/src/domain/models.rs`.

```rust
pub struct Memo {
    #[serde(flatten)]
    pub node: Node,
    pub content: String, // Mapped to/from JSONB in Repository
    pub priority: String,
    pub status: String,
    pub color: String,
    pub is_pinned: bool,
    pub due_at: Option<DateTime<Utc>>,
    pub reminder_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}
```

## 3. Backend Implementation (Rust/Axum)

### 3.1 API Endpoints
Base URL: `/api/memos`

| Method     | Endpoint | Payload             | Description                      |
| :--------- | :------- | :------------------ | :------------------------------- |
| **POST**   | `/`      | `CreateMemoRequest` | Create a new memo.               |
| **GET**    | `/`      | `author_id` (Query) | List memos (filtered by author). |
| **GET**    | `/:id`   | -                   | Get detailed memo.               |
| **PUT**    | `/:id`   | `UpdateMemoRequest` | Update specific fields.          |
| **DELETE** | `/:id`   | -                   | Delete memo.                     |

### 3.2 Repository Logic
*   **File**: `backend/src/infrastructure/persistence/repositories/memo.rs`
*   **Transactions**: Uses SeaORM transactions to ensure `nodes` and `memo_details` are inserted/updated atomically.
*   **Mapping**: Handles conversion between DB Enums (e.g., `MemoColor::Red`) and API Strings ("Red").

## 4. Frontend Implementation (Vue 3 / TypeScript)

### 4.1 Module Structure
Path: `frontend/src/components/self-space/modules/memos/`

*   **`MemosModule.vue`**: The main shell. Handles routing, view switching (Masonry/Kanban), and search state.
*   **`MemoMasonry.vue`**: CSS Column-based waterfall layout. Optimal for variable-height text cards.
*   **`MemoKanban.vue`**: Horizontal scrolling columns bucketed by `status`. Supports Native HTML5 Drag & Drop for status updates.
*   **`MemoCard.vue`**: The atomic unit. Visualizes color (via Tailwind classes), content preview (line-clamp), and actions (Pin/Delete).
*   **`MemoEditor.vue`**: Modal-based editor. Supports "Inline" feel via backdrop-blur. Edit all GTD fields.

### 4.2 State Management (Pinia)
*   **File**: `frontend/src/stores/memos.ts`
*   **Store**: `useMemosStore`
*   **Getters**: `filteredMemos` (Search/Tag filter), `kanbanColumns` (Group by Status).
*   **Optimistic UI**: Updates local state immediately while sending requests to backend.

### 4.3 Plugin System
*   **Registration**: `frontend/src/plugins/memos.ts`.
*   **Integration**: Registered in `frontend/src/main.ts` and appearing in `ModuleSwitcher` dock.

## 5. Integration: Editor Sidebar

The Memos module is uniquely integrated into the Article Editor (`EditorView.vue`) to serve as a "Reference Factory".

*   **Location**: Right Sidebar (Tabbed: "Settings" | "Memos").
*   **Mechanism**:
    1.  Sidebar lists filtered memos.
    2.  User drags a memo card.
    3.  `dragstart` event sets `dataTransfer` with:
        *   `text/plain`: Memo content (for direct text paste).
        *   `application/json`: Full Memo Object (for potential rich block parsing in Tiptap).
    4.  Tiptap Editor accepts the drop (via default browser behavior for text, or custom extension for JSON).

## 6. Future Enhancements (Roadmap)
*   **Smart Paste**: API endpoint to split bulk text into multiple memos.
*   **AI Auto-Tagging**: Hook into `MemoRepository.save` to trigger an async Node classification job.
*   **Calendar View**: Frontend component `MemoCalendar.vue` using a grid layout to visualize `due_at`.
*   **Mobile Touch Drag**: Replace HTML5 Drag & Drop in Kanban with `useGesture` or Pointer Events for better mobile support.
