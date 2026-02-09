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

## 2026-02-05: Database & Migrations

### 4. Database Migrations
- **Verification**: Always verify that a corresponding migration file exists when adding a new Entity. The existence of a Rust struct does not imply the table exists in the DB.
- **Foreign Keys**: Be careful with self-referencing FKs in copy-paste migrations. Ensure they point to the correct table (e.g., `vocab_details.root_id` -> `vocab_roots.id`, not `vocab_details.id`).

### 5. Frontend State Management
- **Draft Restoration**: When restoring a draft for an existing article, **NEVER skip loading the live article**.
    - **Risk**: Skipping `load()` results in missing metadata (permissions, author info) and relies on potentially incomplete draft data.
    - **Correct Pattern**: `await load(id)` -> Then `fetchDraft()` -> If draft exists, overlay draft properties onto the form state.

## 2026-02-07: Shared Content Architecture

### 6. Many-to-Many Content Sharing
- **Content Addressing**: When implementing shared content (like sentences used by multiple words), prefer deduplication by content (text) rather than forcing manual ID management.
    - **UX Benefit**: Users can simply type or paste text. The backend automatically links to an existing global record if the text matches, or creates a new one.
    - **Implementation**: In the repository `save()` method, check for existence by content before inserting.
- **SeaORM Nullability**: When migrating a column from Required to Optional (Nullable), ensure `ActiveModel` sets the value to `Set(None)` to explicitly write NULL. If the field is deprecated but kept for legacy, ensure the read logic falls back gracefully (e.g., `global_sentence.text` OR `legacy_sentence_column`).

## 2026-02-07: API Governance

### 7. Documentation Standards
- **Format**: **OpenAPI (Swagger)** is the single source of truth.
- **Methodology**: **Code-First with `utoipa`**. Do not manually edit YAML/JSON specs. Annotate Rust controllers with macros.
- **Error Handling**: Define **Detailed Business Errors** in the spec (e.g., `TitleExists`), not just generic HTTP codes.
- **Frontend**: Generate **TypeScript Types (`.d.ts`)** only. Do not generate full API clients; keep manual control over `axios` calls for flexibility.
