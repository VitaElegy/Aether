# Failed to Load Article History & Diff Errors

## 1. Failed to Load Version Data
### Problem Description
When accessing the Article History's detail view (`/content/:id/version/:v`), the UI displayed an error toast: **"Failed to load version data"**. The version details were not rendering.

### Root Cause Analysis
- **Missing Data**: The frontend expected the API response to contain a `body` field with the article content.
- **Backend Implementation**: The `ContentVersionSnapshot` struct in `models.rs` and the `ArticleRepository::get_version` method did not include `body`. They only returned metadata (title, reason, date).

### Solution
- **Updated Model**: Added `pub body: Option<ContentBody>` to `ContentVersionSnapshot`.
- **Updated Repository**: Modified `ArticleRepository::get_version` to deserialize and map the `body` from the database to the snapshot struct.

---

## 2. Blank Review Changes (Diff) Interface
### Problem Description
When clicking "Diff" in the history view, the "Review Changes" card appeared but was completely blank/white, even though the API returned a 200 OK.

### Root Cause Analysis
- **Format Mismatch**:
    - **Backend**: The `ArticleRepository::get_diff` method (using the `similar` crate) was returning a raw Unified Diff **String** (e.g., `--- \n +++ \n @@ ...`).
    - **Frontend**: The `DiffViewer.vue` component expected a **Structured JSON Array** of changes (e.g., `[{ "tag": "Insert", "value": "..." }]`).
- The frontend silently failed to render the string as an array.

### Solution
- **Refactored Backend**: Updated `ArticleRepository::get_diff` and `ContentDiff` struct to return `Vec<DiffChange>` instead of `String`.
- **Structured Data**: The backend now iterates through the diff chunks and constructs a JSON array explicitly matching the frontend's expectations.

---

## 3. Invalid Version ID (SemVer Error)
### Problem Description
When toggling the Diff view, the backend returned a **500 Internal Server Error** with message: `Query Error: Invalid version number`.

### Root Cause Analysis
- **Frontend Logic**: The helper function `formatSemVer` in `VersionView.vue` was converting integer version numbers (e.g., `1`) into SemVer strings (e.g., `0.0.1`) before sending them to the API as the "previous version" ID.
- **Backend Expectation**: The `ArticleRepository` expects simple integer string identifiers (e.g., `"1"`) and failed to parse `"0.0.X"` as an integer.

### Solution
- **Frontend Fix**: logic in `VersionView.vue` was updated to stop using `formatSemVer` for API calls. It now sends simple integer strings (e.g., `(currentV - 1).toString()`).

---

## Verification
- **Scripts**:
    - `debug_version_detail.sh`: Verified `body` field is present in version responses.
    - `debug_diff.sh`: Verified diff endpoint returns structured JSON (`tag`, `value`) and valid changes.
- **Manual**:
    - Confirmed History list loads.
    - Confirmed Detail view renders content.
    - Confirmed Diff view renders colored insertions/deletions.
