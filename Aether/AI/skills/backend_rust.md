# Skill: Backend Development (Rust)

## 1. Critical Constraints
| Constraint               | Rule                                                                                    |
| ------------------------ | --------------------------------------------------------------------------------------- |
| **Zero Panic Policy**    | Backend code must **NEVER panic**. Use `Result` and `AppError` types.                   |
| **Version ID Format**    | Version IDs are **integers** (e.g., "1"). **NEVER** use SemVer (`0.0.1`).               |
| **Version Immutability** | `content_versions` table is **APPEND-ONLY**. Never update existing rows.                |
| **Diff Format**          | API MUST return **Structured JSON** (`Vec<DiffChange>`). NEVER return raw diff strings. |
| **SeaORM Query**         | Avoid `select_only()` when hydrating full Models (causes "no column found" errors).     |

## 2. Coding Standards
### 2.1 Error Handling
- **Library**: Use `thiserror` for error definitions.
- **Mapping**: All errors map to `AppError` enum.
- **HTTP Status**: Return `500` only for truly unrecoverable faults.

### 2.2 Versioning Logic
- **Auto-Increment**: v1 -> v2 automatic on save.
- **Reasoning**: `change_reason` is mandatory.

## 3. Tech Stack
- **Framework**: Axum
- **ORM**: SeaORM
- **Search**: Meilisearch
