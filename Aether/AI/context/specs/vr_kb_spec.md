
# Architecture Specification: Vulnerability Research Knowledge Base (VRKB)

**Version**: 1.0 (Draft)
**Date**: 2026-01-21
**Status**: APPROVED

## 1. Core Philosophy
The VRKB is not just a bug tracker; it is a **"Knowledge & Audit Meta-Layer"** designed for security practitioners.
*   **Practitioner-First**: UI/UX focuses on efficiency for auditors, not high-level stats for executives.
*   **Knowledge-Centric**: Every finding is an opportunity to link to and enrich the core Knowledge Base.
*   **Git-like Audit Trail**: Every action is versioned, attributable, and traceable.

## 2. Data Model

### 2.1 Hierarchy
The system uses a strictly hierarchical yet flexible model:
1.  **Project (Container)**: The root unit (e.g., "Linux Kernel Audit v6.1").
    *   *Properties*: Members, Asset Quota, Repository Link.
2.  **Section (Engagement)**: Logical division of work (e.g., "Network Stack", "Scheduler", "Phase 1: Fuzzing").
3.  **Finding (Atomic Unit)**: The vulnerability artifact.
    *   *Properties*: Severity, Status, Tags, Author, Linked Assets, Associated Code Commit.

### 2.2 Global Independence
*   **Findings** are globally indexable. A user can search "Heap Overflow" and find instances across all Projects.
*   **Assets** are stored in a global pool and referenced by Projects.

## 3. Workflow & Lifecycle

### 3.1 Status Machine
*   **Default Flow**: *Triage -> Confirming -> In Remediation -> Verified -> Closed*.
*   **Flexibility**: Users can define custom states or use Tags for ad-hoc status (e.g., `#needs-poc`).

### 3.2 Triage Queue (The Gatekeeper)
*   **Hybrid Entry**:
    *   **Manual**: Direct creation by authorized auditors.
    *   **Automated**: Imports from tools (Nmap, Burp) land in a **Triage Queue**.
*   **Review Process**: An Admin/Reviewer must "Approve" an imported item for it to become a Finding.

### 3.3 Checklists
*   **Auditing Blueprints**: Checklists are not simple booleans on a bug. They are "Methodology Guides" attached to Sections.
*   **Evidence-Based**: Marking a checklist item as "Done" requires creating a "Trace" (Comment/Log) explaining *how* it was verified.

## 4. Asset Management

### 4.1 Hybrid Storage Architecture
*   **Backend (Object Storage)**: S3-compatible storage for binaries, firmware, and videos. Supports deduplication (SHA256) and streaming.
*   **Frontend (Virtual Finder)**: A familiar file-system UI presented to the user. Folders are virtual tags/prefixes.
*   **Quotas**: Implementation of per-user and per-project storage limits.

## 5. Integrations

### 5.1 Code Repositories (GitHub/GitLab)
*   **Meta-Layer**: The VRKB floats "above" the code.
*   **Sync**:
    *   Display latest Commits/Branches in the Project Dashboard.
    *   Link Findings to specific Commit Diffs.

### 5.2 Notifications
*   **Webhooks**: API interface to trigger external bots (e.g., **QQBot**) for Critical findings or TODO deadlines.

## 6. Visualization (Self Space)

### 6.1 Vulnerability Kanban
*   **Columns**: *Pending Review | In Progress | Verified*.
*   **Cards**: Minimalist, showing Severity color, Title, Assignee.
*   **Style**: "London Academic" aesthetic (clean typography, subtle borders, no clutter).

### 6.2 Global TODOs
*   **Scope**: Unified list of "My Tasks" + "Project Tasks".
*   **Features**:
    *   Deadlines with Countdown.
    *   Comments/Discussion threads per task.
    *   Toggle to view "Team Tasks".

## 7. Knowledge Reuse (Smart Linking)
*   **Contextual Suggestion**: When creating a Finding, the system analyzes text (e.g., "Buffer Overflow") and suggests linking to existing "Theory Articles".
*   **Bidirectional**: The Theory Article will eventually show a list of "Real-world instances" from the VRKB.
