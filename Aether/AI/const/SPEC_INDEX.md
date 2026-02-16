# Aether Specification Index

> **Purpose**: Single source of truth for all specification statuses.
> **Last Updated**: 2026-02-09

## Status Definitions
| Status          | Meaning                                      |
| :-------------- | :------------------------------------------- |
| **Active**      | Current authoritative spec for this domain   |
| **Deprecated**  | Superseded by newer spec, kept for reference |
| **Draft**       | Under development, not finalized             |
| **Implemented** | One-time plan that has been executed         |

---

## Architecture & Core

| Spec                                                                                                                                    | Status     | Domain     | Notes                      |
| :-------------------------------------------------------------------------------------------------------------------------------------- | :--------- | :--------- | :------------------------- |
| [kb_architecture_v3.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/kb_architecture_v3.md)               | **Active** | KB Core    | "OS Model" - supersedes V2 |
| [kb_architecture_v2.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/kb_architecture_v2.md)               | Deprecated | KB Core    | Superseded by V3           |
| [unified_data_model.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/unified_data_model.md)               | **Active** | Data Model | UObject pattern            |
| [navigation_topology_v1.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/navigation_topology_v1.md)       | **Active** | Navigation | Smart Trace algorithm      |
| [navigation_lifecycle_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/navigation_lifecycle_spec.md) | **Active** | Navigation | Loading/Error states       |

---

## Self Space & Frontend

| Spec                                                                                                                                      | Status      | Domain     | Notes                        |
| :---------------------------------------------------------------------------------------------------------------------------------------- | :---------- | :--------- | :--------------------------- |
| [self_space_architecture_v2.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/self_space_architecture_v2.md) | **Active**  | Self Space | "Silky" standard             |
| [self_space_refactor_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/self_space_refactor_spec.md)     | Implemented | Self Space | Execution details, completed |
| [editor_v3_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/editor_v3_spec.md)                         | Draft       | Editor     | Pending implementation       |

---

## Special Knowledge Bases

| Spec                                                                                                                    | Status     | Domain | Notes                     |
| :---------------------------------------------------------------------------------------------------------------------- | :--------- | :----- | :------------------------ |
| [memos_module.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/memos_module.md)           | **Active** | SKB    | Memos KB design           |
| [paper_v1_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/paper_v1_spec.md)         | **Active** | SKB    | Paper research KB         |
| [paper_metadata_v1.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/paper_metadata_v1.md) | **Active** | SKB    | Paper metadata schema     |
| [vr_kb_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/vr_kb_spec.md)               | **Active** | SKB    | Vulnerability research KB |
| [prkb_v2_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/prkb_v2_spec.md)           | **Active** | SKB    | Paper reading KB V2       |
| [ppt_spec.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/ppt_spec.md)                   | Draft      | SKB    | Presentation KB           |

---

## Data & Portability

| Spec                                                                                                                                  | Status     | Domain      | Notes            |
| :------------------------------------------------------------------------------------------------------------------------------------ | :--------- | :---------- | :--------------- |
| [asset_kb_spec_v1.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/asset_kb_spec_v1.md)                 | **Active** | Assets      | My Assets system |
| [kb_backup_protocol_v1.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/kb_backup_protocol_v1.md)       | **Active** | Backup      | .akb format      |
| [data_portability_spec_v1.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/data_portability_spec_v1.md) | **Active** | Portability | Export/Import    |
| [kb_parser_interface.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/specs/kb_parser_interface.md)           | **Active** | Parser      | Block parsing    |

---

## Skills (Always Active)

| Skill                                                                                                  | Domain       |
| :----------------------------------------------------------------------------------------------------- | :----------- |
| [backend_rust.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/skills/backend_rust.md) | Backend      |
| [frontend_vue.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/skills/frontend_vue.md) | Frontend     |
| [rebac_schema.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/skills/rebac_schema.md) | Permissions  |
| [general_arch.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/skills/general_arch.md) | Architecture |

---

## Protocols (Guides)

| Guide                                                                                                                      | Domain            |
| :------------------------------------------------------------------------------------------------------------------------- | :---------------- |
| [new_kb_protocol_v2.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/context/guides/new_kb_protocol_v2.md) | Creating new SKBs |
