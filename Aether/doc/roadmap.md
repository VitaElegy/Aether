# Aether Roadmap & Status

## 🟢 Status: Phase 1 (Foundation) - COMPLETED

### Core Architecture

- [x] **Project Structure**: Hexagonal Architecture (Domain/Infra/Interface) established.
- [x] **Database**: Postgres integration via SeaORM with auto-migration.
- [x] **Dependency Injection**: `AppState` utilizing `Arc<dyn Trait>` for loose coupling.

### Authentication & Security

- [x] **Secure Storage**: Argon2id password hashing implementation.
- [x] **Token System**: JWT generation and verification service.
- [x] **API**: `/api/auth/login` and `/api/auth/register` endpoints.
- [x] **Frontend Guard**: Vue Router navigation guards for protected routes.

### User Interface (The "Void" Design)

- [x] **Visual Identity**: "Future Minimalist" theme (Dark mode, Glassmorphism).
- [x] **Motion**: Fluid entrance animations using `v-motion`.
- [x] **Login Experience**: Split-screen design with generative abstract art.
- [x] **Dashboard**: Information-dense layout with sidebar navigation and stats.

### Observability

- [x] **Structured Logging**: JSON logging to file + Pretty printing to stdout (`tracing`).
- [x] **Trace Middleware**: Automatic HTTP request tracking.

### Extensibility

- [x] **WASM Host**: Basic `WasmPluginHost` structure integrated into infrastructure.
- [x] **Dynamic Rendering**: Frontend `DynamicRenderer` component for polymorphic content.

---

## 🟡 Phase 2 (Content & Interaction) - NEXT STEPS

### Content Management

- [ ] **CRUD API**: Implement `GET/POST/PUT/DELETE` for `/api/content`.
- [ ] **Rich Editor**: Integrate a Markdown/WYSIWYG editor in the frontend.
- [ ] **Tag System**: Implement the logic for tag aggregation and searching.

### Advanced Plugin System

- [ ] **WIT Interface**: Define a standard WIT (Wasm Interface Type) for plugins.
- [ ] **Hot Reloading**: Allow plugins to be uploaded/reloaded without restarting the backend.
- [ ] **Marketplace**: A simple UI to list and enable installed plugins.

### Social Features

- [ ] **Comments**: A threaded comment system (stored as JSONB or separate table).
- [ ] **Reactions**: Simple emoji reactions for posts.

---

## 🔴 Phase 3 (The Singularity) - LONG TERM

- [ ] **Edge Replication**: Support for SQLite replication to edge nodes (Turso/LiteFS).
- [ ] **AI Integration**: Local LLM inference via `rust-bert` for content summarization.
- [ ] **3D Space**: Replace the 2D dashboard with a WebGL-based navigation system.
