# Specification: Paper Knowledge Base Metadata Extension (v1)
# 规范：论文知识库元数据扩展 (v1)

> **Status**: Draft
> **Language**: Bilingual (English/Chinese)

## 1. Overview (概述)
为了丰富 Inbox 卡片的信息密度，我们需要从源头（CVPR, NeurIPS 等）抓取更多的元数据。本规范定义了扩展的字段结构。
To enrich the information density of Inbox cards, we need to capture more metadata from sources (CVPR, NeurIPS, etc). This spec defines the extended field structure.

## 2. Extended Schema (扩展结构)

We will extend the `signals` or `venue` JSONB column (or add a new `metadata` column) to store the following:

```rust
pub struct PaperMetadata {
    // 1. Publication Track (赛道/形式)
    // Value: "Oral", "Spotlight", "Poster", "Main Track"
    // Impact: High priority display on Card.
    pub track: Option<String>,

    // 2. Official Series/Proceedings (会议卷号)
    // Value: "CVPR 2024", "NeurIPS 2023 Vol.36"
    // Impact: Replaces generic venue string.
    pub series: Option<String>,

    // 3. BibTeX Fields (引用数据)
    // Value: Full structured data for citation export.
    pub bibtex: Option<BibTexInfo>,

    // 4. Subject Areas (学科领域)
    // Value: ["Deep Learning", "Optimization"]
    pub subjects: Vec<String>,
}

pub struct BibTexInfo {
    pub publisher: Option<String>, // e.g., "IEEE", "Curran Associates"
    pub editor: Option<String>,
    pub pages: Option<String>,     // e.g., "1024-1035"
    pub doi: Option<String>,
    pub isbn: Option<String>,
}
```

## 3. UI Presentation (界面展示)

### 3.1 Inbox Card (卡片视图)
- **Top Right**: Show `track` as a colored badge (e.g., Red for Oral). (右上角显示赛道标签，如 Oral 为红色)
- **Subtitle**: Show `series` instead of generic year. (副标题显示具体的卷号/系列)
- **Hover**: Show `subjects`. (悬停显示领域标签)

### 3.2 Detail View (详情页)
- **Citation Block**: One-click "Copy BibTeX" button using the `bibtex` fields. (一键复制 BibTeX)
- **Official Link**: Button linking to `official_url` (DOI).

## 4. Scraper Strategy (爬虫策略)

| Source                                 | Strategy                  | Fields                                              |
| :------------------------------------- | :------------------------ | :-------------------------------------------------- |
| **CVPR / ICCV / WACV** (TheCVF)        | `thecvf.com` HTML Parsing | Track (from text), Pages, DOI                       |
| **NeurIPS / ICML / ICLR** (OpenReview) | OpenReview API / HTML     | **Keywords**, **Primary Area**, Track (Oral/Poster) |
| **ACL / EMNLP** (ACL Anthology)        | `aclanthology.org` HTML   | **Volume** (Track), Pages, BibTeX                   |
| **ArXiv**                              | API                       | Primary Category (Subject)                          |

---

## 5. User Decision Required (需要用户决策)

请确认：
1. **Track (赛道)** 是否是您最关心的优先级字段？(Is Track the highest priority?)
2. 是否需要 **Editor/Publisher** 等详细引用信息？(Do you need deep citation info like Editor?)
