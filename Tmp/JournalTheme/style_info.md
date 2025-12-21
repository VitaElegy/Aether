# Journal / Traveler's Notebook Theme

This directory contains the Vue components that implement the "High-end Journal" aesthetic.

## Style Characteristics
- **Theme**: Warm Light Mode (Cream Paper & Saddle Brown Leather).
- **Metaphor**: An open leather-bound traveler's notebook on a wooden desk.
- **Typography**:
    - `Crimson Pro` (Serif) for headers and dates.
    - `Caveat` (Handwriting) for tags, signatures, and notes.
    - `Inter` (Sans-serif) for readable body text.
- **Visual Elements**:
    - 3D Book perspective effect.
    - Paper textures (background images).
    - Stamp-style avatars and dates.
    - "Mood" icons (Sun, Rain, Cloud) for entries.

## Files
- `App.vue`: Sets the warm color palette (`--td-brand-color: #8B4513`), loads the custom fonts from Google Fonts, and applies the wood texture background.
- `HomeView.vue`: The main interface. Displays the open book with navigation on the left page and the entry stream on the right page.
- `UserProfileView.vue`: A standalone "Identity Card" style page for viewing user profiles, accessible by clicking avatars or signatures.
- `router_index.ts`: The router configuration including the `/profile/:id` route.

## How to Restore
1. Copy `App.vue`, `HomeView.vue`, and `UserProfileView.vue` back to `Aether/frontend/src/views/` (and `App.vue` to `src/`).
2. Copy `router_index.ts` content back to `Aether/frontend/src/router/index.ts`.

