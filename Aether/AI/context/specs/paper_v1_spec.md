# Specification: Computer Science Paper Knowledge Base (`paper_v1`)

> **Status**: Draft
> **Author**: Senior Systems Architect
> **Version**: 1.0

## 1. Overview

The **Paper Knowledge Base** is a specialized domain for managing academic literature, focusing on Computer Science. It features a "Triage-First" workflow, separates metadata from full-text, and emphasizes "Clean Academic" aesthetics.

### 1.1 Core Philosophy
- **Triage > Storage**: The world produces too many papers. The system acts as a filter (Inbox), not just a bucket.
- **Metadata First**: PDFs are heavy; metadata is light. Always index metadata; download PDF only on demand.
- **Code is First-Class**: A paper without code is often just math. GitHub repositories are treated as first-class citizens alongside the PDF.
- **Visual Rationality**: "Lab Mode" aesthetics – clean, grid-based, data-dense, void of clutter.

---

## 2. Requirements Summary

| Feature          | Decision                                                        |
| :--------------- | :-------------------------------------------------------------- |
| **Ingestion**    | **Inbox/Triage Mode**. Auto-fetch recent; Manual confirm.       |
| **Storage**      | **Metadata First**. PDF Download-on-Demand.                     |
| **Reading**      | **Classic PDF Mode** (Zotero-style) with Highlights.            |
| **Annotations**  | **Bi-Directional**. Highlights = System Blocks (Referenceable). |
| **Organization** | **Hybrid**. AI Auto-Tags + Manual Folders.                      |
| **Code**         | **Active Intelligence**. Auto-scrape GitHub (Stars/README).     |
| **Graph**        | **Local Graph** (1-hop Neighbors).                              |
| **AI**           | **Passive Q&A**. Answer only when asked.                        |
| **Theme**        | **Clean Academic / Lab** (Swiss Style, Grids).                  |

---

## 3. Data Model (Schema)

### 3.1 The `PaperBlock` (Main Entity)
This block represents a single paper. It is the "Metadata Container".

```rust
// backend/src/domain/blocks/schemas/paper.rs

pub fn get_paper_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            // Core Metadata
            "title": { "type": "string" },
            "authors": { "type": "array", "items": { "type": "string" } },
            "abstract": { "type": "string" },
            "venue": { "type": "string" }, // e.g., "CVPR 2024"
            "year": { "type": "integer" },
            "doi": { "type": "string" },
            
            // External Links
            "pdf_url": { "type": "string" },
            "arxiv_id": { "type": "string" },
            "project_page": { "type": "string" },
            
            // Code Intelligence
            "github_repo": { "type": "string" }, // URL
            "github_stars": { "type": "integer" },
            "has_code": { "type": "boolean" },
            
            // System State
            "ingest_status": { 
                "type": "string", 
                "enum": ["inbox", "archived", "trash"] 
            },
            "read_status": {
                "type": "string",
                "enum": ["unread", "reading", "read"]
            },
            
            // Local Storage
            "local_pdf_path": { "type": "string" } // Nullable. If Set, PDF is downloaded.
        },
        "required": ["title", "ingest_status"]
    })
}
```

### 3.2 The `AnnotationBlock` (Highlights)
Highlights are stored as separate Blocks linked to the Paper (Parent).

```rust
// Represented as a child Node in the general Node table, 
// using a distinct block schema if needed, or generic 'text_selection'.
```

---

## 4. Frontend Architecture (Vue 3)

### 4.1 Layout: `PaperLayout.vue`
- **Theme**: Light Mode / Cool Grey.
- **Font**: Inter (UI), Merriweather/Times (Content).
- **Structure**:
    - **Sidebar**: Folders + Auto-Tags (e.g., `#CVPR2025`, `#Transformer`).
    - **Main Area**: 
        - **Inbox View**: Kanvan/List for Triage.
        - **Library View**: Data Grid (Ag-Grid style) with columns for "Stars", "Year", "Venue".

### 4.2 Component: `PaperCard.vue` (Inbox Item)
- Displays Title, Authors, Venue, GitHub Stats.
- Actions: "Accept" (Move to Library), "Trash", "Quick PDF Preview".

### 4.3 Component: `PDFReader.vue`
- Wrapper around `pdf.js` or `browser-native`.
- **Overlay**: Layer to draw Highlights over the PDF canvas.
- **Sidebar**: "Local Graph" (D3 Force Graph of 1-hop citations).
- **AI Chat**: Floating/Docked panel for Passive Q&A.

---

## 5. Backend Services (Rust)

### 5.1 `PaperIngestService`
- **Source**: arXiv API, Semantic Scholar API (if key available).
- **Trigger**: 
    - CRON (Daily) -> Fetch new papers -> Put in `ingest_status: inbox`.
    - Manual Import (DOI/URL) -> Fetch Metadata -> Put in `inbox`.

### 5.2 `CodeIntelligenceService`
- Input: GitHub URL.
- Process: Fetch `api.github.com/repos/...`.
- Output: Update `github_stars`, `last_commit_date`.

### 5.3 `ReferenceGraphService`
- Input: Paper DOI/Title.
- Process: Query Semantic Scholar Graph API (S2).
- Output: List of Parents (References) and Children (Citations).

---

## 6. Implementation Plan
1.  **Scaffold**: Run `new_kb.py` to geneate `paper_v1`.
2.  **Schema**: Implement Rust Schemas.
3.  **Mocks**: Create fake arXiv data to test Inbox UI.
4.  **UI**: Build `PaperLayout` & `InboxView`.
5.  **Reader**: Integrate PDF.js.
6.  **Services**: Implement Real Fetching Logic (arXiv/GitHub).
