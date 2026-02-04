use crate::domain::prkb::models::InboxItem;
use chrono::{DateTime, Utc};
use reqwest::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct RssService {
    client: Client,
}

impl RssService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_feed(&self, url: &str) -> Result<Vec<InboxItem>, anyhow::Error> {
        let content = self.client.get(url).send().await?.bytes().await?;
        let feed = feed_rs::parser::parse(&content[..])?;

        let mut items = Vec::new();

        for entry in feed.entries {
            // Extract Title
            let title = entry.title.map(|t| t.content).unwrap_or_else(|| "Untitled".to_string());

            // Extract URL
            let url = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();

            // Extract Authors
            let authors: Vec<String> = entry.authors.iter().map(|p| p.name.clone()).collect();

            // Extract Content/Summary
            let abstract_text = entry.summary.map(|s| s.content)
                .or_else(|| entry.content.map(|c| c.body.unwrap_or_default()))
                .unwrap_or_default();

            // Extract Date
            let publish_date = entry.published
                .or(entry.updated)
                .unwrap_or_else(|| Utc::now());

            items.push(InboxItem {
                id: Uuid::new_v4(),
                feed_id: Uuid::nil(), // Caller sets this
                external_id: entry.id, // RSS GUID/ID
                title,
                authors,
                abstract_text, // Usually HTML in RSS, might need stripping
                url,
                pdf_url: None, // RSS rarely links PDFs directly, mostly HTML
                publish_date,
                is_read: false,
                is_saved: false,
                fetched_at: Utc::now(),
            });
        }

        Ok(items)
    }
}
