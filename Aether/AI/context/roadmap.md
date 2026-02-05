# Project Roadmap

**Current Focus**: Phase 4 - Standardization & Unification
**Date**: 2026-02-05

## 🏆 Active Objectives

1.  **Architecture Unification**: Move to "Single-Window OS" model (V3 Spec).
2.  **User Experience**: Refactor Navigation from Website-style to OS-style (Launchpad/Dock).
3.  **Resilience**: (Completed) Standard Backup/Restoration Protocol.

---

## 📅 Roadmap Steps

### Phase 4.1: The Asset & Backup Foundation (COMPLETED)
> **Goal**: Secure data ownership and portability.

-   [x] **Spec Ratification**:
    -   [x] Backup Protocol V1 (`AI/context/specs/kb_backup_protocol_v1.md`)
    -   [x] Asset KB Spec V1 (`AI/context/specs/asset_kb_spec_v1.md`)
-   [x] **Asset KB Implementation**:
    -   [x] **Backend**: Created `MyAssets` KB Type & Schemas (`image`, `ip`, `domain`).
    -   [x] **Backend**: Implemented `AssetManager` with CAS storage and Contextual Access.
    -   [x] **Frontend**: Built "My Assets" App (Grid View, Drag & Drop Upload).
-   [x] **Backup System Implementation**:
    -   [x] **Backend**: `BackupService` (Zip generation, `meta.json` serialization, Asset inclusion).
    -   [x] **Backend**: `RestoreService` (ID Remapping, Smart Import, Asset Recovery).
    -   [x] **UI**: Backup Manager in Settings (Export/Import/Download).

### Phase 4.2: The OS Shell Refactor (Next Up)
> **Goal**: Implement Aether V3 Navigation Topology.

-   [ ] **Shell**: Remove `KnowledgeBaseDetail` route (Legacy Detail Page).
-   [ ] **Shell**: Implement "Launchpad" (Library) View - The new Home.
-   [ ] **Shell**: Implement "Dock" (Taskbar) with Pinning/Running states.
-   [ ] **Window Manager**: Implement "System Header" (Home/Close/Title) replacing browser-like navigation.

### Phase 5: Domain Knowledge Bases

#### 5.1 Math KB (Refinement)
-   [ ] Adopt V3 Shell (Run as an App).
-   [ ] Integrate "My Assets" for diagrams/images.

#### 5.2 Vulnerability KB (New)
-   [ ] **Schema Design**: `vulnerability_report`, `exploit_script`.
-   [ ] **Asset Integration**: Strong dependency on `My Assets` (IPs, Domains).
-   [ ] **Scanner Integration**: (Future) Auto-import Nmap/Burp results.

---

## 📜 Complete Feature Log

### [2026-02-05] Data Resilience & Assets
- **Asset KB**: Implemented "My Assets" system singleton for centralized file/entity management.
- **Backup System**: Implemented Server-Side Zip backups (`.akb`) with "Always New" restoration policy.
- **Contextual Security**: Implemented B2 authorization (Access Asset via Article Reference).

### [2026-01-28] Architecture V3
- Pivoted to "OS Model".
- Eliminated "Detail Pages".
- Introduced "Dock" and "Launchpad".
