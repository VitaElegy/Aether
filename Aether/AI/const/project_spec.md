# Aether Project Specification & Agent System Prompt

> **CRITICAL INSTRUCTION**: This document is the "Constitution". For specific implementation details, you **MUST** load the relevant **Skill Modules** listed below.

---

## 📑 1. The Skill System (Modular Context)

Instead of reading one giant file, you must **dynamically load** the skills relevant to your task:

| Domain           | Skill File                  | Description                                          |
| :--------------- | :-------------------------- | :--------------------------------------------------- |
| **Backend**      | `AI/skills/backend_rust.md` | Rust, Axum, SeaORM, Error Handling, Versioning.      |
| **Frontend**     | `AI/skills/frontend_vue.md` | Vue 3, Composables, State Management, UI Guidelines. |
| **Database**     | `AI/skills/rebac_schema.md` | Schema, ReBAC, CTI Pattern, Renderer IDs.            |
| **Architecture** | `AI/skills/general_arch.md` | Global Nav, Mixed Mode, General Rules.               |

### 1.1 Protocol Registry (Feature Governance)
If your task involves one of these features, you **MUST** read the corresponding protocol:

| Feature Domain         | Protocol File                                   | Key Rules                                     |
| :--------------------- | :---------------------------------------------- | :-------------------------------------------- |
| **New Knowledge Base** | `AI/context/guides/new_kb_protocol_v2.md`       | Schema, Search Trait, Quarantine, Migrations. |
| **API Endpoint**       | `AI/context/api_reference.md`                   | OpenAPI, Utoipa, Detailed Errors.             |
| **Navigation**         | `AI/context/specs/navigation_lifecycle_spec.md` | Loading State, Keep-Alive, Crash Boundary.    |
| **Permission**         | `AI/skills/rebac_schema.md`                     | **STANDARD**: Zanzibar-style Tuples, Groups.  |
| **UI Component**       | (Pending)                                       | Lazy loading, A11y, Theming.                  |

### 1.2 Runbook Registry (Step-by-Step Operations)
For common operations, follow the **exact checklist** in these runbooks:

| Operation            | Runbook File                     | Key Checkpoints                   |
| :------------------- | :------------------------------- | :-------------------------------- |
| **Add DB Entity**    | `AI/runbooks/database_entity.md` | Migration → Entity → Repository   |
| **Add API Endpoint** | `AI/runbooks/api_endpoint.md`    | DTO → Handler → Route → Frontend  |
| **Create SKB**       | `AI/runbooks/skb_creation.md`    | Spec → Backend → Registry → UI    |
| **Fix Bug**          | `AI/runbooks/bug_fix.md`         | Reproduce → Test → Fix → Document |

> **Index**: See [`AI/runbooks/RUNBOOK_INDEX.md`](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/runbooks/RUNBOOK_INDEX.md) for full list.

---

## ⚖️ 2. The Constitution (Immutable Laws)

These rules apply **globally** and cannot be overridden by specific skills.

### 2.1 Operational Directives
1. **Zero Panic**: Backend must never panic.
2. **Zero Regression**: You generally **MUST NOT** repeat any error listed in `AI/memory/ERROR_LOG.md`.
3.  **Governance Maintenance**: New Specs/Plans MUST be saved to `AI/context/specs/` or `.agent/workflows/`.
4.  **Protocol Discovery**: Check the **Protocol Registry** (Section 1.1) and `.agent/workflows/` before starting new features.
5.  **Bilingualism**: English and Chinese.
6.  **Compliance Audit**: You MUST run `npm run audit:kb` (or `npx -y tsx scripts/ai/audit_kb_specs.ts`) before marking any Frontend Dashboard task as complete.

### 2.2 Decision Heuristics

When unsure how to proceed:

1.  **Spec Lookup**: Check [`AI/const/SPEC_INDEX.md`](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/const/SPEC_INDEX.md) to find the authoritative (Active) spec for a domain.
2.  **Conflict Resolution**: If specs conflict, the **newer Active spec** wins. Deprecated specs are for reference only.
3.  **DB Changes**: If your change touches DB schema, you **MUST** add a migration file AND update `ERROR_LOG.md` with the migration name.
4.  **Missing Protocol**: If no protocol exists for your task, create a draft at `AI/context/specs/` and notify the user before proceeding.
5.  **Ambiguous Requirement**: Ask clarifying questions. Do NOT guess.

### 2.3 Directory Structure (Source of Truth)

```
Aether/
├── AI/                        # Agent Knowledge
│   ├── const/                 # Constitution, SPEC_INDEX, Templates
│   ├── skills/                # Modular Skill Files
│   ├── context/               # State, Specs, Guides
│   └── memory/                # Logs, Discussions, Error Reports
├── backend/src/               # Rust Source
├── frontend/src/              # Vue Source
├── scripts/                   # Centralized Tools
│   ├── ai/                    # AI Agent Tools (audit_kb_specs.ts)
│   ├── scaffold/              # Generators (new_kb.py)
│   ├── backend/               # Backend utilities
│   └── utils/                 # Shared utilities
├── doc/                       # Human Docs
└── .agent/workflows/          # Execution Plans
```

---

## 3. Communication Protocol

1.  **Complex Function Discussion**:
    -   **Stop & Discuss**.
    -   **Deep Inquiry (5-10 Questions)**.
    -   **Sequential Interaction** (One by One).
    -   **Record & Sync** to `AI/memory/discussions/`.

---

**End of Spec**

