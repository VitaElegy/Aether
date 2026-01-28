# Special Knowledge Base (SKB) Navigation & Template Design - Deep Inquiry

**Date**: 2026-01-28
**Participants**: User, Senior Architect (Elegy)
**Goal**: Define the interaction model for "Pinning" and "Entering" special knowledge bases to resolve navigation confusion.

---

## Session Log

### Q1: Dock Icon Semantics
**Choice**: **View Injection**.

### Q2: Registration Protocol
**Choice**: **Instance Override**.

### Q3: Surface Architecture
**Choice**: **Monolithic Black Box**.

### Q4: State Persistence (URL)
**Choice**: **Deep Linking**.

### Q5: Data Boundary (Search)
**Choice**: **Strict Context**.

### Q6: Associations (Cross-linking)
**Choice**: **Hybrid Strategy**.

### Q7: Data Sovereignty
**Choice**: **Database First**.

### Q8: Dock Pinning Norms
**Choice**: **Strict Auto-Stacking**.
- **Rule**: Multiple instances of the same Template MUST be grouped.
- **Experience**: 2nd instance triggers "Stack Mode".

### Q9: Registration Manifest Norms
**Choice**: **Rich Manifest (Enforced by Spec)**.
- **Philosophy**: "Convention over Configuration".
- **Implementation**: We do not write rigid validator code. Instead, we write a **Strict Specification** that AI Agents must follow when creating new modules.
- **Requirement**: Every Plugin must provide a standard `plugin_manifest.ts` export defining its capabilities, routes, and settings schema.

