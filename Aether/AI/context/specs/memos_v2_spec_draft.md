# Self Space: Memos Module V2 (Fragmented Information)

**Version**: 2.0 (Upgrade)
**Date**: 2026-02-09
**Status**: Draft

## 1. Core Philosophy: "Atomic Fragments, Fluid Context"

The V2 upgrade transforms the Memos module into a high-speed "Thought Stream" engine. It prioritizes:
1.  **Atomic Capture**: Every thought is an independent entity.
2.  **Fluid Organization**: Categories are just saved filters (Tags), not rigid folders.
3.  **Hybrid Interaction**: Seamlessly switch between "Chatting with yourself" (Input) and "Managing Knowledge" (Review).

## 2. User Experience Design

### 2.1 The "Chat Stream" Interface (Center Stage)
Replacing the pure Masonry layout, the main view becomes a **Chronological Stream**:
*   **Visual Style**: Hybrid Chat/Card.
    *   **Short Text**: Renders as a "Chat Bubble" (left-aligned), compact and clean.
    *   **Long Text / Rich Content**: Renders as a "Card" with a "Show More" expansion, title visualization, and image previews.
*   **Time Grouping**: Explicit dividers for "Today", "Yesterday", "This Week", etc.
*   **Interaction**: Click any bubble/card to open the **Detail Modal** (Editor Mode).

### 2.2 The "Smart Dock" (Left Sidebar)
Abandoning the "Filter Panel", the sidebar becomes a **Navigation Dock**:
*   **Pinned Zones (Favorites)**:
    *   User-defined "Channels" like `#Paper`, `#Dev`, `#Idea`.
    *   **Architecture**: These act as *Saved Searches*. Clicking `#Dev` applies `tag:Dev`.
    *   **Persistence**: Synced to Cloud (Database) so it persists across devices.
*   **All Tags**: Expandable tree view of all tags used in the system.
*   **Timeline**: Optional "Calendar View" or Month list.

### 2.3 Hybrid Input (The "Compose Bar")
*   **Mode A: Quick Capture (Bottom Bar)**
    *   Fixed at the bottom of the stream (like Telegram/Discord).
    *   Input: `Text` + `Top-Left Tag Selector` + `Image Upload`.
    *   Action: `Enter` to send. Immediate rendering in stream.
*   **Mode B: Editor Modal (Expansion)**
    *   Triggered by an "Expand" button in the Quick Capture bar OR clicking "New Note".
    *   Full Markdown editor, Title field, Property manager.

## 3. Data Architecture Changes

### 3.1 Database Schema (PostgreSQL/SQLite)
We need to update `memo_details` and add a user preference system.

```sql
-- 1. Update Memo Details (No major structural change, mostly usage pattern)
-- Ensure 'tags' is a robust JSONB array (already is).

-- 2. New Table: User Preferences (Scope: Module Config)
CREATE TABLE user_module_settings (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    module_key TEXT NOT NULL, -- 'memos'
    settings JSONB NOT NULL,  -- { "pinned_tags": ["Dev", "Paper", "Life"], "view_mode": "stream" }
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, module_key)
);
```

### 3.2 Backend API Updates
*   **GET /api/memos**: Add `group_by=date` support (optional, or handle in frontend).
*   **GET /api/users/settings/:module**: Fetch pinned tags.
*   **PUT /api/users/settings/:module**: Update pinned tags.

## 4. Frontend Implementation Strategy

### 4.1 Components
*   `MemoStream.vue`: The new main container. Handles date-grouping logic.
*   `MemoBubble.vue`: Component for short text (Atomic).
*   `MemoCardV2.vue`: Component for long/rich content (Complex).
*   `SmartDock.vue`: Left sidebar with Drag-and-Drop pinning.
*   `ComposeBar.vue`: The bottom input component.

### 4.2 State Management
*   `useMemoStore`:
    *   Add `pinnedTags` state.
    *   Add `viewMode` state (Stream vs Masonry).
    *   Action `togglePin(tag)`: Optimistic update + API call.

## 5. Migration Plan
1.  **DB**: Create `user_module_settings` table.
2.  **Backend**: Implement Settings API.
3.  **Frontend**:
    *   Refactor `MemosModule.vue` layout (3-pane: Sidebar | Stream | Right-Panel).
    *   Implement `SmartDock`.
    *   Implement `ComposeBar`.
    *   Refactor List View to support Date Grouping.
