# Specification: Asset Knowledge Base ("My Assets")

**Status**: DRAFT (Created 2026-02-05)
**Type**: Feature Specification

> **Purpose**: Define the architecture for a User-Singleton Knowledge Base dedicated to managing reusable entities (files, IPs, domains) with contextual permission inheritance.

## 1. Core Concept

The **Asset KB** is a specialized application that every user possesses automatically. It acts as the central repository for "Referenced Entities".

-   **Singleton**: One per user.
-   **Structured**: Not just files; supports logical entities.
-   **Context-Aware**: Permissions flow from usage.

## 2. Schema Definition

The Asset KB supports multiple **Block Types** (Schemas).

### 2.1 `image_asset`
Standard uploaded image.
```json
{
  "file_path": "uploads/2026/02/uuid.png",
  "mime_type": "image/png",
  "width": 1920,
  "height": 1080,
  "alt_text": "Server Architecture Diagram",
  "hash": "sha256:..."
}
```

### 2.2 `ip_asset` (Structured Data Example)
Represents a network endpoint (for Vulnerability KB).
```json
{
  "address": "192.168.1.10",
  "type": "ipv4",
  "tags": ["production", "internal"],
  "owner_contact": "admin@example.com"
}
```

### 2.3 `credential_stub`
A reference to a credential (NOTE: Actual secrets should NOT be stored in plaintext here, but this entity represents the *existence* of a credential).
```json
{
  "service": "AWS",
  "key_id": "AKIA...",
  "vault_path": "secret/prod/aws" // Pointer to external Vault
}
```

## 3. Architecture

### 3.1 Storage Strategy: **Split Model**
-   **Metadata**: Stored in PostgreSQL `nodes` table (jsonb body).
-   **Binary Data**: Stored in File System (`/storage/uploads/`).
-   **Linkage**: The `node.body.file_path` points to the physical file.

### 3.2 Permission Model: **Contextual Access (The "B2" Pattern)**

Traditional ACLs are too rigid for assets used across many docs. We use **Reference-Based Authorization**.

**The Rule**:
> "You can view Asset X if you can view (Article A OR Article B OR ...) that references Asset X."

**Implementation**:
1.  **Direct Access**: The Owner (User) always has full access to `My Assets`.
2.  **Embedded Access (The Proxy)**:
    -   Frontend requests asset via a Contextual URL:
        `GET /api/assets/:asset_id?context_id=:article_id`
    -   **Backend Validation**:
        1.  Check: Does User have `read` permission on `article_id`? (Standard ReBAC)
        2.  Check: Does `article_id` actually *contain* a reference to `asset_id`? (Parse/Graph Check)
        3.  If BOTH True -> **Grant Access**.
    -   *Cache Strategy*: Generate a signed, short-lived JWT (AssetToken) for the user session to avoid re-checking strictly on every image load.

## 4. User Interface (The "Full App")

The "My Assets" app runs in the standard OS Window.

### 4.1 Views
-   **Grid View**: Thumbnail grid for images.
-   **Table View**: Detailed columns for structured assets (IPs, Domains).
-   **Usage Tab**: Selecting an asset shows a list of "Used In: [Vulnerability Report #1], [Math Note #2]".

### 4.2 Interaction
-   **Drag & Drop**: Drag file from desktop to upload.
-   **Picker Mode**: When another app (e.g., Markdown Editor) requests an asset, "My Assets" opens in a **Modal / Split View** to allow selection.

## 5. Integration

-   **Vulnerability KB Integration**:
    -   When creating a Vuln Report, user types `@192...`.
    -   System searches "My Assets" for `ip_asset`.
    -   If not found, prompts "Create new Asset: 192...?".
    -   On selection, inserts a **Reference Block** `[[asset:uuid]]`.
