# Knowledge Base Parser Interface Specification

**Status**: DRAFT (In Design/Interview Phase)
**Objective**: Define a rigorous, standardized interface for parsing different content types (English, Math, VRKB, etc.) within Aether.


## Design Decisions

### 1. Unified Intermediate Representation (IR)
**Constraint**: All Knowledge Base parsers **MUST** output a standardized JSON AST.
**Input**: Raw Source (Markdown, LaTeX, etc.)
**Output**: `AetherAST` (Standard JSON Structure)

The system rejects any design that relies on ad-hoc frontend parsing of raw strings without a corresponding structured representation.

### 2. Block-First Architecture
**Constraint**: The fundamental unit of storage is a **Block**, not a Document.
- A Document is an ordered list (or tree) of Blocks.
- Each Block has a unique, immutable UUID.

### 3. I/O Adapters
**Requirement**: Every Parser Implementation must provide:
- `parse(raw_content) -> AST`: For importing legacy/external formats.
- `serialize(AST) -> raw_content`: For exporting to portable formats (Markdown, LaTeX).

### 4. Schema Registry
**Constraint**: All Block types must be explicitly registered.
- The Core System maintains a `SchemaRegistry`.
- KBs must contribute structural definitions (Schema) for their custom blocks.
- **Validation**: The system performs strict schema validation on all Block payloads before persistence.

### 5. Versioning Strategy
**Constraint**: **Document-Level Snapshots**.
- While storage is Block-based, versioning is applied to the **Manifest** (the list of blocks that make up a document).
- Changing a block creates a new revision of the *Document*, not just the block. (Or effectively Copy-on-Write for the document structure).

### 6. Reference Protocol
**Constraint**: **Hyperlink-First**.
- The primary mechanism for cross-KB linking is the **Internal URI** (`aether://<kb_id>/<block_id>`).
- **Reserved Interface**: The architecture defines a `BlockRenderer` interface.
    - *Purpose*: To allow future "Transclusion" (embedding).
    - *Status*: Optional for now. If a KB does not implement this, the system falls back to a generic link card.

### 7. Search Protocol
**Constraint**: **Explicit Indexing Trait**.
- Every Block Schema MUST implement a method (e.g., `get_searchable_text()`) that returns a clean string.
- The Core System will ONLY index the output of this method, ignoring the raw JSON payload structure.

### 8. Asset Management
**Constraint**: **Managed Reference Counting**.
- **No Raw URLs**: Blocks are forbidden from storing direct file paths (e.g., `/uploads/img.png`) in persistent storage.
- **Asset IDs**: Blocks must store `AssetID` (UUID).
- **Logic**: The Core System maintains a reference count of `(BlockID -> AssetID)`.
    - When a Block is deleted, ref_count--.
    - When ref_count == 0, the physical file is deleted asynchronously.

### 9. Error Handling
**Constraint**: **Fail-Safe Rendering**.
- Parsers MUST NOT crash the application on invalid input.
- Parsers MUST return a structured `ErrorBlock` when parsing fails.
- The UI MUST render this `ErrorBlock` (allowing KBs to customize the error display), preserving the ability to edit/fix the source.

### 10. Schema Evolution
**Constraint**: **Write-Time Migration**.
- The `SchemaRegistry` enforces the *current* version.
- Legacy data MUST be migrated to the current schema *before* it can be loaded/edited by the application (or via an async migration background job).
- The codebase assumes all objects match the current schema.

### 11. Layout Protocol
**Constraint**: **Managed Shell, Free Canvas**.
- **Reserved Areas**: Top Bar and Bottom Dock are managed by `SelfSpaceView` and strictly conform to the Plugin Protocol.
- **Main Stage**: The central area is delegated entirely to the Plugin's Component.
- **Style Isolation**: Plugins SHOULD use Scoped CSS or Aether Design Tokens. Plugins MUST NOT modify global `body` or shell z-indices.

### 12. Context Protocol
**Constraint**: **Reactive Context**.
- Plugins expose a reactive `context` object (e.g., via `defineExpose` or a shared composable).
- The Shell (SelfSpaceView) observes this context to update the Top Bar Actions automatically.

### 13. Persistence Protocol
**Constraint**: **Hybrid Strategy**.
- **Interface**: Plugins MUST implement a `save(): Promise<void>` method.
- **Behavior**: 
    - Plugins SHOULD implement internal auto-save (debounce).
    - Plugins MUST respond to the global `save()` signal from the Shell.

### 14. Navigation Protocol
**Constraint**: **Smart Hierarchical Back**.
- The "Back" button MUST prioritize Hierarchical navigation (returning to the parent context) over chronological history if the history chain is broken or irrelevant.
- Plugins SHOULD provide a `backRoute` or `parentRoute` hint in their Context.

### 15. Session Protocol
**Constraint**: **Multi-Stack State**.
- The Shell MUST persist the "Last Active Route" for each Plugin.
- **Behavior**: Switching away from a Plugin and returning to it MUST restore the precise view state (URL), not reset to the root.
