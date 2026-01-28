---
description: Create a new specialized Knowledge Base
---
# Workflow: Create Specialized Knowledge Base

## 1. Requirement Definition
- [ ] Define the **Domain** (e.g., Math, Music, Code).
- [ ] Define the **Renderer ID** (e.g., `music_v1`).
- [ ] Requirements gathering conversation with User.

## 2. Backend Implementation (Rust)
- [ ] Create Migration: Add `renderer_id` support if needed (already exists).
- [ ] Define Domain Schema (if using Semantic Markdown or JSON blocks).

## 3. Frontend Implementation (Vue)
- [ ] Create Layout: `frontend/src/views/kb/layouts/MusicLayout.vue`.
- [ ] Register Layout: Update `frontend/src/services/LayoutRegistry.ts`.
- [ ] Implement Components: `MusicPlayer.vue`, `ScoreViewer.vue`.

## 4. Verification
- [ ] scripts/backend/run_tests.sh
- [ ] Manual Check: Create KB with ID `music_v1` and verify layout loads.
