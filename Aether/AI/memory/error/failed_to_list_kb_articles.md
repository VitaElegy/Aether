# Failed to List Knowledge Base Articles Error (2026-01-08)

## 1. Error Description
After fixing the article *association* (so they were correctly correctly persisted with a `knowledge_base_id`), the articles still did not appear in the Knowledge Base detail view in the UI.
- **Symptom**: The "Articles" list in a specific Knowledge Base page remained empty, even though the database records showed they were linked to that KB.
- **Backend Condition**: The API returned a global list or filtered incorrectly, failing to return the specific entries.

## 2. Root Cause Analysis

### Primary Cause: Missing Filter Capability
The backend's `list_content` capability was designed for the global feed and did not support filtering by `knowledge_base_id`.
- **API Layer**: `ListParams` in `content.rs` lacked the `knowledge_base_id` field.
- **Repository Layer**: `ArticleRepository::list` did not accept or use a `knowledge_base_id` argument to refine the SQL query.
- **Frontend Layer**: The API client (`contentApi.list`) wasn't passing the ID.

### Secondary Cause: Data Consistency (Case Sensitivity)
During the fix of the Primary Cause, a second issue was discovered:
- **Inconsistency**: The legacy data and some parts of the system used "Article" (Title Case) for the `type` column. The new repository save logic I introduced initially used "article" (lowercase).
- **Impact**: The filter `node::Column::Type.eq("Article")` failed to match the new records stored as "article", causing the list to return empty results even when the association filter was working.

## 3. Solution

### Step 1: Enable Filtering
- Updated `ListParams` struct to include `knowledge_base_id: Option<Uuid>`.
- Updated `ArticleRepository::list` signature and implementation to filter by `knowledge_base_id` if provided:
  ```rust
  if let Some(kb_id) = knowledge_base_id {
      query = query.filter(node::Column::KnowledgeBaseId.eq(kb_id));
  }
  ```
- Updated Frontend `contentApi.list` to accept and pass this parameter.

### Step 2: Standardize Data Format
- Standardized the `NodeType` string to **"Article"** (Title Case) across the application.
- Updated `NodeRepository` and `ArticleRepository` `save` methods to explicitly write "Article".
- Updated `ArticleRepository` `list` filter to match "Article".

## 4. Verification
- **Automated**: `debug_kb_list.sh` script confirmed that querying with `knowledge_base_id` returns *only* the specific article linked to that KB.
- **Manual**: Browser verification confirmed that creating an article within a KB immediately displays it in the list.
