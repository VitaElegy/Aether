# Refactor V1 Compilation Issues

## Symptoms
Refactoring the core architecture to use modular `Repositories` and `AppState` triggered widespread compilation errors:
1. **Axum Handler Type Mismatches**:
   ```text
   the trait bound `fn... {handler}: Handler<_, ...>` is not satisfied
   expected `Router<AppState>`, found `Router<Arc<dyn KnowledgeBaseRepository>>`
   ```
2. **Missing Dependencies/Modules**:
   - `md5` crate used without adding to Cargo.toml.
   - `aether_backend` crate not found in `src/bin` scripts.
3. **Private Struct Imports**: `UserId` and `AuthClaims` were not accessible where needed.
4. **Entity Field Mismatches**: `Vocabulary` missing `image_url` context, `Comment` entity referring to `target_type`.

## Root Cause
1. **Axum State Extraction**: Axum's `State` extractor expects the exact type defined in the Router (`AppState`). Passing `Arc<dyn Repo>` directly works only if the Router is typed for that specific component (which it wasn't, `main.rs` merges them into an `AppState` router), OR if `FromRef` is implemented perfectly and ambiguous types are avoided. The `AuthenticatedUser` extractor also depends on `AppState` for its own `FromRef` logic (Authorizer), causing conflicts when mixing State types.
2. **Entity Evolution**: The `Node` refactor introduced new fields (`image_url`) in the Domain Models that were not immediately reflected in the SeaORM entities or API payloads.
3. **Dependency Drift**: Usage of `md5` for hashing content versions was a legacy import that wasn't tracked in `Cargo.toml`.

## Resolution
1. **Standardize State**: Updated *all* API handlers to use `State<AppState>` instead of granular `State<Arc<dyn Repository>>`. This ensures satisfying the `Router<AppState>` requirement and allows `AuthenticatedUser` to extract `AuthService` from the same `AppState` without type confusion.
   - *Example Change*: `State(repo): State<Arc<dyn ContentRepo>>` -> `State(state): State<AppState>`.
2. **Fix Entities**: 
   - Added `context_sentence` and `image_url` to `Vocabulary` struct.
   - Updated `Comment` entity to use polymorphic `target_id` (Uuid) without a rigid `target_type` column constraint in SeaORM entity (handled by logic).
3. **Remove Unused Deps**: Replaced `md5` with `Uuid::new_v4().simple()` for version hashing to avoid adding a dependency for a non-critical feature.
4. **Public Exports**: Exposed `UserId` via `domain::models` to fix visibility.
