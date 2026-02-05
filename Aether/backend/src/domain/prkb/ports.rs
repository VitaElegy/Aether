use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::prkb::models::{Paper, Feed, InboxItem};
use crate::domain::ports::RepositoryError;

#[async_trait]
#[allow(dead_code)]
pub trait PrkbRepository: Send + Sync {
    // Feed Management
    async fn create_feed(&self, feed: Feed) -> Result<Uuid, RepositoryError>;
    async fn list_feeds(&self) -> Result<Vec<Feed>, RepositoryError>;
    async fn get_feed(&self, id: Uuid) -> Result<Option<Feed>, RepositoryError>;
    async fn delete_feed(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn update_feed_last_fetched(&self, id: Uuid, time: chrono::DateTime<chrono::Utc>) -> Result<(), RepositoryError>;

    // Inbox Management
    async fn save_inbox_items(&self, items: Vec<InboxItem>) -> Result<(), RepositoryError>;
    async fn get_inbox(&self, limit: u64, offset: u64, unread_only: bool, publication: Option<String>) -> Result<Vec<InboxItem>, RepositoryError>;
    async fn markup_inbox_item_read(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn delete_inbox_item(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn get_inbox_item_by_external_id(&self, external_id: &str) -> Result<Option<InboxItem>, RepositoryError>;
    async fn count_inbox(&self, unread_only: bool, publication: Option<String>) -> Result<u64, RepositoryError>;
    async fn update_inbox_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError>;
    async fn get_unique_publications(&self) -> Result<Vec<String>, RepositoryError>;

    // Library (Papers)
    async fn save_paper(&self, paper: Paper) -> Result<Uuid, RepositoryError>;
    async fn list_papers(&self, filter: crate::domain::prkb::models::PaperFilter, limit: u64, offset: u64) -> Result<Vec<Paper>, RepositoryError>;
    async fn get_paper(&self, id: Uuid) -> Result<Option<Paper>, RepositoryError>;
    async fn update_paper_read_status(&self, id: Uuid, is_read: bool) -> Result<(), RepositoryError>;
    async fn update_paper_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError>;
    async fn delete_paper(&self, id: Uuid) -> Result<(), RepositoryError>;

    // Venues
    async fn list_venues(&self) -> Result<Vec<crate::domain::prkb::models::Venue>, RepositoryError>;
}
