# Startup Panic & Build Failures

## Root Cause
1.  **Backend Panic**: The `main.rs` router composition merged `auth::router()` and `user::router()`. Both modules registered the route `GET /api/users/search`, causing Axum to panic at startup with "Overlapping method route".
2.  **Frontend Build Failure**: 
    - `tsconfig.node.json` used `moduleResolution: "Node"`, causing incompatibility with `vite`/`rollup` types (`parseAst`).
    - `EditorView.vue` had multiple syntax errors (duplicate object keys) and TypeScript type errors (missing properties on `localCache` assignment).
    - `UserProfileView.vue` called `knowledgeApi.list(uid)` with an argument, but the definition accepted zero arguments.

## Resolution
1.  **Backend**: Removed the duplicate `search_users_handler` and its route registration from `backend/src/interface/api/auth.rs`. The route remains correctly defined in `user.rs`.
2.  **Frontend**:
    - Updated `tsconfig.node.json` (and `tsconfig.json`) to use `"moduleResolution": "bundler"`.
    - Fixed `EditorView.vue` by removing duplicate keys and correctly handling `localCache` assignment (added missing `knowledge_base_id`, removed invalid `visibility/status`).
    - Fixed `UserProfileView.vue` to use a direct `axios` call with query parameters for filtering KBs by author.

## Verification
- Verified `cargo run --bin aether_backend` starts successfully without panic.
- Verified `npm run build` completes with Exit Code 0.
