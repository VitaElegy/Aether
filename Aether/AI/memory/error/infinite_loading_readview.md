# Error Report: Infinite Loading / Template Crash on ReadView

## 1. Issue Description
**Symptom**: User opening an article (specifically created via API scripts or older versions) sees "ESTABLISHING UPLINK..." indefinitely.
**Severity**: High (Content Inaccessible).
**Status**: Resolved.

## 2. Root Cause
1.  **Backend Data Evolution**:
    - Old Schema: `body` was always `{ "type": "Markdown", "data": "..." }`.
    - New Schema (after recent refactors): `body` is `JSONB`. Some scripts (like `debug_publish_rules.sh`) saved it as a simple string `"Content..."` or `{ "body": "Content" }` without the nested `data` field.
2.  **Frontend Fragility (`ReadView.vue`)**:
    - The `loadData` function blindly accessed `data.body.type` and `data.body.data`.
    - When `data.body` was a string, `data.body.type` was `undefined`.
    - Code logic `data.body.type || 'Markdown'` might have worked, but `data.body.data` on a string returns `undefined`.
    - Resulting `post` object had malformed fields.
    - **CRITICAL**: If `loadData` threw an error, `post` remained `null`.
    - The template attempted to render `post.title`.
    - Vue runtime crashed: `Cannot read properties of null (reading 'title')`.
    - This crash halted the component update, leaving the "Loading" div (v-if="loading") on screen indefinitely (or rather, the whole view froze).

## 3. Resolution
### 3.1 Robust Data Parsing
Updated `loadData` to handle both formats:
```javascript
type: typeof data.body === 'string' ? 'Markdown' : (data.body.type || 'Markdown'),
data: typeof data.body === 'string' 
    ? { content: data.body } 
    : (data.body.type === 'Markdown' ? { content: data.body.data } : data.body)
```

### 3.2 Error State UI
Added a specific `v-else-if="!post"` state to display "Signal Lost (404)" if data loading fails, preventing the template from trying to render null data and crashing.

## 4. Verification
- Manual verification required: Open an article created by `debug_duplicate_title_flow.sh` (which sets body as string).
- Expected result: Article renders content correctly instead of infinite loading.
