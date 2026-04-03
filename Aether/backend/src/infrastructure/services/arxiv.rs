use crate::domain::prkb::models::InboxItem;
use chrono::Utc;
use reqwest::Client;

#[derive(Clone)]
pub struct ArxivService {
    client: Client,
}

impl ArxivService {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Aether/1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    pub async fn fetch_recent_by_category(
        &self,
        category: &str,
        limit: usize,
    ) -> Result<Vec<InboxItem>, anyhow::Error> {
        let url = format!(
            "http://export.arxiv.org/api/query?search_query=cat:{}&sortBy=submittedDate&sortOrder=descending&max_results={}", 
            category, limit
        );

        let content = self.client.get(&url).send().await?.bytes().await?;
        let feed = feed_rs::parser::parse(&content[..])?;

        let mut items = Vec::new();

        for entry in feed.entries {
            let authors: Vec<String> = entry.authors.into_iter().map(|a| a.name).collect();

            let pdf_url = entry
                .links
                .iter()
                .find(|l| {
                    let mt = l.media_type.as_deref().unwrap_or("");
                    let title = l.title.as_deref().unwrap_or("").to_lowercase();
                    mt == "application/pdf" || title == "pdf" || l.href.contains("/pdf/")
                })
                .map(|l| l.href.clone());

            let abstract_text = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.map(|c| c.body.unwrap_or_default()))
                .unwrap_or_default()
                .replace("\n", " ")
                .trim()
                .to_string();

            let publish_date = entry
                .published
                .or(entry.updated)
                .unwrap_or_else(|| Utc::now());

            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "Untitled".to_string())
                .replace("\n", " ")
                .trim()
                .to_string();

            let external_id = entry.id.clone();

            let item_url = if external_id.starts_with("http") {
                external_id.clone()
            } else {
                entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_else(|| external_id.clone())
            };

            items.push(InboxItem {
                id: uuid::Uuid::new_v4(),   // Transient ID
                feed_id: uuid::Uuid::nil(), // Caller sets this
                external_id,
                title,
                authors,
                abstract_text,
                url: item_url,
                pdf_url,
                publish_date,
                is_read: false,
                is_saved: false,
                fetched_at: Utc::now(),
                publication: None, // Assuming journal_ref is dropped in feed_rs Atom parse
                state: "new".to_string(),
                priority: None,
                note: None,
            });
        }

        Ok(items)
    }
}
