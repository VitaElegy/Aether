# Aether Architecture Manifesto

## 1. Design Philosophy (设计哲学)

Aether implies the medium that connects all things. This system is not a monolith; it is a kernel that orchestrates plugins.

### 1.1 The Core Law: Interface over Implementation
Code against `Traits`, never against `Structs`.

*   **Wrong:** `fn save_post(db: &PostgresConnection, post: Post)`
*   **Right:** `fn save_content<S: StoragePort>(storage: &S, content: impl Content)`

## 2. Backend Architecture (Rust)

We utilize a Hexagonal (Ports & Adapters) Architecture.

```
src/
├── domain/           # Pure Logic. NO EXTERNAL DEPENDENCIES (No sqlx, no axum).
│   ├── models/       # Structs representing the core data.
│   └── ports/        # Traits defining what the system NEEDS (Repository interfaces).
├── application/      # Orchestration.
│   ├── use_cases/    # Application logic (e.g., "PublishPost").
│   └── services/     # Domain services.
├── infrastructure/   # The dirty details.
│   ├── persistence/  # Database implementations (SeaORM, Redis).
│   └── adapters/     # External APIs, File Systems.
└── interface/        # Entry points.
    ├── api/          # REST/GraphQL endpoints (Axum handlers).
    └── cli/          # CLI commands.
```

### 2.1 Key Abstractions

#### `ContentProvider` Trait
The blog is agnostic to *what* it serves. It could be a text post, a video, a code snippet, or a 3D model.

```rust
#[async_trait]
pub trait ContentProvider: Send + Sync {
    type Metadata: Serialize + DeserializeOwned;
    fn render(&self) -> Result<String, RenderError>;
    fn validate(&self) -> ValidationResult;
}
```

#### `StoragePort` Trait
The blog is agnostic to *where* it lives.

```rust
#[async_trait]
pub trait StoragePort<T>: Send + Sync {
    async fn save(&self, item: T) -> Result<Id, StorageError>;
    async fn find_by_id(&self, id: Id) -> Result<Option<T>, StorageError>;
    async fn query(&self, query: Query) -> Result<Vec<T>, StorageError>;
}
```

## 3. Frontend Architecture (Vue 3 + TDesign)

The frontend mirrors the backend's elegance through **Composition API**.

### 3.1 The "Renderer" Pattern
Instead of a giant `if-else` block for different post types, we use a dynamic component loader strategy.

```typescript
// dynamic-loader.ts
const renderers = new Map<string, Component>();
export function registerRenderer(type: string, component: Component) { ... }
```

### 3.2 State Management
Pinia stores should be minimal. Complex logic belongs in pure TypeScript service classes or composables, decoupled from the UI framework.

## 4. Tech Stack Decision Matrix

| Component | Choice | Rationale |
| :--- | :--- | :--- |
| **Language** | Rust | Memory safety without GC. Enforces correctness at compile time. |
| **Web Fx** | Axum | Built on Tokio/Tower. Ergonomic yet extremely powerful. |
| **ORM** | SeaORM | Async, dynamic. Fits our generic/plugin architecture better than Diesel. |
| **Frontend** | Vue 3 | Composition API allows logic reuse better than React Hooks (imo). |
| **UI** | TDesign | Clean, enterprise-grade, fits the "Elegant" persona. |
| **Build** | Vite | Webpack is for dinosaurs. |

---
*Signed,*
*The Architect*

