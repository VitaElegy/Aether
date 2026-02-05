# Paper KB (PRKB) Upgrade Consultation
Date: 2026-02-05

## 1. Crawling Strategy
**Question**: Do you need "Historical Backfill" (past years) and "Citation Deep-Crawling" (recursive)?
**Answer**: Yes, both are required.
- **Requirement**: System must support searching specific keywords over time ranges (e.g., "Mali GPU" 2020-2024).
- **Requirement**: System must support recursive fetching of references for "Deep Read" papers.

## 2. Entity Resolution
**Question**: Do you need First-Class Author Entities (Profiles, disambiguation, subscriptions)?
**Answer**: Yes ("可以").
- **Requirement**: Treat Authors as distinct entities, not just strings.
- **Requirement**: Support Author Profiles, Disambiguation, and Subscriptions.

## 3. Full-Text Indexing
**Question**: Do you need automatic PDF downloading and Full-Text Indexing?
**Answer**: Hybrid / On-Demand ("自由决定是否下载...提供获取能力...支持手动上传").
- **Decision**: Default is Link Index (Click -> External URL).
- **Decision**: Optional "Download to Local" button for specific papers.
- **Decision**: Support Manual Upload fallback.

## 4. Workflow State Machine
**Question**: Do you need a Kanban-like lifecycle (Inbox -> Screening -> Reading...)?
**Answer**: Yes ("可以").
- **Requirement**: Implement a state machine for papers (e.g., Inbox, Screening, Reading, Archived).

## 5. Smart Filtering
**Question**: Do you need Dynamic Faceted Filtering (by Author, Venue, Year, Tag)?
**Answer**: High Priority but Concise UI ("优先级高...UI界面一定要简洁").
- **Requirement**: Backend must support aggregation queries.
- **Requirement**: Frontend must provide powerful filters without cluttering the interface (e.g., collapsible sidebar or smart search bar).

## 6. AI Pre-Flight Agent
**Question**: Do you need AI to auto-tag, score, and summarize incoming papers?
**Answer**: No ("暂时不用").
- **Constraint**: User wants to focus on **Frontend Design**, **Retrieval**, and **Crawling** capabilities instead.

## 7. Frontend Visualization
**Question**: Do you need Graphs or Timelines?
**Answer**: No ("先暂时就是极致优化的列表即可").
- **Decision**: Stick to **Rich List** view.
- **Requirement**: Focus on list typography, density, and readability. Avoid complex D3/Canvas graphs for now.

## 8. Advanced Retrieval
**Question**: Do you need Structured Query Syntax (e.g., `year:>2023`)?
**Answer**: Hybrid ("更喜欢点点选选...但也要支持硬核搜索").
- **Decision**: Primary UI is Faceted/Visual Search.
- **Decision**: Search bar must support advanced syntax (Parsel/Lucency style) for power users.

## 9. External Signals
**Question**: Do you need Citation Counts, GitHub Stars, SOTA rankings?
**Answer**: Yes ("这非常重要").
- **Requirement**: Integrate Semantic Scholar / PapersWithCode APIs.
- **Requirement**: Display these signals prominently in the card capabilities.

## 10. Export & Output
**Question**: Do you need Weekly Reports or BibTeX Export?
**Answer**: Yes ("可以").
- **Requirement**: One-click "Weekly Summary" generation.
- **Requirement**: BibTeX export for selected items.
