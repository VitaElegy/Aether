# Error Report: Auto-Save Unpublishes Article

## 1. Issue Description
**Symptom**: User edits a "Published" article in the Editor. Upon auto-save, the article's status silently reverts to "Draft". This causes it to potentially disappear from public feeds or confuse the user ("Inexplicably publishes/refreshes" - possibly misinterpreted as "Unpublishes").
**Severity**: High (Data Integrity / UX).
**Status**: Resolved.

## 2. Root Cause
- **Frontend Logic Error**: In `EditorView.vue`, the `saveDraft` function (triggered by auto-save) had a hardcoded payload:
  ```javascript
  const payload = {
      // ...
      status: 'Draft', // HARDCODED!
      // ...
  };
  ```
- This meant *any* auto-save, regardless of the article's current state, would force it back to Draft status.

## 3. Resolution
- **Respect Form State**: Updated `saveDraft` to use `form.status`:
  ```javascript
  status: form.status, // Respect current status
  ```
- Now, if an article is "Published", editing it preserves the "Published" status (Live Edit). If it is "Draft", it remains "Draft".

## 4. Verification
- **Script**: `backend/debug_edit_published.sh`
- **Steps**:
    1. Create Article with status="Published".
    2. Verify status is "Published".
    3. Update Article (simulate Editor auto-save) with status="Published".
    4. Verify status remains "Published".
    5. Update Article with status="Draft".
    6. Verify status becomes "Draft".
