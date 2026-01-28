# Error Report: Auto-Save Refresh/Redirect Loop & "Phantom" Publishing

## 1. Issue Description
**Symptom**: Users reported that while typing a title in the Editor, the page would suddenly "refresh", and the article would appear to be "Published" or saved as a different entity without their explicit consent.
**Severity**: High (UX Critical). It caused user confusion and fear of data integrity loss.
**Status**: Resolved.

## 2. Context
- **Component**: Frontend (`EditorView.vue`).
- **Trigger**: Typing a title that *already exists* in the database.
- **Mechanism**: The `useDebounceFn` auto-save triggers every 2 seconds.

## 3. Root Cause Analysis
The issue was a "Correctness vs. UX" conflict in the `saveDraft` function:

1.  **Backend Behavior**: When `POST /api/content` is called with a duplicate title, the Backend correctly returns `409 Conflict` and the ID of the *existing* article.
2.  **Frontend Logic (The Bug)**:
    ```javascript
    // OLD BROKEN LOGIC
    if (err.response.status === 409) {
        const existingId = err.response.data.id;
        // AGGRESSIVE REDIRECT
        router.replace({ name: 'editor', params: { id: existingId } }); 
        draftId.value = existingId;
    }
    ```
3.  **The Loop**:
    - User types "My Great Post" (which already exists).
    - Auto-save fires -> 409 Conflict.
    - Frontend *immediately* redirects to the *existing* "My Great Post".
    - If the existing post was `Published`, the UI loads it.
    - The user sees the page "flash" and suddenly sees "Status: Saved" (because the fetch succeeded). 
    - **Result**: The user thinks their *new* draft was just forced-published, when in reality they were just teleported to an old published post.

## 4. Resolution
We decoupled the "Duplicate Detection" from "Navigation".

### 4.1 Frontend Fix (`EditorView.vue`)
- **Removed Redirect**: We no longer auto-redirect on 409.
- **Warning Toast**: We show a toaster warning: *"Title exists. Auto-save failed. Change title to save."*
- **Explicit Status UI**: We split the generic "Saved" indicator into:
    - `Draft Saved`
    - `Published`
    - `Editing Published`

### 4.2 Code Change
```javascript
// NEW LOGIC
if (err.response.status === 409) {
    // Just warn the user. Let them decide what to do (rename or manually leave).
    MessagePlugin.warning('Title exists. Auto-save failed. Change title to save.', 3000);
}
```

## 5. Verification
- **Script**: `backend/debug_duplicate_title_flow.sh`
- **Steps**:
    1. Create "Article A".
    2. Try to create "Article B" with same title.
    3. Verify Backend returns `409 Conflict`.
    4. (Manual) Verify Frontend stays on "Start writing..." screen and shows warning.

## 6. Lessons Learned
- **Never Auto-Navigate on Error**: Interrupting user flow during data entry is dangerous.
- **Clarify UI State**: "Saved" is too ambiguous when multiple states (Draft vs Public) exist.
