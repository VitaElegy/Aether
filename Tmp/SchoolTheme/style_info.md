# School Homework Theme

This directory contains the Vue components that implement the "School Homework / Student Planner" aesthetic.

## Style Characteristics
- **Theme**: Light Mode (Paper White & Academic Blue).
- **Metaphor**: A physical notebook/planner on a desk.
- **Typography**: `Patrick Hand` (handwriting) for notes, `Georgia` for headers, `Courier New` for inputs.
- **Visual Elements**:
    - Spiral binding on the left.
    - Lined paper backgrounds (`linear-gradient` CSS).
    - Sticky note cards with tape effects.
    - "Grade" stickers (A+, Good).
    - Ruled input fields (underline only).

## Files
- `App.vue`: Sets the global light theme, background grid texture, and TDesign token overrides for the "paper" feel.
- `LoginView.vue`: "Student Portal" login card with stacked paper effect and red notebook spine.
- `HomeView.vue`: The main "Planner" interface. Left spiral binding, subject tabs, and a grid of sticky note posts.
- `EditorView.vue`: Simple editor compatible with the light theme.

## How to Restore
Copy these files back to their respective locations in `Aether/frontend/src/` to restore this look.

