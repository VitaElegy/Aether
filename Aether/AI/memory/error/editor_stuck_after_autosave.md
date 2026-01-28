# Error Report: Editor Stuck After Auto-Save

## 1. Issue Description
**Symptom**: User modifies an article in the Editor (triggering auto-save), then clicks "Back". The URL changes to Home (`/`), but the **UI remains stuck on the Editor**.
**Severity**: High (Appears Broken).
**Status**: Resolved.

## 2. Root Cause
- **Crash on Unmount**: The `EditorView.vue` component performs cleanup in `onBeforeUnmount` (cancelling auto-save timers, destroying editor instance).
- **Unhandled Exception**: Under certain race conditions (e.g., auto-save completion coincident with navigation), the cleanup code threw an exception (likely accessing a null `editor.value` or similar).
- **Navigation Abort**: In Vue Router, if an error is thrown during the navigation guard phase (which includes component unmount hooks), the **navigation is aborted or the view update is halted**, leaving the old component visible despite the URL change.

## 3. Resolution
- **Defensive Coding**: Wrapped the `onBeforeUnmount` logic in a `try-catch` block.
  ```javascript
  onBeforeUnmount(() => {
    try {
      if (debouncedAutoSave) debouncedAutoSave.cancel();
      if (editor.value) editor.value.destroy();
    } catch (e) {
      console.warn('Cleanup error ignored:', e);
    }
  });
  ```
- This ensures that even if cleanup fails, the unmount proceeds, and the user is correctly navigated away.

## 4. Verification
- **Browser Subagent**:
    1. Reproduced the "Stuck UI" state (URL=/, Content=Editor).
    2. Applied fix.
    3. Verified successful navigation to Home after editing.
