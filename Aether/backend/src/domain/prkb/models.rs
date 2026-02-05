use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signals {
    pub citation_count: i32,
    pub github_stars: i32,
    pub sota_rank: Option<String>,
    pub last_updated: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<Author>, // Upgraded from Vec<String>
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
}

#[derive(Debug, Clone, Default)]
pub struct PaperFilter {
    pub venue_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub is_read: Option<bool>,
    // Future: query: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: Uuid,
    pub name: String, // e.g., "cs.AI"
    pub url: String, // e.g., "http://export.arxiv.org/rss/cs.AI" or just "cs.AI" for arxiv provider
    pub feed_type: String, // "arxiv_category", "rss"
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: Uuid,
    pub feed_id: Uuid,
    pub external_id: String, // e.g., arxiv ID
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub publication: Option<String>,
    pub publish_date: DateTime<Utc>,
    pub is_read: bool,
    pub is_saved: bool, // If true, it exists in 'papers' table too (or we just move it)
    pub fetched_at: DateTime<Utc>,
}
