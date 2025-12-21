# Aether Architecture Documentation

## Core Principles

- **Modular Monolith**: The backend is structured as a monolith but with strict domain boundaries (Hexagonal Architecture).
- **Type Safety**: Leveraging Rust's type system to prevent illegal states.
- **Git-like Versioning**: Content is versioned with semantic hashing and full snapshots.

## Content Management System (CMS)

### Version Control Strategy

The system implements a robust, Git-like version control mechanism for content.

1.  **Semantic Hashing**:
    Every time content is saved, we calculate a SHA-256 hash of its semantic components (`Title` + `Body` + `Tags` + `Status`).
    A new version snapshot is ONLY created if:
    - The hash differs from the previous version's hash (Content changed).
    - Or, the user explicitly provided a `reason` (Manual commit).

2.  **ACID Guarantees**:
    Content updates and Version creation happen within a single Database Transaction. This ensures that the `current` state in the `contents` table always matches the `latest` entry in `content_versions` table (if a version was created).

3.  **Diff-Match-Patch (DMP)**:
    While we store full snapshots for reliability, we expose a **Diff API** (`/api/content/:id/diff/:v1/:v2`) that calculates the differences between two versions on the fly.

    **Algorithm**:
    We use the Myers' Diff Algorithm (via the `similar` crate) to generate a line-by-line or char-by-char difference set.

    **API Response Format**:
    ```json
    {
      "changes": [
        { "tag": "Equal", "value": "This text is unchanged. " },
        { "tag": "Delete", "value": "old word" },
        { "tag": "Insert", "value": "new word" }
      ]
    }
    ```
    Frontend clients can use this structured data to render "Red/Green" diff views similar to GitHub PRs.

### Database Schema

- `users`: Authentication and permissions.
- `contents`: The *current* state of all articles.
- `content_versions`: Immutable history of all changes.
    - `change_reason`: The "commit message".
    - `content_hash`: Integrity check.
    - `editor_id`: Audit trail.

## Tech Stack

- **Language**: Rust
- **Framework**: Axum
- **Database**: PostgreSQL (via SeaORM)
- **Diff Engine**: `similar` crate

