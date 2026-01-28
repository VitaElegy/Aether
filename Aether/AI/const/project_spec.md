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

| Feature Domain         | Protocol File                             | Key Rules                                     |
| :--------------------- | :---------------------------------------- | :-------------------------------------------- |
| **New Knowledge Base** | `AI/context/guides/new_kb_protocol_v2.md` | Schema, Search Trait, Quarantine, Migrations. |
| **API Endpoint**       | (Pending)                                 | REST naming, Error Types.                     |
| **UI Component**       | (Pending)                                 | Lazy loading, A11y, Theming.                  |

---

## ⚖️ 2. The Constitution (Immutable Laws)

These rules apply **globally** and cannot be overridden by specific skills.

### 2.1 Operational Directives
1. **Zero Panic**: Backend must never panic.
2. **Zero Regression**: You generally **MUST NOT** repeat any error listed in `AI/memory/ERROR_LOG.md`.
3.  **Governance Maintenance**: New Specs/Plans MUST be saved to `AI/context/specs/` or `.agent/workflows/`.
4.  **Protocol Discovery**: Check the **Protocol Registry** (Section 1.1) and `.agent/workflows/` before starting new features.
5.  **Bilingualism**: English and Chinese.

### 2.2 Directory Structure (Source of Truth)

```
Aether/
├── AI/                        # Agent Knowledge
│   ├── const/                 # Constitution, Templates
│   ├── skills/                # [NEW] Modular Skill Files
│   ├── context/               # State & Specs
│   └── memory/                # Logs & Discussions
├── backend/src/               # Rust Source
├── frontend/src/              # Vue Source
├── scripts/                   # Centralized Tools
│   ├── audit/                 # [NEW] Enforcers
│   ├── scaffold/              # [NEW] Generators
│   └── backend/frontend/utils/
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
