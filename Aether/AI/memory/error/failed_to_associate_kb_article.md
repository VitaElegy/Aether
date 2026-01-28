# Failed to Associate Article with Knowledge Base

## Problem Description
When attempting to move an existing article to a Knowledge Base (or associating one upon creation) via the API or Frontend, the association would silently fail (backend) or return a 422 error (frontend).

## Root Cause Analysis

### 1. Backend Persistence Failure (Silent Failure)
The `ArticleRepository::save` and `NodeRepository::save` methods use an "Upsert" (Insert on Conflict) strategy to handle both creation and updates.
However, the `knowledge_base_id` column was **missing** from the `.update_columns([...])` list in the `OnConflict` logic.
**Result**: When updating an existing node, any change to `knowledge_base_id` was ignored by the database driver, despite the API handler correctly passing the value to the repository.

### 2. Frontend 422 Unprocessable Entity (API Error)
The frontend `EditorView.vue` bound the `knowledge_base_id` to a `<select>` element. When no KB was selected, the value was an empty string `""`.
The Rust backend uses `serde` to deserialize the JSON payload. The field was defined as `Option<Uuid>`.
**Result**: `serde` attempted to parse `""` as a UUID, causing a deserialization error ("invalid length: expected length 32"). `Option<Uuid>` strictly expects `null` or a valid UUID string, not an empty string.

## Solution

### 1. Backend Fix
Updated `backend/src/infrastructure/persistence/repositories/article.rs` and `node.rs` to explicitly include `node::Column::KnowledgeBaseId` in the `update_columns` list.

```rust
// Before
.update_columns([node::Column::Title, node::Column::UpdatedAt, ...])

// After
.update_columns([
    node::Column::Title, 
    node::Column::UpdatedAt, 
    node::Column::KnowledgeBaseId, // Added
    ...
])
```

### 2. Frontend Fix
Updated `frontend/src/views/EditorView.vue` to sanitize the payload before sending it to the API. We now explicitly convert falsy values (like `""`) to `null`.

```typescript
// Payload construction
knowledge_base_id: form.knowledge_base_id || null, // Sanitize empty string to null
```

## detailed Verification
- **Backend Verification**: Created a shell script `verify_fix.sh` that created a KB and an Article with that KB's ID, then verified via the list API that the article was correctly associated.
- **Frontend Verification**: Manual testing via Browser Subagent confirmed that selecting a "Debug KB" from the dropdown and publishing resulted in a successful 200 OK (and visible association in the UI), whereas previously it threw a 422 error.
