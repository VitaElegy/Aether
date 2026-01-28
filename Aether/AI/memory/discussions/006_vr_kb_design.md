# Discussion: Vulnerability Research Knowledge Base (VRKB)

**Date**: 2026-01-20
**Participants**: User, AI

## Context
The user wants to add a "Vulnerability Research Knowledge Base" to `self_space`.
**Core Goal**: Support multi-user synchronization for vulnerability mining progress, reports, TODOs, etc.
**Nature**: A specialized Knowledge Base within the existing system.

## Deep Questions for Definition

### 1. Data Structure & Granularity (Resolved)
**Decision**:
*   **Core Unit**: **Project** (e.g., "Linux Kernel Audit").
*   **Hierarchy**: Project -> Sections (e.g., Network, Scheduler) -> Findings/Bugs.
*   **Independence**: Bugs must be independently retrievable (global list) but belong to a project.
*   **Type**: Projects are "Special Articles" acting as containers.
*   **Future Proofing**: Design must support non-text nodes (Binary, Video) in the future.

**Refinement**: This aligns with the **DefectDojo** model: *Product (Project) -> Engagement (Section/Phase) -> Finding*.

**(Original Question preserved below for archive)**
*   ~~What constitutes the atomic unit of this KB?~~

### 2. Workflow & Lifecycle (Resolved)
**Decision**:
*   **Checklist Type**: **"Auditing Blueprint/Guide"** (Not key-value pairs).
    *   Items serve as prompts (e.g., "Check SQLi").
    *   **Evidence**: Users must fill in *how* they checked it (Methodology, Logs) to mark as done. "Elegy checked via sqlmap...".
*   **Status**: **Flexible & Customizable**. Defaults provided, but users can add/remove states and use **Tags**.
*   **History**: **Git-like Audit Trail**. Must record *who* did *what* and *when* (and potentially file diffs).
*   **Spirit**: Flexible, powerful, robustness over rigid constraints.

**(Original Question preserved below for archive)**
*   ~~Can you define the ideal "status lifecycle"...?~~

### 3. Collaboration & Audit Workflow (Resolved)
**Decision**:
*   **Checklist**: **"Auditing Blueprint"** (Methodology & Evidence recording).
*   **History**: **Git-like** traces (Who, What, When, Diff).
*   **Flexibility**: Flexible state transitions, customizable via Tags.

**(Original Question preserved below for archive)**
*   ~~multi-user synchronization... conflict resolution...?~~

### 4. Asset & Target Management (Resolved)
**Decision**:
*   **Architecture**: **Mixed Model**.
    *   **Backend**: **Object Storage (S3-like)** for flat, deduplicated, scalable storage (Binary, Firmware, Video).
    *   **Frontend**: **Virtual Finder** (Directory structure based on Metadata/Tags) for user habit.
*   **Quotas**: User/Project limits + Deduplication (SHA256) to save space.
*   **Philosophy**: Assets are an independent "Pool" referenced by Projects.

**(Original Question preserved below for archive)**
*   ~~Does the system need to track "Targets"... independently...?~~

### 5. Integration & Import (Resolved)
**Decision**:
*   **Hybrid Model**: Supports both Manual Entry and Tool Import (Nmap/Burp).
*   **Mandatory Review (Triage)**: Imported data enters a "Pending/Unverified" queue.
*   **Gatekeeper**: Must be audited/reviewed by an Admin/Senior before becoming an official "Finding".
*   **Analogy**: GitHub Pull Request workflow.

**(Original Question preserved below for archive)**
*   ~~Will this workflow involve automated tools...?~~

### 6. Reporting & Output (Resolved)
**Decision**:
*   **Engineering-First**: Priority on **GitLab/GitHub Integration**.
    *   Sync Commits, Issues, Branches.
    *   System acts as a "Meta-Layer" over the code repo.
*   **Discussion System**: Built-in commenting/threading for collaboration.

**(Original Question preserved below for archive)**
*   ~~What is the final output? PDF...?~~

### 7. Knowledge Reuse (Resolved)
**Decision**:
*   **Smart Suggestions**: System MUST proactively suggest standard KB articles based on finding context.
*   **Goal**: "Comfortable" workflow where knowledge is surfaced automatically.

**(Original Question preserved below for archive)**
*   ~~How should this link to your *general* Knowledge Base?~~

### 8. Security & Access Control (Resolved)
**Decision**:
*   **Strict RBAC**: Leverage existing Project permissions.
*   **Groups**: User Groups (e.g., Audit Team, Viewers, Admins).
*   **Granularity**: Reviewer vs Submitter privileges.

**(Original Question preserved below for archive)**
*   ~~Do we need Granular Permissions...?~~


6.  **Reporting & Output**:
    *   What is the final output? Is it a standard PDF report for a vendor, a Markdown advisory, or a JIRA ticket? Do we need a "Report Generator" that pulls selected vulnerabilities into a formatted document?

7.  **Knowledge Reuse**:
    *   How should this link to your *general* Knowledge Base? For instance, if I find a "Heap Overflow," should I be able to easily link to my general "Heap Overflow" study notes? Should this linkage be bidirectional?

8.  **Security & access Control**:
    *   Vulnerability data is sensitive. Do we need "Granular Permissions"? (e.g., User A can see the "Web" project but not the "Kernel" project, or even "User B can see the bug title but not the Exploit POC")?

### 9. Visualization & Dashboard (Resolved)
**Decision**:
*   **Style**: **Vulnerability-Centric Kanban Board** (Pending -> Triage -> Fixing -> Verified).
*   **Audience**: **For Practitioners**, not managers. Show "Elegant, Useful Information".
*   **Aesthetics**: Follow the "London Academic" design spec.

**(Original Question preserved below for archive)**
*   ~~For the "Self Space" view, what is the most critical... info?~~

### 10. The "TODO" System (Resolved)
**Decision**:
*   **Scope**: **Global + Personal**. "All" tasks appear in personal views by default (toggleable).
*   **Features**:
    *   **Deadlines**: Yes.
    *   **Alerts**: App-level + **External Webhooks** (QQBot API interface).
    *   **Collaboration**: Comments on TODOs.

**(Original Question preserved below for archive)**
*   ~~Are these simple checklists... or a global "Task Queue"?~~

## Conclusion
Key Design Pillars established:
1.  **Project-Centric**: *Project -> Section -> Finding*.
2.  **Git-like Audit Trail**: Who/What/When + Diffs.
3.  **Hybrid Entry**: Manual + Tool Import (via Triage Queue).
4.  **Mixed Asset Model**: S3 Backend + Virtual Finder UI.
5.  **Practitioner First**: Kanban Dashboard + GitHub Integration.
6.  **Smart Knowledge**: Proactive linking to Theory KB.
