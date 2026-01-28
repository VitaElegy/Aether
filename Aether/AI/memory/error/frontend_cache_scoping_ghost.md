# Error Report: Frontend Cache Scoping (Ghost Articles/Refreshes)

## 1. Issue Description
**Symptom**: User creates an article (e.g., "Draft A") in **Folder A**.  When they subsequently navigate to **Folder B** and click "New Article", the editor pre-fills with "Draft A" instead of starting empty. Additionally, users reported unexpected page refreshes when navigating between locations.
**Severity**: Medium (Data Integrity / User Confusion).
**Status**: Resolved.

## 2. Root Cause
1.  **Aggressive Cache Restoration**:
    - The Frontend `EditorView.vue` relied on `localStorage.getItem('editor-content')` to restore unsaved drafts.
    - This logic checked if `id` was null (indicating a new entry) but **failed to validate the target location** (`knowledge_base_id`, `parent_id`).
    - Consequently, *any* cached "new entry" data was blindly restored regardless of where the user currently was in the application.

2.  **Ghost Refreshes**: 
    - The `watch` logic for auto-saving would sometimes trigger immediately upon this incorrect restoration, saving the "ghost" content to the *new* location's context if the user wasn't careful, or causing reactive loops as route parameters fought with form state.

## 3. Resolution
### 3.1 Strict Context Matching
We modified the restoration logic in `EditorView.vue` `onMounted` hook:

- **Constraint Added**: When restoring a draft where `id` is null (New Entry), we now rigorously compare the **Cached Context** vs **Current URL Context**:
    ```typescript
    const kbMatch = (intendedKbId === cacheKbId) || (!intendedKbId && !cacheKbId);
    const parentMatch = (intendedParentId === cacheParentId) || (!intendedParentId && !cacheParentId);
    
    if (kbMatch && parentMatch) {
       // Restore
    } else {
       // Ignore Cache
    }
    ```
- This ensures that a draft started in one folder stays in that folder's context and doesn't "leak" into others.

## 4. Verification
- **Reproduction Script**: Updated `backend/scripts/debug/repro_kb_folder.sh` confirmed that the backend correctly handles independent parent IDs, proving the issue was purely frontend state leaking.
- **Manual Test**: Validated that "Draft A" in "Folder A" does not appear when opening "New Article" in "Folder B".
