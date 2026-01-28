# English Knowledge Base Design & Implementation
**Date:** 2026-01-18
**Topic:** English Learning Module, Hybrid Anchoring, London Academic Aesthetics

## 1. Core Philosophy
The English module is designed as a **specialized Knowledge Base**, not a separate app. It leverages the core content system but adds specific metadata and UI layers to facilitate language learning.

## 2. Key Decisions

### A. Data Model (Backend)
- **Sentences**: Not stored as individual graph nodes (too complex/heavy). Instead, they are parsed and stored as a `derived_data` JSON blob (`sentence_map`) within the `ArticleDetail` table.
- **Vocabulary Association**:
  - `VocabularyExample` entities now store `article_id` (weak reference) and `sentence_uuid`.
  - This allows us to link a vocabulary word back to its specific context in an article without rigid foreign key constraints that would break on article updates.

### B. Sentence Stability ("Hybrid Anchoring")
- **Problem**: Markdown edits (fixing typos, reformatting) shouldn't break sentence annotations.
- **Solution**: `SentenceParser` service.
  - **Level 1**: Exact Content Hash (MD5).
  - **Level 2**: Fuzzy Match (Levenshtein Distance > 85%).
  - **Level 3 (Future)**: Context Anchoring (checking preceding/succeeding sentences).
- This ensures that even if a sentence is slightly edited, its UUID remains stable, preserving user annotations.

### C. Logic Flow
1. **Article Save/Update**: Triggers `SentenceParser`. Computes new `sentence_map`, tries to match UUIDs with previous map (via `derived_data`), and saves the new map.
2. **Add Example**: User clicks a word -> Frontend finds the sentence UUID -> Sends to Backend -> Backend saves `VocabularyExample` with that UUID.

### D. Frontend Aesthetics ("London Academic")
- **Style**: Depart from the "Tech/SaaS" look. aim for "London English" / "Academic Journal".
- **Typography**: Serif fonts (Playfair Display, Crimson Text) for headers and body.
- **Interaction**:
  - **Contextual Drawer**: Clicking a word opens a side drawer (paper texture, elegant typography) rather than a simple tooltip.
  - **Renderer**: `EnglishArticleAnalyzer` component specially handles text interaction.

## 3. Implementation Status
- [x] Backend Entities (`derived_data`, `sentence_uuid`).
- [x] `SentenceParser` with Hybrid Anchoring.
- [x] `add_example` API.
- [x] Frontend `EnglishArticleAnalyzer` & `AcademicDrawer`.
- [x] Renderer Selection UI in KB Settings.

## 4. Future Considerations
- **Context Anchoring**: Improve stability if fuzzy match fails.
- **Renderer Switching**: If we add more specialized KBs (e.g., Code Review, Math Problem Solving), genericize the renderer selector further.
