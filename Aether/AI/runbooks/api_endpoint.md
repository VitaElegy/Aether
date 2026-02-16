# Runbook: Adding an API Endpoint

> **Prevents**: Route conflicts, auth bypass, DTO mismatches
> **Error Examples**: `Startup Panic (Duplicate Route)`, `Invalid Version ID (500)`

---

## Phase 1: Contract Design

- [ ] Define Request DTO in `backend/src/models/`
- [ ] Define Response DTO in `backend/src/models/`
- [ ] Add `#[derive(Serialize, Deserialize, ToSchema)]` to DTOs
- [ ] **CRITICAL**: Match ID format with frontend (String vs UUID)

**Common Pitfall**: Frontend sends `"123"`, Backend expects `Uuid::parse()` → 400 Error

---

## Phase 2: Handler Implementation

- [ ] Create handler in `backend/src/handlers/{domain}.rs`
- [ ] Add Utoipa annotation:
  ```rust
  #[utoipa::path(
      post,
      path = "/api/your-resource",
      request_body = YourRequest,
      responses((status = 200, body = YourResponse))
  )]
  ```
- [ ] Return `Result<Json<T>, AppError>` — never panic
- [ ] Use specific error variants (`AppError::NotFound`, `AppError::Conflict`)

---

## Phase 3: Route Registration

- [ ] **SEARCH** for duplicate routes before adding:
  ```bash
  grep -r '"/api/your-path"' backend/src/
  ```
- [ ] Add route to router:
  ```rust
  .route("/api/your-resource", post(your_handler))
  ```
- [ ] Apply auth middleware if needed:
  ```rust
  .route_layer(middleware::from_fn(require_auth))
  ```

**Gate**: If grep finds duplicates, resolve before proceeding.

---

## Phase 4: Frontend Integration

- [ ] Add API method in `frontend/src/api/{domain}.ts`:
  ```typescript
  export async function createResource(data: CreateRequest): Promise<Response> {
    return api.post('/api/your-resource', data, {
      headers: { Authorization: `Bearer ${getToken()}` }
    });
  }
  ```
- [ ] **CRITICAL**: Always include `Authorization` header for auth endpoints
- [ ] Match TypeScript types exactly with Rust DTOs

---

## Phase 5: Verification

- [ ] Test with curl:
  ```bash
  curl -X POST http://localhost:3000/api/your-resource \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{"field": "value"}'
  ```
- [ ] Verify OpenAPI spec updated: `GET /api-docs/openapi.json`
- [ ] `cargo build` and `npm run build` pass
- [ ] Update ERROR_LOG if this fixes an existing issue

---

## Anti-Patterns to Avoid

| ❌ Don't                            | ✅ Do                                     |
| :--------------------------------- | :--------------------------------------- |
| Use `unwrap()` in handlers         | Use `?` with `AppError`                  |
| Hardcode `/api/users/search` twice | Search for duplicates first              |
| Return generic 500 for all errors  | Return specific error codes              |
| Skip auth header in frontend       | Always include if endpoint requires auth |
