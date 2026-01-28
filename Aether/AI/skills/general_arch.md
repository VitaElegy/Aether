# Skill: General Architecture & Rules

## 1. Global Navigation and Layout
### 1.1 Global Beacon
- **Concept**: Top-left Unified Anchor.
- **Rule**: NO other back buttons allowed. Beacon is logic source of truth.

### 1.2 Dashboard Registry ("Mixed Mode")
- Custom Dashboards (Minisites) MUST wrap standard content management.

## 2. Project Organization
- **Directory Structure**: See `AI/const/project_spec.md` for the directory map.
- **Scripts**: All scripts centralized in `scripts/`.
- **Docs**: Human docs in `doc/`.

## 3. Communication & Governance
- **Bilingualism**: English & Chinese.
- **Governance**: New plans must go to `AI/context/specs/` or `.agent/workflows/`.
- **Protocol Discovery**: Check `.agent/workflows/` before inventing process.
- **Discussion**: 5-10 deep questions for complex tasks.
