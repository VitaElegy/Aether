# 🧠 Aether Knowledge Base: Lessons Learned

This file serves as the long-term memory for the AI Agent. Read this upon initialization to avoid repeating past mistakes.

## 2026-02-03: API & Deployment

### 1. Backend Process Staleness
- **Symptom**: `400 Bad Request` or "Query too short" errors persist even after code modification.
- **Cause**: The backend binary/container does not hot-reload Rust code changes.
- **Solution**: You **MUST** restart the backend service (`cargo run` or `docker restart backend`) after *any* change to `.rs` files.
- **Protocol**: Since you have **Autonomous Control**, do not ask the user. Just restart it.

### 2. User User Interface
- **UX**: Users prefer "Search on Type/Enter" or "Active Search". However, for lists like "User Management", **Lazy Loading (Auto-load initial)** is preferred over an empty state.
- **Refactor**: If changing search logic, ensure the backend supports `limit` and `offset` for pagination.

### 3. Frontend Image Handling
- **Resilience**: Never trust URL strings alone. Always implement an `@error` handler to catch 404s/Corruption and switch to a UI fallback (Icon/Placeholder).
- **Paths**: Components must handle both full URLs (`http`) and relative paths (`/uploads/...`). Use a helper function or computed property.
