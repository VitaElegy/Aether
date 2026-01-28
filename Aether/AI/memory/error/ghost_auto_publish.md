# Ghost Auto-Publish

## Root Cause
- The Backend `create_content_handler` and `update_content_handler` defaulted `status` to `"Published"` if the `status` field was missing in the request payload (`unwrap_or("Published")`).
- The Frontend `EditorView.vue`'s auto-save logic generally sends the status, but if a race condition (e.g. `onMounted` vs `watch`) or a partial update scenario occurred where `status` was undefined/null, the backend would silently upgrade the article to "Published".

## Resolution
- **Backend `create_content_handler`**: Changed default from `"Published"` to `"Draft"`.
- **Backend `update_content_handler`**: Changed logic to fallback to `existing.status` if `payload.status` is missing. It only changes status if explicitly requested.

## Verification
- Created `backend/scripts/debug/debug_auto_publish.sh`.
- Verified that `POST /api/content` without `status` creates a **Draft**.
- Verified that `PUT /api/content/:id` without `status` preserves the **Draft** status.
