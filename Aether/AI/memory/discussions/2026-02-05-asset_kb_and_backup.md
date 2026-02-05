# Discussion: Asset KB & Backup Protocol

**Date**: 2026-02-05
**Participants**: User, Senior Systems Architect

## 1. Summary
We established the protocols for two critical infrastructure components:
1.  **Backup System**: A robust, server-side archive system for disaster recovery and portability.
2.  **Asset Knowledge Base**: A centralized, user-owned repository for all referenced entities (images, IPs, etc.).

## 2. Key Decisions

### 2.1 Backup Protocol
-   **Format**: **Hybrid Zip**. Contains both `meta.json` (for system restore) and `content/` (Markdown for humans).
-   **Restoration Strategy**: **Always New**. Never overwrites; always creates a new KB instance with a timestamped name.
-   **Scope**: **Latest Only**. Does not include undo/redo history to save space.
-   **Storage**: **Server Archive**. Stored on server, managed via UI.

### 2.2 Asset Knowledge Base ("My Assets")
-   **Nature**: **System Singleton**. Every user has exactly one "My Assets" app.
-   **Content**: Not just files. Includes structured data like **IPs, Domains, Credentials** (useful for Vulnerability KB).
-   **Access Control**: **Contextual Access (B2)**.
    -   *Logic*: "If you can see the Article, you can see the Asset it references."
    -   *Impl*: Backend validates `(user -> article)` AND `(article -> asset)` linkage to grant access.
-   **Storage**: **Split**. Metadata in DB, binaries in File System/S3.
-   **UX**: **Full App**. Users can open "My Assets" as a standalone app to manage/clean/organize entities.

## 3. Action Items
-   [x] Create `AI/context/specs/kb_backup_protocol_v1.md`.
-   [x] Create `AI/context/specs/asset_kb_spec_v1.md`.
-   [x] Update `AI/context/roadmap.md`.
-   [ ] Start Implementation (Phase 4.1).
