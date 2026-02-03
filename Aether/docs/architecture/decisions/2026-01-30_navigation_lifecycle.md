# Discussion: Standardizing Knowledge Base Navigation & Switching

## Context
Current Issue: Switching to the "System" KB (and potentially others) causes a blank screen and UI freeze. This suggests a race condition, state desynchronization, or lack of a clear "Switching Protocol" between the global Dock and the routing system.

## Objective
Establish a robust, modular, and error-proof protocol for switching between Knowledge Bases (Apps) in the Aether OS.

### Question 1: The "Source of Truth" for Active State
**Decision**: **Option B (Store Driven)**.
- `appStore.switchKB(id)` is the primary action.
- UI updates immediately based on Store state.
- Router URL is updated reactively (sync) to match the Store, but is not the driver.
- **Goal**: Native app-like responsiveness.

### Question 2: Component Lifecycle & Memory (Keep-Alive)
**Decision**: **Option B (Global Keep-Alive)**.
- We will wrap the main router view for KBs in `<KeepAlive :max="10">`.
- Components will persist in memory (pausing execution via `onDeactivated`).
- Switching back is instant (no mounting cost).
- **Implication**: Every KB component must implement `onActivated` to check for data staleness.

### Question 3: The "Loader" Protocol (Transition State)
**Decision**: **Option B (Target App Self-Managed)**.
- Every KB component is responsible for rendering its own Skeleton/Loading state immediately upon mount/activation.
- **Requirement**: This MUST be documented in `specs` and `guide` so future AI agents (and devs) do not forget to implement the loading state.
- **Contract**: The Shell mounts the component instantly. The Component checks `!isReady` and shows a skeleton.

### Question 4: Dock & "System App" Registration
**Decision**: **Option B (Segregated Zones - macOS Style)**.
- **Left Zone**: Pinned/Persistent User Apps.
- **Divider**: A visual separator.
- **Right Zone**: Transient/Open Apps (or System Tools if pinned there).
- **System Apps (Admin)**: Can be pinned to the Left zone, or appear in the Right zone when opened.
- **Implementation**: The Dock component will iterate over two lists: `pinnedKBs` and `openKBs`.

### Question 5: Error Handling & Fallback
**Decision**: **Option A (Crash to Desktop / Auto-Close)**.
- If a renderer fails (timeout/error), the OS catches it.
- Action: Unmount the component, return to Home/Dock.
- Feedback: Show a Toast notification "App Crashed: [Name]".
- **Philosophy**: "Fail Fast, Fail Clean". Don't let a bad UI state persist.

---

## Part 2: Process Standardization & AI Protocols

We are now defining the **Legislative Framework** for future AI development. This ensures that rules (like "Must have Loading State") are not just chat suggestions but enforced laws.

**Decision**: **Option A (`AI/context/specs/navigation_lifecycle_spec.md`)**.
- We will create a new, formal spec file in the existing `AI` context directory.
- This file will serve as the "Constitution" for navigation logic.
- Future Agents will be instructed to read this file before touching `SelfSpace.vue` or KB components.

### Question 7: The "Police" (Enforcement Mechanism)
**Decision**: **Option A (Passive Linting Script)**.
- We will build `scripts/ai/audit_kb_specs.ts`.
- It will perform AST analysis to check for:
    - `v-if="isLoading"` (or equivalent skeleton logic)
    - `onActivated` hook presence.
- We will update `project_spec.md` to mandate running this script.

### Question 8: The Feature Lifecycle (Artifact Retention)
**Decision**: **Option A (Archive to Repo)**.
- We will establish `docs/architecture/decisions/` (ADR folder).
- Upon task completion, we move the `discussion.md` there, renamed to `ADR-001_navigation_refactor.md`.
- This preserves the "Why".

---
**Discussion Concluded.**
All 8 points agreed. Proceeding to Implementation Plan.
