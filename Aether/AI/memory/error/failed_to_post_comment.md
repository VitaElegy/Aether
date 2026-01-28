# Failed to Post Comment Error (2026-01-07)

## 1. Error Description
Users were unable to post comments on articles.
- **Symptom**: Clicking "Post Comment" resulted in a failure.
- **Frontend Error**: The UI initially showed a generic "Failed to post comment. You probably need to log in" alert. After debugging improvements, it returned a 500 Internal Server Error.
- **Backend Error**: The backend log (initially swallowed) would have shown a SQL execution error regarding a missing column.

## 2. Root Cause Analysis
The issue was caused by a combination of code-schema mismatch and a deployment zombie process.

### A. Code-Schema Mismatch (Primary Cause)
- **The Mismatch**: The backend code in `postgres.rs` (repository layer) and `comment.rs` (entity layer) was trying to interact with a column named `content_id`.
- **The Reality**: The actual SQLite database schema (defined in `main.rs`) had evolved. The `content_id` column had been removed/replaced by a polymorphic `target_id` column to support comments on multiple entity types (Memos, etc.).
- **The Result**: When `ActiveModel` tried to save a comment, it generated SQL including `content_id`, which the database rejected with "column does not exist" (or similar SQL error), leading to a 500 panic.

### B. Zombie Backend Process (Secondary Cause)
- **The Mismatch**: Even after the code was patched to remove `content_id`, the browser still reported errors.
- **The Reality**: An old instance of the backend process (PID 52859) was still running and holding port 3000. It was running the *old* binary code.
- **The Result**: The new, fixed binary could not bind to port 3000 (likely failed silently or wasn't checked carefully), so the browser continued talking to the old, broken backend.

## 3. Solution

### Step 1: Fix the Code
- Modified `backend/src/infrastructure/persistence/postgres.rs` to remove the assignment of `content_id` in the `ActiveModel` conversion.
- Updated `backend/src/infrastructure/persistence/entities/comment.rs` to remove the `content_id` field from the struct.
- Updated `backend/src/interface/api/comment.rs` to ensure it returns explicit error messages (`e.to_string()`) instead of generic 500s, aiding future debugging.

### Step 2: Fix the Frontend
- Patched `CommentSection.vue` to fix a syntax error (missing brace).
- Enhanced `CommentSection.vue` to alert the actual error message from the backend response, rather than a generic "Login" message.
- Added explicit `Authorization` headers to the axios request as a safeguard.

### Step 3: Fix the Deployment
- Identified the zombie process using `lsof -i :3000`.
- Killed the process (`kill -9 52859`).
- Restarted the backend using the correct environment variables (`DATABASE_URL="sqlite://aether.db?mode=rwc"`).

## 4. Verification
- Logged in as user `elegy`.
- Posted a comment "Hello from elegy".
- Verified the comment appeared instantly in the UI.
