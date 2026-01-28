# Failed to Search (500 Internal Server Error)

## Problem Description
Upon implementing the search functionality, requests to `/api/search?q=...` failed with **500 Internal Server Error**.
Backend logs revealed the error: `Query Error: no column found for name: author_id`.

## Root Cause Analysis
The issue originated in the `ArticleRepository::search` method using `sea-orm`.
We used an optimization pattern:
```rust
let matching_nodes = node::Entity::find()
    .select_only() // <--- PROBLEM
    .column(node::Column::Id)
    // ... filters ...
    .all(&self.db).await?;
```
**Mechanism of Failure**:
- `select_only()` tells `sea-orm` to construct a `SELECT id FROM ...` query.
- However, we were retrieving the result into `node::Model` (implied by `node::Entity::find()`).
- `node::Model` has required fields (non-Option) like `author_id`, `type`, `title`.
- When `sea-orm` attempted to hydrate the `node::Model` from the result row (containing only `id`), it looked for the `author_id` column in the row, found it missing, and panicked/errored with "no column found".

## Solution
Removed the `select_only()` and `.column(Id)` restrictions. This allows `sea-orm` to fetch the full row (`SELECT * ...`). While slightly less efficient than fetching IDs only, it ensures that the `node::Model` can be safely hydrated without errors.

```rust
// Fixed Code
let matching_nodes = node::Entity::find()
    // .select_only() <--- REMOVED
    // .column(...)   <--- REMOVED
    .filter(node::Column::Type.eq("Article"))
    // ...
```

## Verification
- **Script**: `debug_search.sh`
- **Result**:
    - **Before**: `Hits for Title: ` (Empty/Failed) + 500 Log.
    - **After**: `Hits for Title: 1`, `Hits for Body: 1`, `Hits for Negative: 0`. Success.
