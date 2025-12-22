use async_trait::async_trait;
use super::models::{ContentAggregate, ContentId, User, UserId, AuthClaims, Comment};
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
#[allow(dead_code)]
pub trait ContentRepository: Send + Sync {
    async fn save(&self, content: ContentAggregate, editor_id: UserId) -> Result<ContentId, RepositoryError>;
    async fn find_by_id(&self, id: &ContentId) -> Result<Option<ContentAggregate>, RepositoryError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<ContentAggregate>, RepositoryError>;
    async fn list(&self, limit: u64, offset: u64) -> Result<Vec<ContentAggregate>, RepositoryError>;
    async fn delete(&self, id: &ContentId) -> Result<(), RepositoryError>;
    async fn get_version(&self, id: &ContentId, version: i32) -> Result<Option<String>, RepositoryError>; // Returns JSON body
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
    async fn get_comments(&self, content_id: &ContentId) -> Result<Vec<Comment>, RepositoryError>;
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
