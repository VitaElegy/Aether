# Discussion: KB Parser Protocol Standardization
**Date**: 2026-01-21
**Participants**: User, Aether Architect (AI)

## Context
Refining the Knowledge Base (KB) architecture to strictly define how different KBs (English, Math, VRKB) parse and render their content.

## Log

### Q1: Data Source Purity
**Question**: Should all KBs share a unified, structured Intermediate Representation (IR/AST), or allow ad-hoc storage/parsing maps?
**Decision**: **Unified AST (Option B)**.
**Rationale**: 
- A unified AST (JSON-based) is required to ensure consistent search, versioning, and cross-referencing.
- All specific parsers (Markdown, LaTeX) must output to this standard structure.

### Q2: Computation Ownership
**Question**: Frontend Parsing (Thick Client) vs Backend Parsing (Thick Server)?
**Decision**: **Hybrid Approach**.
**Rationale**:
- User wants the best of both worlds: Instant editing feedback (Frontend) + Deep indexing/Reference integrity (Backend).
- Requires a synchronization protocol where Frontend handles display/edit, and Backend handles canonical storage and graph generation.

### Q3: Granularity (Block vs Document)
**Question**: Block-First (Notion-style) vs Document-First (Obsidian-style)?
**Decision**: **Block-First with Adapters**.
**Rationale**:
- **Core Storage**: Block-First (JSON AST) to enable fine-grained referencing (e.g., linking specific sentences/formulas) and "Everything is an Object".
- **Compatibility**: Must implement **Import/Export Adapters**.
    - Importers convert raw Markdown -> Block AST.
    - Exporters serialize Block AST -> Markdown for portability.

### Q4: Extension Protocol (Custom Blocks)
**Question**: Black Box (Option A) vs Registry (Option B)?
**Context**: Project is 100% AI-implemented.
**Analysis**:
- **AI Consistency**: AI agents perform better with **explicit constraints** (Option B) than implicit conventions (Option A).
- **Context Management**: A central Registry acts as a high-quality context source, allowing the AI to understand the entire data model by reading one file/directory, rather than hunting through distributed parser logic.
- **Hallucination Prevention**: Explicit Schemas (e.g., Zod/Rust Structs) prevent AI from "hallucinating" fields that don't exist in the database, catching errors at compile time.
**Decision**: **Option B (Strict Schema Registry)**.
**Rationale**: 
- We will define a `BlockSchema` trait/interface.
- Every KB must register its Block Schemas (e.g., `EnglishSentenceBlock`, `MathGraphBlock`) at startup.
- Core rejects any Block payload that doesn't validate against the registered Schema.

### Q5: Versioning Granularity
**Question**: Document-Level (Git-style) vs Block-Level (Notion-style)?
**Decision**: **Option A (Document-Level)**.
**Rationale**:
- **Consistency**: Aligns with the existing Git-like versioning system for standard articles.
- **Simplicity**: Avoids the extreme complexity of joining thousands of block versions for a single read operation.
- **Sufficiency**: For academic/knowledge work, "Snapshotting" the whole concept context is often more valuable than tracking atomic character changes.

### Q6: Cross-Reference Protocol
**Question**: Hyperlink (Weak) vs Transclusion (Strong)?
**Decision**: **Hyperlink Primary + Reserved Interface**.
**Rationale**:
- **Pragmatism**: Hyperlinks are easy to implement and cover 90% of use cases.
- **Future-Proofing**: We will define a `RenderableBlock` interface (Vue Component contract) but not enforce its implementation immediately. This reserves the architectural slot for future Transclusion support.

### Q7: Search Indexing Strategy
**Question**: Generic Indexing (Option A) vs Explicit Trait (Option B)?
**Decision**: **Option B (Explicit Indexing Trait)**.
**Rationale**:
- **Precision**: Avoids polluting the search index with system metadata (keys, ids, booleans).
- **Control**: Allows KBs to define exactly what is searchable (e.g., OCR text for math, translations for english).
- **Requirement**: Each Schema must implement `to_searchable_text()`.

### Q8: Asset Lifecycle
**Question**: Reference Counting (Option A) vs Unmanaged URLs (Option B)?
**Decision**: **Option A (Managed References)**.
**Rationale**:
- **Cleanliness**: Ensures zero "orphan files" taking up disk space.
- **Integrity**: Assets are tracked as first-class citizens in the dependency graph.
- **Workflow**: Upload -> Get AssetID -> Create Block with AssetID.

### Q9: Error Handling
**Question**: Strict/Panic (Option A) vs Lenient/Error Block (Option B)?
**Decision**: **Option B+ (Lenient with Custom Error View)**.
**Rationale**:
- **Resilience**: A single bad block should not crash the entire article.
- **Flexibility**: KBs can define how their errors look (e.g., a "Broken Image" icon for assets, a red text box for formulas).

### Q10: Schema Evolution
**Question**: Lazy/Read-time (Option A) vs Eager/Write-time (Option B)?
**Decision**: **Option B (Eager Migration)**.
**Rationale**:
- **Cleanliness**: Codebase stays free of `if (v1) ...` legacy logic.
- **AI-Friendliness**: AI agents perform much better when the data structure is unique and canonical.
- **Cost**: We accept the one-time cost of running a migration script when upgrading standard definitions.

### Q11: Frontend Layout Constitution
**Question**: Strict Internal Grid vs Free Canvas?
**Decision**: **Managed Shell, Free Canvas**.
**Rationale**:
- **Consensus**: Top Navigation and Bottom Dock are strictly managed by the Core (Standardization).
- **Freedom**: The "Main Stage" (Center Area) is a free canvas. The KB can render whatever it wants (WebGL, Kanban, Text Editor) within this frame.
- **Constraints**: KB must not break out of its container (no `position: fixed` overlaying the Top Bar).

### Q12: Context State Synchronization
**Question**: Push (Imperative) vs Pull (Reactive)?
**Decision**: **Option B (Reactive/Pull)**.
**Rationale**:
- **Cleanliness**: The plugin exposes an `activeContext` reactive object. The Shell watches it. When the component unmounts, the context naturally vanishes. No manually "resetting" the top bar.

### Q13: Persistence Contract
**Question**: Atomic Auto-save vs Global Signal?
**Decision**: **Hybrid (Global Signal + Internal Auto-save)**.
**Rationale**:
- **Consistency**: The Core can enforce a "Save All" command (e.g., on window close).
- **Performance**: Intricate editors (like a 3D scene) know best when to save incrementally to avoid data loss.

### Q14: Navigation Semantics (Back Button)
**Question**: Chronological (Browser) vs Hierarchical (App)?
**Decision**: **Option B (Hierarchical Up / Smart Back)**.
**Rationale**:
- **Behavior**: `if (internal_history) back() else up()`.
- **Logic**: Users expect "Back" to typically mean "Up to Parent" in a nested KB structure unless they just drilled down.
- **Implementation**: Requires a custom `useBackOverride` composable that plugins can use to define where "Up" is.

### Q15: Module State Persistence
**Question**: Reset vs Keep-Alive (Multi-Stack)?
**Decision**: **Option B (Multi-Stack Router)**.
**Rationale**:
- **Experience**: Switching tabs (modules) should not lose the user's place (scroll position, active sub-page).
- **Architecture**: We will implement a `RouteStackStore` that remembers the last active path for each module ID.
- **Flow**: Click Module Icon -> Check Store -> `router.push(saved_path || default_path)`.
