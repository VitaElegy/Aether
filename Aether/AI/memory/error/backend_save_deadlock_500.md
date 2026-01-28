# Error: Backend Save Deadlock (500 Internal Server Error)

## Symptoms
- **User Impact**: Creating or Publishing an article hangs for 30 seconds and then fails with a 500 error.
- **Logs**: `tower_http` logs show a 30000ms latency (timeout). No specific panic stack trace.
- **Frequency**: 100% reproduceable on every write operation to `ArticleRepository::save`.

## Root Cause
- **Deadlock Condition**:
  1. `ArticleRepository::save` begins a database transaction (`txn`).
  2. It performs `node::Entity::insert` and `article_detail::Entity::insert` using `&txn`.
  3. *Inside the same transaction scope*, it calls `self.add_relation(...)`.
  4. `add_relation` (in `PostgresRepository`) acquires a **new, separate database connection** from the pool (it doesn't accept the `txn`).
  5. `add_relation` attempts to ensure the entity exists or validate foreign keys.
  6. **BLOCKER**: The new connection cannot see the uncommitted changes from `txn` (Isolation level). Worse, if there's a row lock or table lock involved, the new connection waits for `txn` to release it.
  7. `txn` is waiting for `add_relation` to finish before it can commit.
  8. **Result**: Classic deadlock.

## Resolution
- **Fix**: Moved all `add_relation` calls to **after** `txn.commit()`.
- **Logic**: ReBAC permissions effectively "decorate" the entity. It is acceptable for the entity to exist for a split second without permissions if it prevents a deadlock.

## Prevention
- **Action Item**: Ensure repository methods that mix transactional writes and external calls (like ReBAC or other services) strictly separate the transaction scope from the external call, or pass the transaction handle down if possible (though SeaORM/ReBAC separation makes passing `txn` hard).
