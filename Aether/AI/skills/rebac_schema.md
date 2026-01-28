# Skill: ReBAC & Schema Design

## 1. Architecture Patterns
### 1.1 Class Table Inheritance (CTI)
- **Structure**: `nodes` table (Shared) + `article_details`/`vocab_details` (Specific).
- **Rule**: All queries involving specific data **MUST JOIN** the `nodes` table.

### 1.2 ReBAC (Zanzibar)
- **Tuple Store**: `relationships` table.
- **Auto-Tuple**: Creating content automatically creates `(node, owner, author)`.
- **Public Access**: Public groups map to `group:public`.

## 2. Specialized Architectures
### 2.1 Renderer ID Pattern
- **Concept**: Single backend content serving multiple frontends.
- **Mechanism**: `knowledge_bases.renderer_id`.
- **Frontend Rule**: Check `renderer_id` before mounting layout.

### 2.2 Semantic Markdown
- **Syntax**: `::: [type] {json} :::`
- **Benefit**: No DB migration for new content types.
