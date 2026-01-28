# Self Space Integration: Design Decisions (Deep Inquiry)

**Date**: 2026-01-28
**Participants**: User, Senior Architect (Elegy)
**Topic**: Integration of Special Knowledge Bases (SKBs) like Memos, English, and Vulnerability Mining into the core generic system.

---

## 1. Lifecycle & Cardinality
**Decision**: **Mixed Mode (Templates + Singletons)**.
- System supports both "Singleton" modules (e.g., Ticketing System, one per user) and "Instantiatable Templates" (e.g., Memos, English).
- User can create multiple distinct "Memo Spaces" (e.g., Work Memos, Personal Memos).

## 2. Data Isolation
**Decision**: **Soft Isolation (Scoped Default / Optional Global)**.
- Default View: Strictly scoped. "Work Memos" does not show "Personal Memos" tags or content.
- Global Search: User can explicitly toggle global scope to search across all spaces.
- **Rule**: Search results must respect the active scope by default.

## 3. Navigation Topology
**Decision**: **Manual Pinning + Grouped Dock**.
- **The "Dock"**: The bottom-center navigation bar in Self Space.
- **Pinning**: Users manually "Star/Pin" specific KBs from the Management List. Only pinned items appear in the Dock.
- **Grouping**: To prevent overcrowding, pinning multiple KBs of the same type (e.g., 3 Memo KBs) groups them under a single "Memo Type" icon, which expands on click.

## 4. Entry Point (Dashboard)
**Decision**: **Reuse Knowledge List**.
- The existing `Self Space > Knowledge` page serves as the unified manager.
- **Requirement**: Special KBs must be visible here (currently missing/filtered).
- **UI**: Add "Pin to Dock" action on cards.

## 5. Metadata & Association
**Decision**: **Weak References (Loose Citations)**.
- Associations (e.g., Word Card -> Source Article) use standard Block Links (`[[UUID]]`).
- No hard-coded Foreign Keys. Use "Backlinks" logic to show "Source Context".

## 6. Configuration Paradigm
**Decision**: **Opinionated Defaults + Advanced Settings**.
- **Creation**: One-click preset (minimal friction).
- **Post-Creation**: dedicated Settings page for power users (FSRS parameters, etc.).

## 7. Data Portability
**Decision**: **Hybrid Strategy**.
- **JSON**: For full system state/restore (includes FSRS history, complex math).
- **Markdown**: For human readability and interoperability (standard folders/files).
- User chooses format at export time.

## 8. Search Result Presentation
**Decision**: **Categorized Sections**.
- Global Search results are grouped visually:
    - **Documents**: Articles, Papers.
    - **Cards/Entities**: Memos, Vocabulary, Tickets.
- **Goal**: Prevent high-volume fragment data (Memos) from polluting the document search experience.

## 9. Permission Model
**Decision**: **Strict Inheritance with Privacy Narrowing**.
- **Basics**: KB permissions cascade to children.
- **Constraint**: Child items can be **Stricter** than parent (e.g., a "Private" doc inside a "Shared" KB must remain invisible to others).
- **System**: ReBAC must check the specific node's effective policy, ensuring "Deny" or "Private" scopes override inherited "Public/Shared" scopes.

## 10. Activation & Migration
**Decision**: **Fresh Start**.
- Ignore/Hide legacy hardcoded modules.
- User will manually create new, V2-compliant special KBs via the new flow.
- Ensures data integrity and schema compliance from Day 1.
