# Error: Frontend Publish Validation (422 Unprocessable Entity)

## Symptoms
- **User Impact**: Clicking "Publish" results in a generic "Failed to publish" error message.
- **Logs**: Browser console shows a 422 response from `PUT /api/content/:id`.
- **Payload Inspection**: The JSON payload sent to the backend had `visibility: undefined` or `knowledge_base_id: ""`.

## Root Cause
- **Frontend State Initialization**:
  - When loading a new draft or switching modes, the reactive `form` object in `EditorView.vue` did not have a default value for `visibility` if the backend didn't provide one (e.g., for a fresh, empty draft).
  - The backend `CreateContentRequest` struct (and `Update`) uses strict deserialization and failed when `visibility` was missing or invalid.
  - `knowledge_base_id` was being sent as an empty string `""` when no KB was selected, but the backend expects `null` or valid UUID.

## Resolution
- **Fix**: Updated `executePublish` in `EditorView.vue` to sanitize the payload before sending:
  ```typescript
  visibility: form.visibility || 'Public',
  knowledge_base_id: form.knowledge_base_id || null, 
  ```

## Prevention
- **Action Item**: Implement Zod or similar schema validation on the frontend to catch invalid payloads before network requests.
- **Backend**: Consider making `visibility` an `Option<String>` with a default of "Private" or "Public" to be more resilient.
