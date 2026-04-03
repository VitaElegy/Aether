use crate::domain::prkb::models::InboxItem;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct RssService {
    client: Client,
}

impl RssService {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Aether/1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    pub async fn fetch_feed(&self, url: &str) -> Result<Vec<InboxItem>, anyhow::Error> {
        let content = match self.client.get(url).send().await {
            Ok(r) => r.bytes().await?,
            Err(e) => return Err(anyhow::anyhow!("Request failed: {}", e)),
        };

        let feed = match feed_rs::parser::parse(&content[..]) {
            Ok(f) => f,
            Err(e) => {
                let text = String::from_utf8_lossy(&content[..std::cmp::min(100, content.len())]);
                return Err(anyhow::anyhow!(
                    "Parsing failed: {}. Body snippet: {:?}",
                    e,
                    text
                ));
            }
        };

        let mut items = Vec::new();

        for entry in feed.entries {
            // Extract Title
            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "Untitled".to_string());

            // Extract URL
            let entry_url = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default();

            // Extract Authors
            let authors: Vec<String> = entry.authors.into_iter().map(|p| p.name).collect();

            // Extract Content/Summary
            let abstract_text = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.map(|c| c.body.unwrap_or_default()))
                .unwrap_or_default();

            // Extract Date
            let publish_date = entry
                .published
                .or(entry.updated)
                .unwrap_or_else(|| Utc::now());

            items.push(InboxItem {
                publication: None,
                id: Uuid::new_v4(),
                feed_id: Uuid::nil(),  // Caller sets this
                external_id: entry.id, // RSS GUID/ID
                title,
                authors,
                abstract_text, // Usually HTML in RSS, might need stripping
                url: entry_url,
                pdf_url: None, // RSS rarely links PDFs directly, mostly HTML
                publish_date,
                is_read: false,
                is_saved: false,
                fetched_at: Utc::now(),
                state: "new".to_string(),
                priority: None,
                note: None,
            });
        }

        Ok(items)
    }
}
