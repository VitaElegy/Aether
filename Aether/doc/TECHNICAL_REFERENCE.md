# Aether Technical Reference & Documentation

**Version:** 2.1.0
**Last Updated:** Dec 2025

## 1. Project Overview

Aether is a modern, block-based publishing platform featuring a **modular monolith backend** in Rust and a **reactive frontend** in Vue 3. Its distinguishing feature is a **Git-like Version Control System** for content, enabling full audit trails, semantic versioning, and difference comparison.

## 2. Technical Stack

### Backend (Rust)

- **Framework**: Axum (Web Server), Tower (Middleware)
- **Database**: PostgreSQL (Production) / SQLite (Dev) via **SeaORM** (Async ORM)
- **Serialization**: Serde, Serde JSON
- **Algorithms**:
  - `ring`: SHA-256 for Semantic Hashing
  - `similar`: Myers' Diff Algorithm for text comparison
  - `jsonwebtoken`: Stateless authentication
- **File Upload**: `axum-extra` (Multipart), `tower-http` (Static FS)

### Frontend (Vue 3 + TypeScript)

- **Build Tool**: Vite (with Proxy configuration)
- **State Management**: Pinia
- **Editor**: Tiptap (Headless wrapper around ProseMirror)
- **HTTP Client**: Axios (with Interceptors/Explicit Auth)
- **Styling**: Tailwind CSS
- **Markdown**: `marked` (with custom Renderer for TOC)

## 3. Core Features & Implementation Details

### 3.1 Git-like Version Control (CMS)

The core innovation of Aether is how it handles content updates. It does not simply overwrite data; it snapshots it based on semantic changes.

**The "Smart Snapshot" Logic (`PostgresRepository`):**

1.  **Input**: New content (Title, Body, Tags, Status).
2.  **Semantic Hash**: The system calculates a `SHA-256` hash of the _combined_ semantic fields.
    ```rust
    hash = SHA256(title + body + tags + status + visibility + category)
    ```
3.  **Comparison**: It fetches the _latest_ version from `content_versions`.
4.  **Decision**: A new version is created IF AND ONLY IF:
    - The Hash has changed.
    - OR the user provided an explicit `change_reason` (Manual Commit).
    - OR it is the first save.
5.  **ACID Transaction**: The update to the `contents` table (Current State) and the insertion into `content_versions` (History) happen in a **single atomic database transaction**.

### 3.2 Diffing Engine

Aether includes a built-in diffing service to visualize changes between versions.

- **Algorithm**: Myers' Diff Algorithm (via `domain/diff_service.rs`).
- **Granularity**: Line-based and Character-based diffing.
- **API**: `GET /api/content/:id/diff/:v1/:v2` returns a structured JSON of `Insert`, `Delete`, and `Equal` operations, ready for UI rendering (Red/Green highlighting).

### 3.3 Strict Type Safety & Audit

- **UUID Enforcement**: All IDs (User, Content, Editor) are strictly typed as UUIDs in the database (`uuid` type in Postgres) to prevent collision and ID enumeration attacks.
- **Attribution**: Every version records an `editor_id`. This is enforced via `NOT NULL` constraints in the database, ensuring no anonymous edits exist in the history.

### 3.4 Authentication Flow & Profile System

- **Stateless JWT**: Uses Bearer tokens.
- **Profile Management**:
  - `UserProfileView`: Displays user stats, bio, and avatar.
  - `SettingsView`: Allows updating profile fields and uploading avatars.
  - **Avatar Upload**: Handled by `upload_handler` (backend) which validates `image/*` types and saves to `uploads/avatars`. Served statically via `tower-http`.
- **Frontend Logic**:
  - `LoginView` authenticates and stores the token in `localStorage`.
  - `EditorView` explicitly attaches `Authorization: Bearer <token>` to sensitive write operations.
  - `UserProfileView` provides a logout mechanism that clears local state.

### 3.5 Reading Experience

- **ReadView**: Dedicated route `/article/:id` for consumption.
  - **Dynamic TOC**: Generated client-side using `marked.lexer`, displayed in a sticky left sidebar.
  - **Author Info**: Header displays avatar (with fallback), name, and timestamp.
  - **Edit Link**: Context-aware "Edit Entry" button appears only for the content author.

## 4. Database Schema

### `users`

| Column          | Type   | Description        |
| :-------------- | :----- | :----------------- |
| `id`            | UUID   | Primary Key        |
| `username`      | TEXT   | Unique login name  |
| `email`         | TEXT   |                    |
| `display_name`  | TEXT   | **New**: Full Name |
| `bio`           | TEXT   | **New**: Biography |
| `avatar_url`    | TEXT   | **New**: Image URL |
| `password_hash` | TEXT   | Argon2/Bcrypt hash |
| `permissions`   | BIGINT | Bitmask for RBAC   |

### `contents` (Head/Current State)

| Column       | Type   | Description                          |
| :----------- | :----- | :----------------------------------- |
| `id`         | UUID   | Primary Key                          |
| `body`       | JSONB  | Structured content (Markdown/Blocks) |
| `status`     | TEXT   | Draft / Published / Archived         |
| `slug`       | TEXT   | URL-friendly identifier              |
| `visibility` | TEXT   | Public / Private / Internal          |
| `category`   | TEXT   |                                      |
| `tags`       | TEXT[] | Array of tags                        |

### `content_versions` (History/Immutable)

| Column          | Type  | Description                     |
| :-------------- | :---- | :------------------------------ |
| `id`            | UUID  | PK for this snapshot            |
| `content_id`    | UUID  | FK to `contents`                |
| `version`       | INT   | Incremental version number      |
| `content_hash`  | TEXT  | SHA-256 integrity check         |
| `editor_id`     | UUID  | **Who** made the change (Audit) |
| `change_reason` | TEXT  | Optional commit message         |
| `body`          | JSONB | Full snapshot of data           |

## 5. Directory Structure

```text
Aether/
├── backend/                  # Rust Backend
│   ├── src/
│   │   ├── domain/           # Business Logic & Ports (Hexagonal)
│   │   │   ├── diff_service.rs # Diff Engine
│   │   │   ├── models.rs     # Core Structs
│   │   │   └── ports.rs      # Traits (Interfaces)
│   │   ├── infrastructure/   # Implementation Details
│   │   │   ├── auth/         # JWT & Hashing
│   │   │   └── persistence/  # Database (SeaORM)
│   │   │       ├── entities/ # ORM Models
│   │   │       └── postgres.rs # Repository Impl
│   │   ├── interface/        # API Layer (Axum)
│   │   │   ├── api/          # Route Handlers (auth, content, upload)
│   │   │   └── mod.rs
│   │   └── main.rs           # Entry & Migrations
│   ├── Cargo.toml
│   └── uploads/              # User Uploads (Avatars)
├── frontend/                 # Vue 3 Frontend
│   ├── src/
│   │   ├── components/       # UI Components
│   │   ├── router/           # Navigation Logic
│   │   ├── stores/           # Pinia State (Auth)
│   │   └── views/            # Pages (Editor, Home, Login, Read, Settings, Profile)
│   └── package.json
└── doc/                      # Documentation
    ├── ARCHITECTURE.md
    └── TECHNICAL_REFERENCE.md # This file
```
