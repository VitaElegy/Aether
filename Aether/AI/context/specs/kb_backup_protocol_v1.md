# Protocol: Knowledge Base Backup & Restoration (V1)

**Status**: DRAFT (Created 2026-02-05)
**Type**: System Protocol

> **Purpose**: Define the standard for exporting knowledge bases into portable, recoverable archives, and the server-side management of these backups.

## 1. Core Philosophy

1.  **Hybrid Portability**: Backups must serve two masters:
    -   **The System**: Requires absolute fidelity (IDs, Relations, Permissions) for restoration.
    -   **The Human**: Requires readability (Markdown, Folder Structure) for manual inspection/migration.
2.  **Non-Destructive Restoration**: Importing a backup **NEVER** overwrites existing data. It always creates a new, timestamped fork.
3.  **Server-Side Sovereignty**: Backups are first-class server resources, managed via UI, not just ephemeral downloads.

## 2. The Artifact: `.akb` (Aether Knowledge Base)

The backup artifact is a standard ZIP file with the extension `.akb` (or `.zip`).

### 2.1 Internal Structure

```text
/ (Root)
├── meta.json                # System Source of Truth (The "Database Dump")
├── assets/                  # Physical binary files (Images, PDFs)
│   ├── 8f3a2...png
│   └── ...
└── content/                 # Human Readable Mirror
    ├── Folder A/
    │   ├── My Article.md
    │   └── ...
    └── Root Article.md
```

### 2.2 `meta.json` Schema

This file contains the raw database state required to reconstruct the graph.

```json
{
  "version": "1.0",
  "exported_at": "2026-02-05T14:30:00Z",
  "knowledge_base": {
    "id": "kb_uuid",
    "title": "Math KB",
    "type": "math_v1"
  },
  "nodes": [
    {
      "id": "node_uuid",
      "parent_id": "parent_uuid_or_null",
      "title": "Theorem 1.2",
      "content_hash": "...",
      "properties": { ... }, // KB-specific fields
      "path": "Folder A/My Article.md" // Mapping to human readable file
    }
  ],
  "edges": [
    { "source": "A", "target": "B", "type": "relates_to" }
  ],
  "assets_map": {
    "asset_uuid": "assets/8f3a2...png" // Mapping internal ID to zip path
  }
}
```

## 3. Workflow

### 3.1 Export (Backup)

1.  **Trigger**: User clicks "Backup" in KB Settings.
2.  **Backend Job**:
    -   Locks the KB (Optional, or snapshot read).
    -   Fetches all Nodes and Edges.
    -   Generates `meta.json`.
    -   Resolves all referenced Assets (from System Asset KB) and copies physical files to `assets/`.
    -   Generates `content/` tree by converting Nodes to Markdown.
    -   Zips everything to `storage/backups/{kb_id}_{timestamp}.akb`.
3.  **Result**: Returns a Job ID. UI polls for completion, then shows the backup in the "Backups" list.

### 3.2 Import (Restore)

1.  **Trigger**: User selects a backup from the list and clicks "Restore".
2.  **Strategy**: **"Always New"**
    -   New KB ID is generated: `uuid_v7()`.
    -   Title becomes: `${Original Title} (Restored ${Date})`.
3.  **Process**:
    -   Unzip to temp.
    -   Read `meta.json`.
    -   Create new KB entry.
    -   **ID Remapping**: All Node IDs in the backup are **Regenerated** to avoid collision with the original KB (if it still exists). All internal links (`[[node_id]]`) in content must be rewritten to match new IDs.
    -   **Asset Handling**:
        -   Check if Asset Hash exists in "My Assets".
        -   If yes: Link to existing.
        -   If no: Import as new Asset.
4.  **Completion**: Notification "Restoration Complete". New KB appears on Launchpad.

## 4. API Endpoints

-   `POST /api/backups/create`: `{ kb_id: string }`
-   `GET /api/backups/list`: Returns list of available backups (Server-side).
-   `POST /api/backups/restore`: `{ backup_filename: string }`
-   `GET /api/backups/download/:filename`: Stream the .zip to client.

## 5. Security Note

-   **Encryption**: V1 does NOT encrypt backups. If the server is compromised, backups are readable.
-   **Permission**: Only `Owner` of a KB can export/restore it.
