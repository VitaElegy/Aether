use crate::domain::ports::RepositoryError;
use crate::domain::prkb::models::{Collection, Feed, InboxItem, Paper, Venue};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait PrkbRepository: Send + Sync {
    // --- PRKB-01: Feed Management ---
    async fn create_feed(&self, feed: Feed) -> Result<Uuid, RepositoryError>;
    async fn list_feeds(&self) -> Result<Vec<Feed>, RepositoryError>;
    async fn get_feed(&self, id: Uuid) -> Result<Option<Feed>, RepositoryError>;
    async fn delete_feed(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn update_feed_last_fetched(
        &self,
        id: Uuid,
        time: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError>;
    async fn update_feed_enabled(&self, id: Uuid, enabled: bool) -> Result<(), RepositoryError>;
    async fn update_feed_health(
        &self,
        id: Uuid,
        status: String,
        error: Option<String>,
    ) -> Result<(), RepositoryError>;
    async fn increment_feed_stats(
        &self,
        id: Uuid,
        fetched: i64,
        errors: i64,
    ) -> Result<(), RepositoryError>;

    // --- PRKB-02: Inbox Management ---
    async fn save_inbox_items(&self, items: Vec<InboxItem>) -> Result<(), RepositoryError>;
    async fn get_inbox(
        &self,
        limit: u64,
        offset: u64,
        unread_only: bool,
        publication: Option<String>,
    ) -> Result<Vec<InboxItem>, RepositoryError>;
    async fn markup_inbox_item_read(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn delete_inbox_item(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn get_inbox_item_by_external_id(
        &self,
        external_id: &str,
    ) -> Result<Option<InboxItem>, RepositoryError>;
    async fn count_inbox(
        &self,
        unread_only: bool,
        publication: Option<String>,
    ) -> Result<u64, RepositoryError>;
    async fn update_inbox_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError>;
    async fn update_inbox_priority(
        &self,
        id: Uuid,
        priority: Option<i32>,
    ) -> Result<(), RepositoryError>;
    async fn update_inbox_note(
        &self,
        id: Uuid,
        note: Option<String>,
    ) -> Result<(), RepositoryError>;
    async fn get_unique_publications(&self) -> Result<Vec<String>, RepositoryError>;

    // --- PRKB-03/06: Library (Papers) ---
    async fn save_paper(&self, paper: Paper) -> Result<Uuid, RepositoryError>;
    async fn list_papers(
        &self,
        filter: crate::domain::prkb::models::PaperFilter,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Paper>, RepositoryError>;
    async fn get_paper(&self, id: Uuid) -> Result<Option<Paper>, RepositoryError>;
    async fn update_paper_read_status(
        &self,
        id: Uuid,
        is_read: bool,
    ) -> Result<(), RepositoryError>;
    async fn update_paper_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError>;
    async fn update_paper_tags(&self, id: Uuid, tags: Vec<String>) -> Result<(), RepositoryError>;
    async fn update_paper_notes(
        &self,
        id: Uuid,
        notes: Option<String>,
    ) -> Result<(), RepositoryError>;
    async fn update_paper_pdf_status(
        &self,
        id: Uuid,
        status: String,
        local_path: Option<String>,
    ) -> Result<(), RepositoryError>;
    async fn delete_paper(&self, id: Uuid) -> Result<(), RepositoryError>;

    // Venues
    async fn list_venues(&self) -> Result<Vec<Venue>, RepositoryError>;

    // --- PRKB-04: Search ---
    async fn search_papers(
        &self,
        query: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Paper>, RepositoryError>;
    async fn count_papers(
        &self,
        filter: crate::domain::prkb::models::PaperFilter,
    ) -> Result<u64, RepositoryError>;

    // --- PRKB-05: Collections ---
    async fn create_collection(&self, collection: Collection) -> Result<Uuid, RepositoryError>;
    async fn list_collections(&self) -> Result<Vec<Collection>, RepositoryError>;
    async fn get_collection(&self, id: Uuid) -> Result<Option<Collection>, RepositoryError>;
    async fn update_collection(
        &self,
        id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<(), RepositoryError>;
    async fn delete_collection(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn add_paper_to_collection(
        &self,
        collection_id: Uuid,
        paper_id: Uuid,
    ) -> Result<(), RepositoryError>;
    async fn remove_paper_from_collection(
        &self,
        collection_id: Uuid,
        paper_id: Uuid,
    ) -> Result<(), RepositoryError>;
    async fn list_collection_papers(
        &self,
        collection_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Paper>, RepositoryError>;

    // --- PRKB-07: Signals ---
    async fn update_paper_signals(
        &self,
        paper_id: Uuid,
        signals: crate::domain::prkb::models::Signals,
    ) -> Result<(), RepositoryError>;

    // --- PRKB-08: Portability ---
    async fn find_paper_by_doi(&self, doi: &str) -> Result<Option<Paper>, RepositoryError>;
    async fn find_paper_by_external_id(
        &self,
        external_id: &str,
    ) -> Result<Option<Paper>, RepositoryError>;
    async fn find_paper_by_title(&self, title: &str) -> Result<Option<Paper>, RepositoryError>;
}
