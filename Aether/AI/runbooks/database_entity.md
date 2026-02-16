# Runbook: Adding a Database Entity

> **Prevents**: Missing tables, FK errors, SeaORM hydration issues
> **Error Examples**: `Missing Table (vocab_roots)`, `Failed to Create Knowledge Base`

---

## Phase 1: Schema Design

- [ ] Define table schema (columns, types, constraints)
- [ ] Identify foreign keys and verify target tables **exist**
- [ ] Choose primary key strategy (UUID vs Integer)

**Gate**: Do NOT proceed if FK target tables don't exist.

---

## Phase 2: Migration

- [ ] Create migration file:
  ```
  backend/migrations/YYYYMMDD_HHMMSS_create_{table}.sql
  ```
- [ ] Run migration:
  ```bash
  cd backend && sqlx migrate run
  ```
- [ ] Verify table exists:
  ```bash
  sqlite3 data.db ".schema {table}"
  ```

**Gate**: Confirm table schema matches your design before proceeding.

---

## Phase 3: SeaORM Entity

- [ ] Create entity file: `backend/src/entity/{table}.rs`
- [ ] Add to `backend/src/entity/mod.rs`:
  ```rust
  pub mod {table};
  pub use {table}::Entity as {Table};
  ```
- [ ] Define `Relation` enum (even if empty)
- [ ] Implement `ActiveModelBehavior` trait

**Common Pitfall**: If using `Related<T>`, ensure the target entity is also exported.

---

## Phase 4: Repository

- [ ] Create repository: `backend/src/repositories/{domain}_repository.rs`
- [ ] Implement CRUD using `Result<T, AppError>` — never panic
- [ ] **NEVER** use `select_only()` when hydrating full models
- [ ] Export in `mod.rs`

---

## Phase 5: Verification

- [ ] `cargo build` passes with no errors
- [ ] Write at least one `#[test]` for the repository
- [ ] If this fixes an ERROR_LOG issue, update that file

---

## Quick Reference

```rust
// Entity template
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "your_table")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```
