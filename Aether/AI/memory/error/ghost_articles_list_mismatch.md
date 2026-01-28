# Error Report: Ghost Articles (List vs Get Mismatch)

## 1. Issue Description
**Symptom**: User sees an article title in the main feed/list, but clicking it results in a "Signal Lost" (404) or broken page.
**Severity**: Medium (Data Inconsistency / UX Confusion).
**Status**: Resolved.

## 2. Root Cause
1.  **Repository Logic Mismatch**:
    - `ArticleRepository::list`: Performed a `LeftJoin` on `nodes` and `article_details`. It iterated through results, and if `article_details` was missing (e.g., due to a failed transaction or bad seed data), it defaulted to returning a generic `ContentItem::Node`.
    - `ArticleRepository::find_by_id`: Correctly identified that for type `Article`, `article_details` is mandatory. It returned `ContentItem::Node` (without body) if details were missing.
2.  **Frontend Expectation**:
    - The Main List view renders `ContentItem::Node` happily (it only needs Title/Author).
    - The Detail View (`ReadView`) expects a full `Article` with `body`. When it received a `Node` (or 404), it failed to render.

## 3. Resolution
### 3.1 Backend Filtering
We enforced stricter integrity checks in the Repository layer:

- **`list`**: Now explicitly checks: `if n.type == "Article" && details == None { continue; }`. "Ghost" articles are now hidden from the feed.
- **`find_by_id`**: Now explicitly checks: `if n.type == "Article" && details == None { return None; }`. Accessing a ghost article ID directly now correctly returns `404 Not Found`.

## 4. Verification
- **Manual Verification**:
    1. Check Main Feed: "Ghost" untitled/broken articles should disappear.
    2. Check Direct Link: Accessing a known broken ID should show the clean "Signal Lost" UI.
