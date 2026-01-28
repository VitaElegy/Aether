# Failed to Publish Article Error (2026-01-08)

## 1. Error Description
Users were unable to publish new articles.
- **Symptom**: Clicking "Commit & Publish" in the editor resulted in a failure.
- **Frontend Error**: The UI displayed "Failed to publish" toast. Console showed 500 Internal Server Error.
- **Backend Error**: `sqlite3` execution error: `(code: 787) FOREIGN KEY constraint failed`.

## 2. Root Cause Analysis
The issue was caused by a configuration logic error in the backend's startup sequence, combined with client-side session persistence.

### A. Database Reset Loop
- **The Mechanism**: `backend/src/main.rs` contained a `DROP TABLE IF EXISTS ...` block that executed on *every* server start. This wiped all users and data, resetting the `users` table to only the default admin.
- **The Trigger**: The user restarted the backend service.

### B. Zombie Session Token
- **The State**: The frontend (Browser) stored a JWT token in `localStorage`. This token contained the User ID (`sub`) of a user created in the *previous* run.
- **The Conflict**: When the user tried to publish, the backend extracted the ID from the token (e.g., `user_A`) and attempted to insert it as the `author_id` in the `nodes` table.
- **The Failure**: Since the database had been wiped, `user_A` no longer existed in the `users` table. The `FOREIGN KEY(author_id) REFERENCES users(id)` constraint failed immediately.

## 3. Solution

### Step 1: Enable Persistence
- Modified `backend/src/main.rs`:
    - Commented out the `DROP TABLE` statements. The database now persists data across restarts, preventing the "vanishing user" scenario.

### Step 2: Validate User Existence
- Modified `backend/src/interface/api/auth.rs`:
    - Updated `AuthenticatedUser::from_request_parts` and `MaybeAuthenticatedUser` to explicitly check if the ID in the JWT `sub` claim exists in the database.
    - **Logic**: If the ID is missing (e.g., wiped DB), return `401 Unauthorized` instead of proceeding. This forces the frontend to log out, preventing the 500 Server Error loop and guiding the user to re-register.

## 4. Verification
- **Automated**: Ran `debug_publish.sh` script to verify API directly.
    - Verified `POST /api/content` returns `201 Created` for a valid, newly registered user.
- **Manual (Browser)**:
    - Registered a new user `ui_tester`.
    - Published article "UI Test Article".
    - Verified success message and article appearance.
