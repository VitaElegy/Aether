# Aether Presentation Specification (`ppt_spec.md`)

This document outlines the structure, content, and functionality of the presentation module (`PresentationContainer.vue`) within the Aether project.

**Current File Path**: `backend/AI/ppt_spec.md` (Note: based on user request to be in "AI" folder)
**Frontend Location**: `frontend/src/components/self-space/presentation`

## Overview

The presentation is a Vue 3 component designed to run as an overlay within the application, providing a technical walkthrough of the Aether project. It uses `PresentationContainer.vue` as the orchestrator to manage slide transitions and navigation.

**Global Navigation**:
- **Keyboard**:
    - `→` / `Space`: Next Slide
    - `←`: Previous Slide
    - `Esc`: Exit Presentation
- **Mouse**:
    - Floating controls at bottom (Prev/Next)
    - Exit button at top-right

---

## Slide Inventory

### 1. Cover (`SlideCover.vue`)
*   **Purpose**: Title screen and project identification.
*   **Content**:
    *   Title: "Aether" (Large gradient text)
    *   Subtitle: "数字意识平台" (Digital Consciousness Platform)
    *   Metadata: Project Type (Course Design/Tech Share), Version (1.0.0).
    *   Hint: Press Space/Arrow to start.
*   **Animations**: Text fade-in, pulsing start hint.

### 2. Intro (`SlideIntro.vue`)
*   **Purpose**: Problem statement and motivation ("Why Aether?").
*   **Content**:
    *   Card 1: "混乱现状" (Chaos) - Fragmented tools.
    *   Card 2: "历史遗失" (Lost History) - Lack of real version control.
    *   Card 3: "解决方案" (Solution) - Structured, version-controlled platform.
*   **Visuals**: Glass-morphism cards with icons.

### 3. Development Process (`SlideDevProcess.vue`)
*   **Purpose**: Highlighting the unique AI-driven development methodology.
*   **Interactive Elements**:
    *   **Clickable Workflow Nodes**: Clicking Spec, Context, Agent, Delivery, or Error Node opens a detailed modal.
    *   **Modal Content**: Explains the "Human-AI" collaboration protocol (e.g., Constitution, Verification First, Error Distillation).
*   **Content**:
    *   **Visual Flow**: Spec/Context -> AI Agent (CoT) -> Code/Docs -> Feedback Loop (Error Logs).
    *   **Agent Core**: Central pulsing "AI Agent" node representing the engine.
    *   **Stats**: 100% AI Code, 32+ Error Logs, 3 Core Specs.
    *   **Philosophy**: "Constitution" (project_spec) + "Context" (roadmap) = Reliable Agent.

### 4. Authors (`SlideAuthors.vue`)
*   **Purpose**: Credit the creators.
*   **Content**:
    *   **Title**: "Authors".
    *   **Cards**:
        *   **Elegy**: Project Lead / Architect.
        *   **Gemini3Pro**: AI Co-Pilot / Assistant.

### 5. Concepts (`SlideConcepts.vue`)
*   **Purpose**: Explaining the core architectural metaphor (Kernel vs. User Space).
*   **Interactive Elements**:
    *   **Layer Diagram**: Clickable layers (Kernel, FS, User).
    *   **Detail Panel**: Clicking a diagram element opens a side panel with details and code snippets.
*   **Data Points**:
    *   **Node**: "Everything is a Node" concept (Class Table Inheritance).
    *   **ReBAC**: Relationship-Based Access Control logic.
    *   **Kernel**: Hexagonal Architecture (Domain vs Infra).
    *   **API**: Axum Interface Layer as "System Calls".
    *   **UserSpace**: Vue 3 Frontend as "User Space".
    *   **Search**: FST/LRU optimization explanation.

### 6. Features (`SlideFeatures.vue`)
*   **Purpose**: Showcasing the core functional innovations of the platform.
*   **Interactive Elements**:
    *   **Feature Grid**: 4 clickable cards covering Versioning, Knowledge, Permission, and Search.
    *   **Detail Modal**: In-depth explanation of technical capabilities.
*   **Content**:
    *   **Git-like Versioning**: Immutable history, semantic hashing, audit logs.
    *   **Knowledge Matrix**: Multi-level hierarchy (KB -> Folder -> Page), Article-KB linkage, Tags system, Bidirectional linking.
    *   **ReBAC Permissions**: Relation-based access control, inheritance, granular sharing.
    *   **Search Engine**: Intelligent discovery, weighted relevance (Title > Tags > Body), hybrid retrieval.
    *   **English Mastery**: Local dict deployment (StarDict), Multi-dict aggregation, Custom vocabulary/sentence builder, FSRS memory algorithm.

### 7. Aesthetics (`SlideAesthetics.vue`)
*   **Purpose**: Demonstrating the "Elegant & Minimalist" design philosophy.
*   **Content**:
    *   **Visual Elements**: Glassmorphism cards, refined Typography (serif/sans mix).
    *   **Philosophy**: Less is More, Fluid Motion, Dark Mode.
    *   **Interactive Demo**: Hover effects on glass cards and typography showcase.

### 8. Tech Stack (`SlideTechStack.vue`)
*   **Purpose**: Showcasing the technologies used.
*   **Interactive Elements**:
    *   **Clickable List Items**: Clicking a technology opens a modal with "Role", "Reason", and "Usage".
*   **Content**:
    *   **Backend**: Rust, Axum, SeaORM, Tokio.
    *   **Frontend**: Vue 3, TypeScript, TailwindCSS, Pinia.

### 9. Design (`SlideDesign.vue`)
*   **Purpose**: Deep dive into system design and specs.
*   **Interactive Elements**:
    *   **Tabs**: Switch between 4 views: **Architecture**, **Modularity**, **Data Model**, **Logic Flow**, **API Matrix**.
*   **Views**:
    *   **Architecture**: Visualization of the 5-layer architecture (Frontend -> Interface -> Domain -> Infra -> Data).
    *   **Modularity (Self Space)**: Visualizing the "Kernel + Plugins" structure. Explains strict decoupling via Rust Features and Hexagonal Ports (Core vs. English/KB/Comment/Export).
    *   **Data Model**: Class diagrams showing relationships (Node -> Article/vocab, User, Comment).
    *   **Logic Flow**: Animated SVG flow diagram showing the path of a request (Client -> Auth -> ReBAC -> Core -> DB/Search).
    *   **API**: Complete matrix of RESTful endpoints organized by domain.

### 10. Algorithms (`SlideAlgorithms.vue`)
*   **Purpose**: Explaining complex technical implementations and performance mechanisms.
*   **Interactive Elements**:
    *   **Grid of 5 Cards**: Versioning, ReBAC, Myers' Diff, English Engine, Caching.
    *   **Detail Modal**: Clicking a card reveals technical depth, visual diagrams, or code snippets.
*   **Content**:
    *   **Git-like Versioning**: DAG visualization, Semantic Hash.
    *   **ReBAC Permission**: Google Zanzibar model, recursive graph traversal logic, O(1) admin bypass.
    *   **Myers' Diff**: Shortest Edit Script concept, O(ND) complexity, Red/Green visualization.
    *   **English Engine**: FST (Finite State Transducer) for prefix matching, FSRS for memory scheduling, Polyglot dictionary aggregation.
    *   **Caching**: Server-side drafts (Redis/DB separation), Moka cache (TinyLFU) for dictionary, SeaORM connection pooling.

### 11. Roadmap (`SlideRoadmap.vue`)
*   **Purpose**: Future development plans.
*   **Content**: Vertical timeline.
    *   **Phase 1 (Completed)**: Core Kernel: Hexagonal Arch, ReBAC, Versioning.
    *   **Phase 2 (Current)**: Self Space: English Engine, FSRS, Local Dictionaries.
    *   Phase 3 (Next Step): Knowledge Graph & Structured Memos.
    *   Phase 4 (Future): Ecosystem: Public Sharing & Advanced Search.

### 12. Commits (`SlideCommits.vue`)
*   **Purpose**: Visualize the project's evolution.
*   **Content**:
    *   **Title**: "版本历史 (Commits)".
    *   **Visualization**: Vertical timeline.
    *   **Items**:
        *   Version Tag (v1, v2).
        *   Timestamp.
        *   Commit Message (Reason).
        *   Author.
    *   **Interactive**: Scrollable list showing the "Git-like" history.

### 13. Closing (`SlideClosing.vue`)
*   **Purpose**: End of presentation.
*   **Content**:
    *   "Thank You" (Animated large text).
    *   "Q & A".
    *   Signature: "Designed by Aether Team".

---

## Maintenance Notes
- **To add a new slide**:
    1. Create `SlideNew.vue` in `frontend/src/components/self-space/presentation/slides/`.
    2. Import it in `PresentationContainer.vue`.
    3. Add it to the `slides` array in the desired position.
- **To update content**: Edit the specific `.vue` file. content is generally hardcoded for this specific presentation.
