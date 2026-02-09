---
description: Standard ReBAC (Relationship-Based Access Control) Model and Architecture for Aether
---

# ReBAC Permission Standard (The "Spec")

> **CRITICAL**: This is the **Single Source of Truth** for Aether's Permission System.
> All AI Agents must adhere to these 10 Design Decisions.

## 1. Core Architecture: Google Zanzibar-style ReBAC
We do **not** use a Roles table. We use a **Relationships** table (Tuples).

-   **Model**: `(Entity:UUID, Relation:String, Subject:UUID)`
-   **Subject**: Can be a `User` OR a `Group`.
-   **Logic**: **Additive Only** (Union of permissions). There is **NO DENY** logic.

## 2. Granularity & Scope (Instance Level)
Permissions can be applied to **ANY** entity instance, not just modules.
-   **Supported**: "User A is viewer of KB #123".
-   **Supported**: "User B is editor of Article #456".
-   **Supported**: "User A is admin of System".

## 3. UI/UX Standard: "Hierarchy + Explicit List"
The `UserManagement` UI (and any permission modal) MUST follow this pattern:
1.  **Top Section (Roles/Groups)**: Checkboxes for System Groups (e.g., "Admin", "Editor").
    -   *Mapping*: Turning "On" adds user to the `Group`.
2.  **Bottom Section (Explicit Grants)**: A table listing **Instance-Level** grants.
    -   *Columns*: Resource Name | Type | Permission | Action (Revoke).
    -   *Add Action*: "Add Permission" button -> Search Modal -> Select Resource.
3.  **Audit Tab**: A dedicated "History" tab inside the modal showing *who changed what and when*.

## 4. Default Policies
### 4.1 Visibility
-   **Private by Default**: New KBs/Folders are visible **ONLY** to the creator.
-   **Inheritance**: Child nodes inherit parent permissions (via Graph Walk).

### 4.2 Super Admin (Break-Glass)
-   **Default**: Super Admins **CANNOT** read user's private content.
-   **Mechanism**: Admin must click "**Take Ownership**" or "**Force Access**".
-   **Audit**: This action triggers a **CRITICAL** audit log entry.

### 4.3 Sharing
-   **Internal**: Other users see content only if authorized (or via "Internal Public" setting).
-   **External**: Supported via **Secret Links** (Token-based) only. No "Public Web" listing.

## 5. Group Management
-   **User-Managed**: Any user can create a **Team** (Group).
-   **Delegation**: Users can invite others to their Teams and share resources with the Team.

## 6. Technical Implementation
### 6.1 The Look-Up Path
To check if `User:U` can `read` `Article:A`:
1.  **Direct**: Is there a tuple `(A, viewer/editor/owner, U)`?
2.  **Group**: Is `U` in `Group:G` AND tuple `(A, viewer..., G)` exists?
3.  **Inheritance**: Recurse check on `A.parent_id`.
4.  **Break-Glass**: Reserved for future implementation.
    -   **Current Code**: Uses `u64::MAX` permission bypass (Super Admin).
    -   **TODO**: Implement proper "Force Access" button that adds an auditable Tuple and logs to `CRITICAL` level, per Spec 4.2.

### 6.2 Frontend Usage
-   **Fetch**: `GET /api/users/:id/permissions` (Returns Groups + Direct Tuples).
-   **Modify**: `POST /api/permissions/tuple` (Add/Remove tuples).
