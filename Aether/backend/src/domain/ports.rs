use async_trait::async_trait;
use super::models::{ContentAggregate, ContentId, User, UserId, AuthClaims, Comment, ContentVersionSnapshot, CommentableId, CommentableType, Memo, MemoId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,
    #[error("Storage conflict: {0}")]
    Conflict(String),
    #[error("Connection failure: {0}")]
    ConnectionError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token invalid or expired")]
    InvalidToken,
    #[error("Repository error: {0}")]
    RepoError(#[from] RepositoryError),
    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),
}

/// The Input Port: Defines what the application CORE expects from the storage layer.
#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn save(&self, content: ContentAggregate, editor_id: UserId, should_create_snapshot: bool) -> Result<ContentId, RepositoryError>;
    async fn find_by_id(&self, id: &ContentId) -> Result<Option<ContentAggregate>, RepositoryError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<ContentAggregate>, RepositoryError>;
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>, limit: u64, offset: u64) -> Result<Vec<ContentAggregate>, RepositoryError>;
    async fn search(&self, query: &str) -> Result<Vec<ContentAggregate>, RepositoryError>;
    async fn delete(&self, id: &ContentId) -> Result<(), RepositoryError>;
    async fn get_version(&self, id: &ContentId, version: i32) -> Result<Option<(String, String)>, RepositoryError>; // Returns (Title, JSON body)
    async fn get_history(&self, id: &ContentId) -> Result<Vec<ContentVersionSnapshot>, RepositoryError>;
}

/// Auth Repository Port
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;
    async fn save(&self, user: User) -> Result<UserId, RepositoryError>;
}

/// Comment Repository Port
#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn add_comment(&self, comment: Comment) -> Result<super::models::CommentId, RepositoryError>;
    async fn get_comments(&self, target: &CommentableId) -> Result<Vec<Comment>, RepositoryError>;
    async fn get_comments_batch(&self, targets: &[CommentableId]) -> Result<Vec<Comment>, RepositoryError>;
}

/// Memo Repository Port
#[async_trait]
pub trait MemoRepository: Send + Sync {
    async fn save(&self, memo: Memo) -> Result<MemoId, RepositoryError>;
    async fn find_by_id(&self, id: &MemoId) -> Result<Option<Memo>, RepositoryError>;
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<Memo>, RepositoryError>;
    async fn delete(&self, id: &MemoId) -> Result<(), RepositoryError>;
}

/// Knowledge Base Repository Port
#[async_trait]
pub trait KnowledgeBaseRepository: Send + Sync {
    async fn save(&self, kb: super::models::KnowledgeBase) -> Result<super::models::KnowledgeBaseId, RepositoryError>;
    async fn find_by_id(&self, id: &super::models::KnowledgeBaseId) -> Result<Option<super::models::KnowledgeBase>, RepositoryError>;
    async fn find_by_title(&self, author_id: &UserId, title: &str) -> Result<Option<super::models::KnowledgeBase>, RepositoryError>;
    async fn list(&self, author_id: UserId) -> Result<Vec<super::models::KnowledgeBase>, RepositoryError>;
    async fn delete(&self, id: &super::models::KnowledgeBaseId) -> Result<(), RepositoryError>;
}

/// Tag Repository Port
#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn get_all_tags(&self) -> Result<Vec<String>, RepositoryError>;
}

// --- Export Domain ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExportFormat {
    Markdown,
    Json,
    Html,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportData {
    pub entity_type: CommentableType,
    pub entity_id: uuid::Uuid,
    pub entity_data: serde_json::Value,
    pub comments: Vec<Comment>,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportMetadata {
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub exported_by: Option<UserId>, // Public export might not have user
    pub format: ExportFormat,
}

/// Export Service Port
#[async_trait]
pub trait ExportService: Send + Sync {
    async fn export_content_with_comments(
        &self,
        content_id: &ContentId,
        format: ExportFormat,
        requester: Option<UserId>
    ) -> Result<Vec<u8>, RepositoryError>;

    async fn export_memo_with_comments(
        &self,
        memo_id: &MemoId,
        format: ExportFormat,
        requester: Option<UserId>
    ) -> Result<Vec<u8>, RepositoryError>;
}

/// Auth Service Port (for encryption/token generation logic)
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthClaims, AuthError>;
    fn verify_token(&self, token: &str) -> Result<AuthClaims, AuthError>;
    fn generate_token(&self, claims: &AuthClaims) -> Result<String, AuthError>;
}

/// Service Port: Defines complex domain logic that doesn't fit in the entity.
#[async_trait]
#[allow(dead_code)]
pub trait ContentService: Send + Sync {
    async fn publish(&self, id: ContentId) -> Result<(), RepositoryError>;
    // Example of extension: specific rendering logic could be injected here
    async fn render_content(&self, id: ContentId) -> Result<String, RepositoryError>;
}
