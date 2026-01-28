# Design Discussion: Global Navigation Beacon & Custom Knowledge Base Architectures

**Date**: 2026-01-14
**Topic**: Unified Navigation and Domain-Specific Knowledge Entry Points

## 1. Context & Motivation
As Aether grows from a simple blog to a complex Personal Knowledge Management (PKM) system, two UX friction points emerged:
1.  **Navigation Redundancy**: "Back" buttons were scattered across different views (`ReadView`, `SelfSpace`), creating visual clutter and inconsistent behavior.
2.  **Generic Entry Points**: All Knowledge Bases looked the same (file lists), failing to capture the unique "nature" of different domains (e.g., Mathematics requires structure visualization, while a Diary requires a timeline).

## 2. Global Navigation Beacon
### The Concept: "Contextual Morphing"
We moved away from static breadcrumbs or persistent sidebars to a minimalist "Beacon" concept.
- **State A (Rest)**: A simple Logo (Aether Triangle). Represents "Home" / "Root".
- **State B (Hover)**: Expands to reveal a Back Arrow. Represents "History".

### Evolution
- **Initial Idea**: A floating, fixed element in the top-left corner.
- **User Feedback**: "Floating feels disjointed (割裂). It should be integrated."
- **Final Design**: identifying the top-left area of the `TopNavBar` as the natural home for this beacon. It is now embedded in the layout flow, maintaining the "Contextual Morphing" behavior but feeling physically grounded in the app structure.

### Technical Implementation
- **Component**: `GlobalBeacon.vue`
- **Logic**: Uses `window.history` for smart back navigation. If history is empty, it falls back to URL path ascent.
- **Integration**: Placed in `TopNavBar.vue`, replacing the static logo. Legacy back buttons in specific views were removed.

## 3. Custom Knowledge Base Architecture
### The Goal: "Mixed Mode" Dashboards
We wanted "Immersive Entry Points" without losing utility. A Math KB should *feel* like a math workspace, but still allow you to edit files.

### Architecture: The Registry Pattern
- **Route**: `/kb/:id`
- **Logic**: The view checks the `renderer_id` of the Knowledge Base.
- **Registry**: `read_layout_registry.ts` maps IDs (e.g., `math_v1`) to Vue components.

### Implementation: Math Dashboard (`math_v1`)
Designed as a "Mixed Mode" interface:
1.  **Visual Header (Immersive)**:
    - **Axiom Tree**: A D3.js visualization of the logical hierarchy (Logic -> Set Theory -> Number Systems).
    - **Theorem of the Day**: A discovery card (DailyTheorem) to surface random content and spark curiosity.
2.  **Content Body (Utility)**:
    - A standard grid/list of files and folders using `KnowledgeBaseLayout`.
    - Retains all CRUD capabilities (Create, Delete, Settings).

## 4. Conclusion
This architecture allows Aether to scale indefinitely. We can now add:
- **Cinema Dashboard**: For movie reviews (Poster Wall).
- **Diary Dashboard**: For journals (Calendar Heatmap).
- **Code Dashboard**: For snippets (Tag Cloud).

The Global Beacon ensures that no matter how deep or custom the specific dashboard is, the user always has a consistent, reliable anchor to return Home or Step Back.
