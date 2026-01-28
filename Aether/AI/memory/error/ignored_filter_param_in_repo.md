# Error: Ignored Filter Parameter in Repository List Method

## Incident Description
**Date:** 2026-01-11
**Error:** `ArticleRepository.list` returned all published articles (global feed) instead of filtering by `author_id` when requested.
**Impact:** User Profile pages displayed all articles from the platform, violating data isolation expectation (though not strictly a security leak since the articles were Public, it broke the profile view's purpose).

## Root Cause
The `list` method in `backend/src/infrastructure/persistence/repositories/article.rs` accepted an `author_id` parameter (and others) but completely ignored it in the query construction.

## Steps to Reproduce
1. Create two users: User A and User B.
2. User A publishes an article.
3. User B publishes an article.
4. Call `GET /api/content?author_id={User_A_ID}`.
5. **Expected:** JSON array containing only User A's article.
6. **Actual:** JSON array containing BOTH articles (Global Feed).

## Resolution
Modified `ArticleRepository::list` to strictly apply the filter if the `Option<UserId>` is `Some`.

```rust
// Before
async fn list(&self, _viewer_id: Option<UserId>, _author_id: Option<UserId>, ...) {
    let mut query = node::Entity::find()...;
    // ... no usage of _author_id
}

// After
async fn list(&self, _viewer_id: Option<UserId>, author_id: Option<UserId>, ...) {
    let mut query = node::Entity::find()...;
    
    if let Some(uid) = author_id {
        query = query.filter(node::Column::AuthorId.eq(uid.0));
    }
    // ...
}
```

## Prevention
*   **Code Review:** Ensure all arguments in Repository methods are either used or explicitly marked as unused with a clear reason.
*   **Testing:** Add specific integration tests for filtering logic (ensure `list(author_id=X)` returns count matching X, not total count).
