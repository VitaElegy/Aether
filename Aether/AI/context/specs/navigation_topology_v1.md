# Navigation Topology Specification V1 (Smart Trace)

**Date**: 2026-01-28
**Status**: Implemented
**Implementation**: `frontend/src/stores/navigationStack.ts`, `frontend/src/router/index.ts`

## 1. Core Concept (Smart Trace)
The "Smart Trace" topology decouples browser history from application navigation.
- **Problem**: Browser History is a global stack. Interleaving navigation between Modules (Math <-> English) corrupts the logical "Back" path for a specific module.
- **Solution**: We implement **Scoped Internal Stacks** for each Module.

## 2. Data Structure (`NavigationStackStore`)

We expand `Record<string, string>` (Last Route) to `Record<string, string[]>` (Full History).

```typescript
type ModuleId = string; // e.g., 'math_kb', 'english', 'library'

interface NavigationState {
    stacks: Record<ModuleId, string[]>; // The history stack for each module
    activeModule: ModuleId | null;
}
```

## 3. The Logic

### 3.1 Push (Forward Navigation)
**Trigger**: User clicks a link *within* a module (e.g., `<a>` or `router.push`).
**Action**:
1. Detect current `activeModule`.
2. Push new `fullPath` onto `stacks[activeModule]`.
3. Execute `router.push(to)`.

### 3.2 Back (Smart Return)
**Trigger**: User clicks Global Beacon "Back" Arrow.
**Action**:
1. Get stack for `activeModule`.
2. If `stack.length > 1`:
    - `stack.pop()` (Remove current page).
    - `prev = stack.last()`.
    - `router.replace(prev)`. (Use replace to avoid adding to browser history).
3. If `stack.length <= 1`:
    - **Fallback**: Navigate to Module Root or Home.

### 3.3 Module Switching
**Trigger**: Sidebar click (Library -> Self Space).
**Action**:
1. `activeModule` changes.
2. Resume: `router.push(stacks[newModule].last())`.

## 4. Integration Points

### 4.1 Global Beacon (`GlobalBeacon.vue`)
- Must connect to `useNavigationStackStore`.
- Override default `router.back()` logic.

### 4.2 Router Interceptor (`router.beforeEach`)
- Need a mechanism to detect "Forward" vs "Back" vs "Switch".
- **Crucial**: How to distinguish a "Back" action from a "Push"?
    - *Constraint*: Browser Back button is hard to intercept perfectly.
    - *Decision*: We prioritize **In-App Navigation**. If user uses Browser Back, we sync our stack best-effort or reset.
    - *Refinement*: We only manage the stack via our own controls.

## 5. Persistence
- Use `IndexedDB` (via `localforage`) to save the `stacks` map.
- Serializer: Simple JSON.
