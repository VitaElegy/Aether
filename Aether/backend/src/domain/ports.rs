use async_trait::async_trait;
use serde::Serialize;
use uuid::Uuid;
use thiserror::Error; // Added back
use crate::domain::models::{
    Article, Vocabulary, Memo, User, UserId, AuthClaims, Comment, CommentId,
    ContentVersionSnapshot, Node, KnowledgeBase, KnowledgeBaseId, ContentItem, Visibility, ContentDiff
};

#[derive(Debug, Clone, Serialize, Error)] // Added Error
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Duplicate title: {0}")]
    DuplicateTitle(String),
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthClaims, AuthError>;
    fn verify_token(&self, token: &str) -> Result<AuthClaims, AuthError>;
    fn generate_token(&self, claims: &AuthClaims) -> Result<String, AuthError>;
}

#[derive(Debug, Serialize, Error)] // Added Error
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Repository error: {0}")]
    RepoError(#[from] RepositoryError),
    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;
    async fn save(&self, user: User) -> Result<UserId, RepositoryError>;
}

#[async_trait]
pub trait NodeRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Node>, RepositoryError>;
    async fn save(&self, node: Node, user_id: UserId) -> Result<Uuid, RepositoryError>;
    async fn list_by_parent(&self, parent_id: Option<Uuid>) -> Result<Vec<Node>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait ArticleRepository: Send + Sync {
    async fn save(&self, article: Article, user_id: UserId, change_reason: Option<String>) -> Result<Uuid, RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ContentItem>, RepositoryError>;
    async fn find_by_title(&self, title: &str) -> Result<Option<Article>, RepositoryError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Article>, RepositoryError>;
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>, knowledge_base_id: Option<Uuid>, limit: u64, offset: u64) -> Result<Vec<ContentItem>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn get_version(&self, id: &Uuid, version: &str) -> Result<Option<ContentVersionSnapshot>, RepositoryError>;
    async fn get_history(&self, id: &Uuid) -> Result<Vec<ContentVersionSnapshot>, RepositoryError>;
    async fn get_diff(&self, id: &Uuid, v1: &str, v2: &str) -> Result<ContentDiff, RepositoryError>;
    async fn search(&self, query: &str) -> Result<Vec<Article>, RepositoryError>;
}

#[async_trait]
pub trait VocabularyRepository: Send + Sync {
    async fn save(&self, vocab: Vocabulary) -> Result<Uuid, RepositoryError>;
    async fn find_by_word(&self, user_id: &UserId, word: &str) -> Result<Option<Vocabulary>, RepositoryError>; 
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Vocabulary>, RepositoryError>;
    async fn list(&self, user_id: &UserId, limit: u64, offset: u64, query: Option<String>) -> Result<Vec<Vocabulary>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait MemoRepository: Send + Sync {
    async fn save(&self, memo: Memo) -> Result<Uuid, RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Memo>, RepositoryError>;
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<Memo>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn add_comment(&self, comment: Comment) -> Result<CommentId, RepositoryError>;
    async fn get_comments(&self, target_id: &Uuid) -> Result<Vec<Comment>, RepositoryError>;
    async fn get_comments_batch(&self, target_ids: Vec<Uuid>) -> Result<std::collections::HashMap<Uuid, usize>, RepositoryError>;
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn get_all_tags(&self) -> Result<Vec<String>, RepositoryError>;
}

#[async_trait]
pub trait KnowledgeBaseRepository: Send + Sync {
    async fn save(&self, kb: KnowledgeBase) -> Result<KnowledgeBaseId, RepositoryError>;
    async fn find_by_id(&self, id: &KnowledgeBaseId) -> Result<Option<KnowledgeBase>, RepositoryError>;
    async fn find_by_title(&self, author_id: &UserId, title: &str) -> Result<Option<KnowledgeBase>, RepositoryError>;
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<KnowledgeBase>, RepositoryError>;
    async fn delete(&self, id: &KnowledgeBaseId) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait DraftRepository: Send + Sync {
    async fn save_draft(&self, draft: crate::domain::models::UserDraft) -> Result<(), RepositoryError>;
    async fn get_draft(&self, user_id: &UserId) -> Result<Option<crate::domain::models::UserDraft>, RepositoryError>;
    async fn delete_draft(&self, user_id: &UserId) -> Result<(), RepositoryError>;
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] // Added Clone
pub enum ExportFormat {
    Markdown,
    Json,
    Html,
}

#[async_trait]
pub trait ExportService: Send + Sync {
    async fn export_node_with_comments(
        &self,
        node_id: &Uuid,
        format: ExportFormat,
        requester: Option<UserId>
    ) -> Result<Vec<u8>, RepositoryError>;
}
