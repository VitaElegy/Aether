# Error Report: Phantom Refresh on Public Page

## 1. Issue Description
**Symptom**: User observes "inexplicable refreshes" or UI glitches ("ESTABLISHING UPLINK...") when hovering over the right side of an article on the Public Page (`ReadView`), especially after editing.
**Severity**: Medium (UX Disturbance).
**Status**: Resolved.

## 2. Root Cause
1.  **Leaked Component Logic**: The `EditorView.vue` uses `useDebounceFn` (from `@vueuse`) for auto-saving with a 2-second delay.
2.  **Unmount Failure**: When navigating away from `EditorView` to `ReadView`, the *pending* debounce timer was NOT cancelled.
3.  **Background Execution**: 2 seconds after leaving the editor, the `saveDraft` function would fire in the background while the user was on the Public Page.
4.  **Side Effects**:
    - The detailed browser investigation revealed console errors: `Auto-save failed: AxiosError`.
    - This background network activity or state checking seemingly interacted with the global app state (loading indicators or router meta) causing the "refresh" perception.
    - Hovering might have coincidentally aligned with the timer or triggered layout shifts that made the glitch noticeable.

## 3. Resolution
- **Explicit Cleanup**: Added `debouncedAutoSave.cancel()` to the `onBeforeUnmount` hook in `EditorView.vue`.
  ```javascript
  onBeforeUnmount(() => {
    debouncedAutoSave.cancel(); // Stop pending saves immediately
    editor.value?.destroy();
  });
  ```

## 4. Verification
- **Browser Subagent**: Reproduced the issue (finding logs of editor errors on the public page) and then verified the fix (clean logs, stable UI) after applying the cleanup code.
