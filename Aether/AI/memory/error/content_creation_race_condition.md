# Error Report: Content Creation Race Condition

## Issue Description
Users reported receiving a **"409 Conflict: Article with this title already exists"** error when attempting to publish a newly written article. This occurred even when the article title was unique and had not been previously created by the user. Visually, the article remained in the editor, but the "Publish" action failed.

## Root Cause Analysis
The issue was identified as a **Race Condition** between the **Auto-Save** mechanism and the **Publish** action in `EditorView.vue`.

1.  **Duplicate Logic**: The "Create Article" logic (HTTP POST) was implemented in *two* separate places:
    *   `saveDraft` (Auto-Save): Directly called `axios.post('/api/content')`.
    *   `executePublish` (Publish): Directly called `axios.post('/api/content')`.

2.  **Bypassed State Lock**: The global `useContent` composable maintains an `isSaving` state (boolean) to track ongoing network requests. However, `saveDraft`'s direct axios call **bypassed** this composable.
    *   Auto-Save would trigger a POST request.
    *   `isSaving` remained `false` (because `useContent` wasn't used).

3.  **The Race**:
    *   User types -> Auto-save triggers (Request A).
    *   User immediately clicks "Publish".
    *   `executePublish` checks `isSaving`. It sees `false` (incorrectly).
    *   `executePublish` triggers its own POST request (Request B).
    *   **Result**: Request A creates the article (ID=1). Request B tries to create the *same* article (ID=2) with the same title, triggering a Unique Constraint Violation (409 Conflict) in the database.

## Resolution
The architecture was refactored to enforce **Centralized State Management** for all content creation operations.

1.  **Updated `useContent.ts`**:
    *   Added a `create(payload)` method.
    *   This method strictly manages `isSaving.value = true` before the request and `false` after.

2.  **Refactored `EditorView.vue`**:
    *   Removed all direct `axios.post('/api/content')` calls.
    *   Replaced them with `await create(payload)`.

## Functional Flow After/Fix
1.  **Auto-Save Trigger**: Calls `create()`. `isSaving` becomes `true`.
2.  **User Clicks Publish**: Calls `executePublish`.
3.  **Lock Check**: `executePublish` checks `isSaving`. It sees `true`.
4.  **Wait**: It `awaits` until `isSaving` becomes `false`.
5.  **Proceed**: Once Auto-Save finishes (and sets `draftId`), `executePublish` sees the article now exists (`draftId` is set) and performs an **Update** (PUT) instead of a Create (POST).

## Regression Testing
*   **Scenario 1 (Fast Publish)**: Type "New Title", immediately click Publish.
    *   *Expected*: Spinner shows "Saving...", then "Publishing...", then success. No 409 error.
*   **Scenario 2 (Auto-Save then Publish)**: Type "New Title", wait 2s (Auto-save), then click Publish.
    *   *Expected*: Auto-save succeeds (Draft created). Publish updates status to "Published". Success.

## Prevention Checklist
*   [ ] **Never** use direct `axios` calls for mutating state (POST/PUT/DELETE) in complex views; always use a Composable.
*   [ ] **Always** implement a global `isSaving` or `isLoading` lock for potentially concurrent operations.
*   [ ] **Review** all `await` chains to ensure no "fire-and-forget" promises are leaving critical state unprotected.
