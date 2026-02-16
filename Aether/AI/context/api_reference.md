# Aether API Reference (v1.0)

> **Status**: Manual Snapshot (Pre-Utoipa Migration)
> **Last Updated**: 2026-02-07
> **Governance**: See [API Standards](#api-governance-standards) section below.

## API Governance Standards

Based on the architectural decision (2026-02-07), the following standards apply to all future API development:

1.  **Format**: **OpenAPI (Swagger)**.
2.  **Strategy**: **Code First + Auto Generation + Hybrid AI**.
    -   We will transition to generating the spec from code annotations.
    -   AI Agents are responsible for maintaining these annotations.
3.  **Implementation**: **Rust `utoipa` Crate**.
    -   Controllers must be annotated with `#[utoipa::path(...)]`.
    -   Schemas must be derived via `#[derive(ToSchema)]`.
4.  **Error Handling**: **Detailed Business Errors**.
    -   The spec must enumerate specific error codes (e.g., `TitleExists`, `QuotaExceeded`) not just generic HTTP 400/500.
5.  **Frontend Integration**: **Generate Types Only**.
    -   We will generate TypeScript interfaces (`.d.ts`) from the OpenAPI spec.
    -   We will *not* generate a full client SDK; `axios` calls remain manual but type-safe.

---

## Migration Plan (To Utoipa)

- [x] **Phase 1**: Add `utoipa` and `utoipa-swagger-ui` dependencies to `Cargo.toml`.
- [x] **Phase 2**: Create `src/interface/api/openapi.rs` to aggregate `OpenApi` struct.
- [x] **Phase 3**: Refactor `Vocabulary` module (Pilot) to add macros.
- [ ] **Phase 4**: Refactor remaining modules.
- [ ] **Phase 5**: Setup `scripts/gen_openapi.sh` and `scripts/gen_ts_types.sh`.

---

## Current API Index (Manual)

### Authentication (`/api/auth`)
| Method | Path | Description |
| :--- | :--- | :--- |
| `POST` | `/api/auth/login` | User login |
| `POST` | `/api/auth/register` | User registration |
| `GET` | `/api/users/:id` | Get user profile |
| `PUT` | `/api/users/:id` | Update user profile |

### Vocabulary & Dictionary (`/api/vocabulary`, `/api/dictionary`)
| Method | Path | Description |
| :--- | :--- | :--- |
| `POST` | `/api/vocabulary` | Save vocabulary word |
| `GET` | `/api/vocabulary` | List vocabulary |
| `POST` | `/api/vocabulary/batch-delete` | Batch delete words |
| `DELETE` | `/api/vocabulary/:id` | Delete single word |
| `POST` | `/api/vocabulary/:id/examples` | Add example to word |
| `POST` | `/api/vocabulary/:id/increment_query` | Increment query count |
| `POST` | `/api/vocabulary/:id/toggle_importance` | Toggle star status |
| `POST` | `/api/vocabulary/sentences/search` | **(New)** Search global shared sentences |
| `GET` | `/api/dictionary/lookup` | Look up word definition |
| `GET` | `/api/dictionary/fuzzy` | Fuzzy search words |

### Content & Knowledge Base (`/api/content`, `/api/knowledge-bases`)
| Method | Path | Description |
| :--- | :--- | :--- |
| `POST` | `/api/content` | Create content |
| `GET` | `/api/content` | List content |
| `GET` | `/api/content/:id` | Get content detail |
| `PUT` | `/api/content/:id` | Update content |
| `DELETE` | `/api/content/:id` | Delete content |
| `GET` | `/api/content/:id/history` | Get version history |
| `GET` | `/api/content/:id/history/:version` | Get specific version |
| `GET` | `/api/content/:id/diff/:v1/:v2` | Get diff between versions |
| `GET` | `/api/search` | Search content |
| `POST` | `/api/drafts/:id` | Save draft |
| `POST` | `/api/drafts/:id/publish` | Publish draft |
| `POST` | `/api/knowledge-bases` | Create KB |
| `GET` | `/api/knowledge-bases` | List KBs |
| `GET` | `/api/knowledge-bases/:id` | Get KB detail |
| `PUT` | `/api/knowledge-bases/:id` | Update KB |
| `DELETE` | `/api/knowledge-bases/:id` | Delete KB |

### VRKB (Virtual Research KB) (`/api/vrkb`)
| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/vrkb/projects` | List projects |
| `POST` | `/api/vrkb/projects` | Create project |
| `GET` | `/api/vrkb/projects/:id` | Get project detail |
| `GET` | `/api/vrkb/projects/:id/sections` | List sections |
| `POST` | `/api/vrkb/projects/:id/sections` | Create section |
| `GET` | `/api/vrkb/projects/:id/stats` | Get project stats |
| `GET` | `/api/vrkb/projects/:id/specs` | Get project specs |
| `PUT` | `/api/vrkb/projects/:id/specs` | Update project specs |
| `GET` | `/api/vrkb/projects/:id/members` | List members |
| `POST` | `/api/vrkb/projects/:id/members` | Add member |
| `DELETE` | `/api/vrkb/projects/:id/members/:uid` | Remove member |
| `PUT` | `/api/vrkb/projects/:id/members/:uid` | Update member role |
| `POST` | `/api/vrkb/assets` | Upload asset |
| `DELETE` | `/api/vrkb/assets/:id` | Delete asset |
| `GET` | `/api/vrkb/projects/:id/assets` | List project assets |
| `POST` | `/api/vrkb/sections/:id/findings` | Create finding |
| `GET` | `/api/vrkb/findings` | List findings |
| `GET` | `/api/vrkb/findings/:id` | Get finding |
| `PATCH` | `/api/vrkb/findings/:id/status` | Update finding status |
| `GET` | `/api/vrkb/projects/:id/docs` | List docs |
| `POST` | `/api/vrkb/projects/:id/docs` | Create doc |
| `GET` | `/api/vrkb/projects/:id/trash` | List trash |
| `GET` | `/api/vrkb/docs/:id` | Get doc |
| `PUT` | `/api/vrkb/docs/:id` | Update doc |
| `DELETE` | `/api/vrkb/docs/:id` | Move doc to trash |
| `POST` | `/api/vrkb/docs/:id/restore` | Restore doc |
| `DELETE` | `/api/vrkb/docs/:id/permanent` | Permanently delete doc |

### PRKB (Personal Research KB) (`/api/prkb`)
| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/prkb/feeds` | List feeds |
| `POST` | `/api/prkb/feeds` | Create feed |
| `DELETE` | `/api/prkb/feeds/:id` | Delete feed |
| `GET` | `/api/prkb/inbox` | Get inbox |
| `PATCH` | `/api/prkb/inbox/:id` | Update inbox item |
| `GET` | `/api/prkb/publications` | Get publications |
| `GET` | `/api/prkb/venues` | List venues |
| `POST` | `/api/prkb/fetch` | Trigger feed fetch |
| `GET` | `/api/prkb/papers` | List papers |
| `POST` | `/api/prkb/papers` | Save paper |
| `PATCH` | `/api/prkb/papers/:id` | Update paper |

### Memos (`/api/memos`)
| Method | Path | Description |
| :--- | :--- | :--- |
| `POST` | `/api/memos` | Create memo |
| `GET` | `/api/memos` | List memos |
| `GET` | `/api/memos/:id` | Get memo |
| `PUT` | `/api/memos/:id` | Update memo |
| `DELETE` | `/api/memos/:id` | Delete memo |
| `GET` | `/api/memos/workflow` | Get workflow |
| `PUT` | `/api/memos/workflow` | Update workflow |

### System & Admin
| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/system/git-log` | Get git log |
| `GET` | `/api/system/settings` | Get settings |
| `PUT` | `/api/system/settings` | Update settings |
| `GET` | `/api/backups` | List backups |
| `POST` | `/api/backups` | Create backup |
| `GET` | `/api/backups/download/:filename` | Download backup |
| `POST` | `/api/backups/restore` | Restore backup |
| `GET` | `/api/permissions/check` | Check permission |
| `POST` | `/api/permissions/grant` | Grant permission |
| `POST` | `/api/permissions/revoke` | Revoke permission |
| `POST` | `/api/permissions/break-glass` | Emergency access |
| `POST` | `/api/upload` | Generic file upload |
| `GET` | `/api/tags` | List tags |
| `POST` | `/api/comments/:type/:id` | Create comment |
| `GET` | `/api/comments/:type/:id` | Get comments |
