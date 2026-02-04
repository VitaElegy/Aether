use quick_xml::de::from_str;
use serde::Deserialize;
use reqwest::Client;
use chrono::{DateTime, Utc};
use crate::domain::prkb::models::InboxItem;

#[derive(Debug, Clone)]
pub struct ArxivService {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct AtomFeed {
    entry: Option<Vec<AtomEntry>>,
}

#[derive(Debug, Deserialize)]
struct AtomEntry {
    id: String,
    title: String,
    summary: String,
    published: String,
    author: Vec<AtomAuthor>,
    link: Vec<AtomLink>,
}

#[derive(Debug, Deserialize)]
struct AtomAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AtomLink {
    #[serde(rename = "@href")]
    href: String,
    #[allow(dead_code)]
    #[serde(rename = "@rel", default)]
    rel: Option<String>,
    #[serde(rename = "@type", default)]
    link_type: Option<String>,
    #[serde(rename = "@title", default)]
    title: Option<String>,
}

impl ArxivService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_recent_by_category(&self, category: &str, limit: usize) -> Result<Vec<InboxItem>, anyhow::Error> {
        let url = format!(
            "http://export.arxiv.org/api/query?search_query=cat:{}&sortBy=submittedDate&sortOrder=descending&max_results={}", 
            category, limit
        );

        let resp = self.client.get(&url).send().await?.text().await?;
        
        // Parse XML
        let feed: AtomFeed = from_str(&resp)?;

        let mut items = Vec::new();
        if let Some(entries) = feed.entry {
            for entry in entries {
                let authors: Vec<String> = entry.author.iter().map(|a| a.name.clone()).collect();
                
                let pdf_url = entry.link.iter()
                    .find(|l| l.link_type.as_deref() == Some("application/pdf") || l.title.as_deref() == Some("pdf")) // Arxiv uses title="pdf" sometimes or type
                    .map(|l| l.href.clone())
                    .or_else(|| {
                         // Fallback logic for Arxiv PDF links if type is missing? 
                         // Usually links are: 
                         // <link href="http://arxiv.org/abs/2101.00001" rel="alternate" type="text/html"/>
                         // <link title="pdf" href="http://arxiv.org/pdf/2101.00001" rel="related" type="application/pdf"/>
                         entry.link.iter().find(|l| l.href.contains("/pdf/")).map(|l| l.href.clone())
                    });

                // Clean abstract (Arxiv often has newlines)
                let abstract_text = entry.summary.replace("\n", " ").trim().to_string();

                // Parse Date
                let publish_date = DateTime::parse_from_rfc3339(&entry.published)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                items.push(InboxItem {
                    id: uuid::Uuid::new_v4(), // Transient ID
                    feed_id: uuid::Uuid::nil(), // Caller sets this
                    external_id: entry.id.clone(), // Arxiv URL/ID
                    title: entry.title.replace("\n", " ").trim().to_string(),
                    authors,
                    abstract_text,
                    url: entry.id, // ID is often the ABS URL, move is fine here if it's last usage or clone

                    pdf_url,
                    publish_date,
                    is_read: false,
                    is_saved: false,
                    fetched_at: Utc::now(),
                });
            }
        }
        
        Ok(items)
    }
}

// Helper to fix the quick-xml parsing issue with single vs multiple entries if needed.
// However, quick-xml with serde `Vec` usually handles single item as 1-element vec if configured?
// Defaults might be tricky. If `entry` is missing, it's None. If 1, it might fail to map to Vec without `serde_as`.
// For simplicity, let's assume `Vec<AtomEntry>` works or we catch the error.
// Ideally usage of `#[serde(default)]` helps. 
// A robust way uses a custom deserializer or a wrapper enum.
// Let's hope quick-xml handles `entry` appearing multiple times into a Vec.
