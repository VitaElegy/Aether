# Failed to Associate Article to Knowledge Base Error (2026-01-08)

## 1. Error Description
When users attempted to create an article within a Knowledge Base (or move an article to one), the association was lost.
- **Symptom**: The article was created successfully but appeared as a standalone article in the "Self Space" list, not inside the specific Knowledge Base.
- **Backend Condition**: The `nodes` table lacked any column to store the `knowledge_base_id`, so it was impossible to persist the relationship.

## 2. Root Cause Analysis
The backend data model was incomplete regarding the "Knowledge Base" feature hierarchy.
- **Database Schema**: The `nodes` table (which stores the core metadata for all content) did not have a `knowledge_base_id` column.
- **Domain Model**: The `Node` struct in `models.rs` was missing the `knowledge_base_id` field.
- **API Layer**: The `CreateContentRequest` payload definition in `content.rs` did not include the `knowledge_base_id` field, so even if the frontend sent it, it was ignored during deserialization.
- **Persistence Layer**: The repository mappers for `Article`, `Vocabulary`, and `Memo` were not configured to map this field between the domain and database.

## 3. Solution

### Step 1: Schema Migration
- Updated `main.rs` to include a safe schema migration for **SQLite** (handling `AddressInUse` and `Duplicate Column` scenarios gracefully).
- Added `ALTER TABLE nodes ADD COLUMN knowledge_base_id UUID ...`.

### Step 2: Domain & Entity Updates
- Added `knowledge_base_id: Option<Uuid>` to:
    - `Node` struct (`domain/models.rs`).
    - `node::Model` and `node::ActiveModel` (`persistence/entities/node.rs`).
- Defined the SeaORM relationship `BelongsTo <-> HasMany` between `nodes` and `knowledge_bases`.

### Step 3: API & Repository Implementation
- Updated `CreateContentRequest` in `content.rs` to accept `knowledge_base_id`.
- Updated `ArticleRepository`, `MemoRepository`, and `NodeRepository` to:
    - Extract the ID from the payload.
    - Persist it to the `nodes` table during `save`.
    - Retrieve and map it back to the Domain Model during `find/list`.

## 4. Verification
- **Automated**: Ran `debug_kb_article.sh`:
    - REGISTER -> LOGIN -> CREATE KB -> CREATE ARTICLE (with `kb_id`).
    - Validated that the created article's JSON response contained the correct `knowledge_base_id`.
