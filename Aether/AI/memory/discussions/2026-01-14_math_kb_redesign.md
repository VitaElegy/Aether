# Math Knowledge Base Redesign: Discussion & Decisions

**Date**: 2026-01-14
**Topic**: Establishing a Rigorous Mathematical Knowledge Base Architecture
**Participants**: User (Product Owner), Agent (System Architect)

## 1. Objective (目的)
Transform the initial "Math Knowledge Base" demo from a simple visual toy into a **semantically rigorous tool** capable of representing serious mathematical structures. The user specifically requested a move away from "scattered graphs" to a "rigorous Computer Science Tree/DAG" structure that reflects logical dependencies (Axiom -> Theorem -> Proof).

## 2. The Inquiry Process (探讨过程)
We conducted a "Deep Design Inquiry" covering 7 architectural layers to define the ontology.

### Layer 1: The Atomic Unit (基本单元)
*   **Discussion**: Should the smallest node be an "Article" (Page) or a "Statement" (Theorem/Def)?
*   **Decision**: **Hybrid Model**.
    *   **Database**: Managed as discrete semantic nodes (Axioms, Theorems) via Markdown parsing.
    *   **Presentation**: These nodes can exist independently but are authored within coherent "Articles" to maintain narrative flow.

### Layer 2 & 3: Topology & Rigor (依赖与结构)
*   **Discussion**: Mathematics is a DAG (Directed Acyclic Graph), but the user wants a "Tree". How to handle multi-parent dependencies (e.g., a theorem using two axioms)?
*   **Decision**: **"A + C" Strategy**.
    *   **Mirroring (A)**: Strict hierarchy where possible.
    *   **Dynamic Focus (C)**: When a node is selected, dynamically highlight its specific upstream/downstream dependency chain, ignoring the rest of the tree.
*   **Validation**: **Weak Validation** (Linter style). The system allows creativity but warns if links break.

### Layer 4: Polymorphism (一题多解)
*   **Discussion**: How to handle multiple proofs for one theorem?
*   **Decision**: **Tabs + Branching (B + C)**.
    *   UI: Use Tabs for compact reading.
    *   Graph: Represent proofs as distinct child nodes hanging off the theorem.

### Layer 5: Input Strategy (输入方式)
*   **Decision**: **Semantic Markdown**.
    *   Use `::: theorem [Title]` syntax to define nodes without leaving the editor.

### Layer 6 & 7: Interaction & Vibe (交互与风格)
*   **Decision**: **Mixed Mode** & **Modern Tool**.
    *   Clicking a node expands a preview; double-clicking navigates.
    *   Aesthetic: "Elegant Rigor" —— Minimalist, Code-font headers, distinct color coding for logical types.

## 3. Implementation Results (最终结果)

### 3.1 New Ontology Parsing (`MathSemanticsParser`)
We implemented a strongly-typed parser that supports the following "Periodic Table" of Math elements:
*   **Axiom** (Red, Octahedron): Universal truths.
*   **Definition** (Blue, Sphere): Terminology.
*   **Theorem** (Purple, Cube): Core Derived Truths.
*   **Proof** (Green, Cone): Logical justifications.

### 3.2 3D Semantic Graph
Rebuilt `KnowledgeGraph.vue` using a **Stratified Force-Directed Layout**:
*   **Y-Axis Force**: Strictly aligns nodes by logical depth (Axioms at top, Proofs at bottom).
*   **Visual Semantics**: Different geometries and colors for immediate type recognition.

### 3.3 Demo Content
Auto-generated a "Group Theory" knowledge base to verify the system's ability to handle:
*   Circular dependencies (Group Axioms <-> Subgroup definitions).
*   Complex notation ($\LaTeX$ support).
*   Hierarchical Proofs.

---
*Verified by User on 2026-01-14.*
