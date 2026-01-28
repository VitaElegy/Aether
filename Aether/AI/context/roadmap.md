## Phase 1: Core Architecture (Priority: Highest)
> "Path A (Inside-Out)": Establish the foundation first.
- **Core Node Refactor**: Implement the `nodes` table and Class Table Inheritance pattern.
- **Data Portability**:
    - **Import**: Ability to upload a Folder structure and auto-convert to Knowledge Base Nodes.
    - **Export**: One-click export of entire Spaces as a `.zip` of Markdown files.
- **Frontend Architecture Refactor** [CRITICAL]:
    - **Task**: Refactor all data-mutating views (Memos, Vocab settings, User Profile) to STRICTLY use Composable-based state management (`use*` stores/composables) instead of ad-hoc Axios calls.
    - **Goal**: Eliminate Race Conditions akin to the `content_creation_race_condition` incident.


## Completed Feature: Git-like Article Versioning
> **Status**: [x] Implemented & Verified
> **Core Philosophy**: Content is immutable; updates create new snapshots.

### implemented Features
- [x] **Auto-Increment Versioning**: `v1` -> `v2` on every save where content hash changes.
- [x] **Change Reasons**: Users must provide a reason (e.g., "Fixed Typo") for updates.
- [x] **Full History**: API endpoints to fetch complete version history and specific snapshot details.
- [x] **Unified Diff**: Backend-calculated diffs (using `similar`) returning structured JSON for frontend rendering.
- [x] **Zero-Data Loss**: Atomic transactions ensure `nodes` and `content_versions` stay in sync.

## Completed Feature: Deep ReBAC Permission System
> **Status**: [x] Implemented & Verified
> **Core Philosophy**: granular, relationship-based access control (Zanzibar model).

### Implemented Features (Test Points)
- [x] **User Registration**: Auth system initializes users correctly (Ready for Subject ID).
- [x] **Tuple Store**: `relationships` table functions as the graph edge store.
- [x] **Automatic Ownership**: Creating an Article automatically creates `(node, owner, author)` tuple.
- [x] **Public Group Access**: Public content automatically creates `(node, viewer, group:public)` usage.
- [x] **Owner Write Access**: `owner` relation implicitly grants `write` permission (Recursive check).
- [x] **Guest Public Read**: Guest Users (Nil UUID) are members of `group:public` and can read Public content.
- [x] **Manual Overrides**: API permits manual tuple insertion for granular overrides.

### ⚠️ Rules for Future AI Development
**Any future modifications to the content system MUST adhere to these rules:**
1.  **Immutability**: NEVER update `content_versions` rows in place. Always insert a new row for a new version.
2.  **Reasoning**: All update APIs MUST support and propagate the `reason` field.
3.  **Structured Diffs**: The Diff API MUST return `Vec<DiffChange>` (JSON Array), NOT raw strings.
    - **Valid**: `[{"tag": "Insert", "value": "foo"}]`
    - **Invalid**: `"+++ foo"`
4.  **Version IDs**: Version identifiers are **Integers** (Stringified, e.g., "1"), NOT SemVer (`0.0.1`). Do not introduce SemVer parsing logic in the frontend.
## Phase 2: Self Space: English/Vocabulary
### 2.1 Core Concept
- **Learning System**: **FSRS (Free Spaced Repetition Scheduler)** algorithm for optimal retention.
- **Hybrid Data**: 
    - **Base**: **StarDict** local dictionary files (Server-side parsing). [IN PROGRESS]
    - **Extension**: User adds custom notes, parsing, and context.

### 2.2 Key Features
- **Smart Lookup**:
    - **StarDict Integration** [IN PROGRESS]: Backend service reads `.ifo`/`.dict`/`.idx` files to provide definitions without external API dependencies.
    - **Fuzzy Search** [IN PROGRESS]: Matches input against local dictionary database using fuzzy matching algorithms.
- **Contextual Linking**:
    - Link words to **Articles** (Source of origin).
    - Link words to **Example Sentences** (Nodes).
- **Review Mode**: FSRS-based review queue (Optimized intervals based on user feedback).

## Phase 3: Self Space: Memos
### 3.1 Core Concept
- **Structured Note-taking**: Not a flat "Tweet" stream, but structured documents. 
- **Organization**:
    - **Domains**: Can be organized by Topic/Domain (e.g., "Coding", "Cooking").
    - **Journal**: Can also act as a miscellaneous daily journal.
    - **Priority System**: First-class support for **Priority Levels** (High/Medium/Low).

### 3.2 Key Features
- **Flexible Filtering/Sorting Engine**: 
    - Instead of rigid views (Kanban), allow dynamic query combinations (e.g., `Priority=High AND Tag=Coding ORDER BY Date DESC`).
    - UI: List view with distinct badges, governed by a powerful filter bar.
- **Rich Editor**: Support for standard "Node" features (Markdown, Links).
- **Calendar & Management**:
    - **Time Management**: Support for `Scheduled Time` (Start), `Due Time` (Deadline), and `Reminder Time`.
    - **Status Flow**: "Pending" -> "Done" states.
    - **Visuals**: Digital Card format. Auto-truncate content or show explicit Title.
    - **Interaction**: Drag-and-drop on Calendar to reschedule.

## Completed Feature: Semantic Math Knowledge Base (Immersive Minisite)
> **Status**: [x] Implemented & Verified
> **Core Philosophy**: Immersive, Rigorous Ontology with Stratified Knowledge Graph.

### 1. Immersive Rendering & Layout
- [x] **Minisite Design**: Dedicated "Zen Mode" layout effectively removing chrome (sidebar/navbar) for deep focus.
- [x] **Visual Atmosphere**: Large background watermarks (e.g., `∫`), serif typography (`Noto Serif SC/Playfair`), and high-contrast semantic blocks.
- [x] **Semantic Blocks**: Distinct color-coded borders for Axioms (Red), Definitions (Blue), Theorems (Purple), and Proofs (Green).

### 2. Advanced Knowledge Graph (2D)
- [x] **Force-Directed DAG**: Physically accurate stratified layout (Axioms at top, Proofs at bottom) using `d3-force`.
- [x] **Geometric Semantics**: 2D shapes representing types (Triangle=Axiom/Proof, Square=Theorem, Circle=Definition).
- [x] **Interactive Navigation**: Click-to-focus and hover effects for dependency tracing.

### 3. Technical Rendering Engine
- [x] **Inline-Block Math**: Custom tokenizer to support `$$...$$` display math nesting inside semantic paragraphs without breaking flow.
- [x] **ASCII Tree Navigation**: Sidebar automatically parses ASCII art trees (`├─`) from content for structural navigation.
- [x] **Dynamic Hydration**: Automatic mounting of interactive components (Function Plots, Mermaid diagrams) within Markdown content.

- **General Knowledge Graph**: Expand Math graph logic to general Wiki links.
- **Search**: Meilisearch Integration (as defined in Spec).

## Phase 5: Math Knowledge Base Redesign (Current Focus)
> **Goal**: Create a rigorous, aesthetic, and immersive mathematical knowledge system.

### 5.1 Ontology & Structure ("Refined Hybrid")
- **First-Class Citizens**:
    - "Articles" act as narrative containers.
    - "Semantic Nodes" (Theorems, Axioms) acts as atomic units with independent Identity, Metadata, and Citation capability.
    - **Implementation**: Semantic Markdown (`::: theorem`) is parsed into a specialized index, allowing "Theorem 1.2" to be searched/linked independently of its parent file.

### 5.2 Topology & Visualization ("Computed Tree")
- **Data Model**: Directed Acyclic Graph (DAG) in backend.
- **Visual Model**: "Computed Tree" rendering.
    - Algorithm selects a "Primary Spanning Tree" for clean hierarchy.
    - Cross-links (DAG edges) are rendered as secondary/interactive connections to avoid visual clutter ("Spaghetti Graph").
    - **Aesthetics**: Move away from generic force-directed layouts. Use rigid, orthogonal, or architectural layouts appropriate for rigorous math.

### 5.3 UX & Navigation
- **Semi-Immersive Mode**:
    - Remove standard sidebar for focus.
    - **Retain Global Anchor**: Keep the "Back to Self Space" button in top-left to maintain context ("Website within a Website").

## Completed Feature: Global Navigation & Custom Dashboards
> **Status**: [x] Implemented & Verified
> **Core Philosophy**: Unified Navigation + Domain-Specific Immersion.

### 1. Global Beacon
- [x] **Contextual Morphing**: Single top-left element morphs between Logo (Home) and Arrow (Back).
- [x] **Integration**: Embedded in `TopNavBar` for seamless UI flow.

### 2. Custom KB Architecture
- [x] **Registry System**: `renderer_id` drives dynamic dashboard loading (`/kb/:id`).
- [x] **Math Dashboard (Mixed Mode)**:
    - **Visual Header**: D3 Axiom Tree + Daily Theorem.
    - **Content Body**: Full standard file management capabilities.

### 3. Dynamic Navigation Architecture
- [x] **Module Registration**: Modules can override Global Nav sections (`Center`, `Right`) using Teleport Portals.
- [x] **Context Awareness**: Navigation controls (e.g., Font Toggle, Tab Switcher) morph based on active context.

## Completed Feature: English KB Aesthetic ("London Academic")
> **Status**: [x] Implemented & Verified
> **Core Philosophy**: Visual Silence, Serif Typography (Playfair/Crimson), Warm Paper Tones.

### 1. Visual Design
- [x] **Strict Scoping**: Frontend firewall ensures English KB manifests remain pure.
- [x] **Immersive Reader**: `EnglishArticleAnalyzer` with contextual controls (Font Toggle) integrated into Global Nav.
- [x] **Tone**: Rejection of "SaaS" look; adherence to "Physical Book" fidelity.

## Phase 9: KB Protocol Standardization (In Progress)
> **Goal**: Establish a unified "Universal Parser" and Navigation Protocol for all Knowledge Bases.

### 9.1 KB Parser Interface
- [ ] **Unified AST**: Block-First Architecture (JSON) with Document-Level Versioning.
- [ ] **Schema Registry**: Strict validation for custom blocks (e.g., Math, Code).
- [ ] **Data Model Constraints**: 
    - **Explicit Indexing**: Blocks must implement `to_searchable_text()`.
    - **Asset Management**: Reference counting for images/attachments.
    - **Error Handling**: Fail-safe "Error Block" rendering.

### 9.2 Navigation & Shell Protocol
- [ ] **Multi-Stack Engine**: Tabs retain their internal route state (Keep-Alive).
- [ ] **Smart Back Logic**: "Hierarchical Up" logic overrides chronological history when appropriate.
- [ ] **Reactive Context**: Plugins expose reactive context to drive Top Bar actions.
- [ ] **Frontend Constitution**: Strict "Shell vs Canvas" layout constraints.
