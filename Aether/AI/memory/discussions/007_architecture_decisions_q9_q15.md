# Architectural Decisions: Navigation, Persistence & Context (Q9 - Q15)

**Date**: 2026-01-28
**Status**: In Progress

## 1. Confirmed Decisions

### 🟢 Q9: Navigation Topology
**Decision**: **Option B (Smart Trace)**
- **Behavior**: "Back" button follows the user's temporal traversal path (History Stack), not the strict file hierarchy.
- **Rationale**: Supports exploratory learning (Graph traversal) better than rigid folder structures.
- **Implementation**: Rely on `window.history` for the primary "Smart Trace". Fallback to Hierarchy when history is empty.

### 🟢 Q10: Multi-Stack State
**Decision**: **Option B (Freeze State / Keep-Alive)**
- **Behavior**: Switching modules (e.g., Library -> Self Space -> Library) restores the *exact* state (scroll position, input drafts).
- **Rationale**: Essential for "Deep Work" multitasking.

### 🟢 Q11: State Lifecycle Ownership
**Decision**: **Option B (Persistent State via IndexedDB)**
- **Behavior**: State survives browser refreshes and reboots.
- **Tech Stack**: Integration of `localforage` (or similar IndexedDB wrapper). Each Module/SKB must implement `serializeState()` and `restoreState()`.

---

## 2. Active Discussion

### 🟢 Q12: Context Isolation
**Context**: Should Special KBs (SKB) isolate search/navigation?
- **Option A (Strict Sandbox)**: SKB only searches own content.
- **Option B (Permeable Mode)**: SKB searches globally.
- **Proposed (Hybrid)**: "Semi-Permeable Membrane". Default to local context, allow explicit escalation to global.

## 3. Upcoming Questions (Q13 - Q15)
- **Q13**: Conflict Resolution (Syncing Frozen State).
- **Q14**: Renderer Graceful Degradation (Mobile).
- **Q15**: Protocol/Schema Versioning.
