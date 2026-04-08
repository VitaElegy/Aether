use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- PRKB-01: Feed with health/diagnostics ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub feed_type: String, // "arxiv", "rss"
    pub enabled: bool,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    // Diagnostics
    pub health_status: String,      // "healthy", "degraded", "error", "unknown"
    pub total_fetched: i64,
    pub parse_errors: i64,
    pub last_error: Option<String>,
}

// --- Authors & Venues ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: Uuid,
    pub name: String,
    pub canonical_name: Option<String>,
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venue {
    pub id: Uuid,
    pub name: String,
    pub tier: Option<String>,
}

// --- PRKB-07: Enhanced Signals ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signals {
    pub citation_count: i32,
    pub github_stars: i32,
    pub sota_rank: Option<String>,
    pub last_updated: DateTime<Utc>,
    // PRKB-07 additions
    pub feed_freshness: Option<String>,      // "fresh", "recent", "stale"
    pub venue_tier: Option<String>,          // "A*", "A", "B", "C"
    pub author_recurrence: Option<i32>,      // how many times this author appeared
    pub custom_importance: Option<i32>,      // 1-5 user-set importance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibTexInfo {
    pub publisher: Option<String>,
    pub editor: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub isbn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperMetadata {
    pub track: Option<String>,
    pub series: Option<String>,
    pub bibtex: Option<BibTexInfo>,
    pub subjects: Vec<String>,
    pub keywords: Vec<String>,
}

// --- PRKB-02: Inbox with full triage states ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: Uuid,
    pub feed_id: Uuid,
    pub external_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub publication: Option<String>,
    pub publish_date: DateTime<Utc>,
    pub is_read: bool,
    pub is_saved: bool,
    pub fetched_at: DateTime<Utc>,
    // PRKB-02 additions
    pub state: String,          // "new", "read", "saved", "skipped", "trashed"
    pub priority: Option<i32>,  // 1-5
    pub note: Option<String>,
}

// --- PRKB-06: PDF Lifecycle ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<Author>,
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub pdf_local_path: Option<String>,
    pub venue: Option<Venue>,
    pub publish_date: DateTime<Utc>,
    pub arxiv_id: Option<String>,
    pub source: String,
    pub saved_at: DateTime<Utc>,
    pub is_read: bool,
    pub state: String,
    pub tags: Vec<String>,
    pub signals: Option<Signals>,
    pub metadata: Option<PaperMetadata>,
    // PRKB-06: PDF lifecycle
    pub pdf_status: String, // "not_attached", "queued", "downloaded", "indexed", "failed"
    // PRKB-03: notes
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PaperFilter {
    pub venue_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub is_read: Option<bool>,
    // PRKB-04: Search / Facet
    pub query: Option<String>,
    pub state: Option<String>,
    pub tag: Option<String>,
    pub year: Option<i32>,
    pub has_pdf: Option<bool>,
    pub pdf_status: Option<String>,
    pub collection_id: Option<Uuid>,
}

// --- PRKB-05: Collections ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub collection_type: String, // "watchlist", "reading_queue", "archive", "topic_collection"
    pub description: Option<String>,
    pub paper_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CollectionItem {
    pub collection_id: Uuid,
    pub paper_id: Uuid,
    pub added_at: DateTime<Utc>,
    pub sort_order: i32,
}

// --- PRKB-08: Portability ---
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ImportResult {
    pub imported: usize,
    pub duplicates: usize,
    pub errors: usize,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ExportOptions {
    pub format: String, // "bibtex", "json", "markdown", "collection_bundle"
    pub collection_id: Option<Uuid>,
    pub paper_ids: Option<Vec<Uuid>>,
}
