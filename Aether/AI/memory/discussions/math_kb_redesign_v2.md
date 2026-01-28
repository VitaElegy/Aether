# Math Knowledge Base Redesign V2: Specification

**Date**: 2026-01-14
**Status**: APPROVED

## 1. Core Philosophy
> "Rigorous, Elegant, Atomic."

The system is not just a viewer; it is a tool for **Mathematical Knowledge Management (MKM)**. It must balance narrative flow (Articles) with logical precision (Theorems).

## 2. Ontology: "Refined Hybrid"
The user selected **Option B: Refined Hybrid**.

### Concept
- **The Article**: The "File". A linear narrative (e.g., "Introduction to Group Theory").
- **The Node**: The "Atom". A specific logical statement (e.g., "Definition 1.1: Group").
- **First-Class Citizenship**:
    - Nodes live *inside* Markdown files but are indexed *globally*.
    - A Node has a stable ID (e.g., `thm:group_uniqueness`).
    - Users can specificially "Link to Theorem 1.1" rather than just "Link to Article".

### Technical Strategy
- **Input**: Semantic Markdown.
  ```markdown
  ::: theorem {id="thm:unique_identity" title="Uniqueness of Identity"}
  In any group $G$, the identity element $e$ is unique.
  :::
  ```
- **Indexing**: A background watcher parses these blocks and populates a separate `semantic_nodes` index (in-memory or DB).

## 3. Topology: "Computed Tree"
The user selected **Option B: Computed Tree**.

### Concept
- **Reality**: Math is a DAG (A theorem relies on 3 lemmas).
- **Presentation**: Users crave hierarchy (A Tree).
- **Algorithm**:
    - **Primary Parent**: Every node has one "Canonical" location (e.g., The chapter it was defined in).
    - **Visual**: Render the "Canonical Tree" (Folder -> File -> Main Theorems).
    - **Cross-Links**: "Ghost Edges" or "Hover Lines". When a user inspects a node, reveal its non-tree dependencies.

### Visualization Engine
- **Constraint**: The user dislikes "inferior 2D force graphs" (wobbly, messy).
- **Solution**: **Custom D3.js Hierarchical Layout**.
    - **Style**: "Circuit Board" or "Metro Map" aesthetic. Rigid, orthogonal lines. No physics jitter.
    - **Library**: `d3-hierarchy` (Cluster or Tree) + Custom Edge Routing.

## 4. Interaction: "Semi-Immersive"
- **Canvas**: Full screen.
- **Anchor**: Top-Left "Back Button" (Styled consistently with Aether Self-Space).
- **Vibe**: "Aether Special Sector". Distinct but connected.
