use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub publish_date: DateTime<Utc>,
    pub arxiv_id: Option<String>,
    pub source: String,
    pub saved_at: DateTime<Utc>,
    pub is_read: bool,
    pub tags: Vec<String>,
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
    pub publish_date: DateTime<Utc>,
    pub is_read: bool,
    pub is_saved: bool, // If true, it exists in 'papers' table too (or we just move it)
    pub fetched_at: DateTime<Utc>,
}
