use async_trait::async_trait;
use serde::Serialize;
use uuid::Uuid;
use thiserror::Error; // Added back
use crate::domain::models::{
    Article, Vocabulary, Memo, User, UserId, AuthClaims, Comment, CommentId,
    ContentVersionSnapshot, Node, KnowledgeBase, KnowledgeBaseId, ContentItem, ContentDiff
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
    async fn search_users(&self, query: &str) -> Result<Vec<User>, RepositoryError>;
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
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>, knowledge_base_id: Option<Uuid>, tag: Option<String>, category: Option<String>, limit: u64, offset: u64) -> Result<Vec<ContentItem>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn get_version(&self, id: &Uuid, version: &str) -> Result<Option<ContentVersionSnapshot>, RepositoryError>;
    async fn get_history(&self, id: &Uuid) -> Result<Vec<ContentVersionSnapshot>, RepositoryError>;
    async fn get_diff(&self, id: &Uuid, v1: &str, v2: &str) -> Result<ContentDiff, RepositoryError>;
    async fn search(&self, query: &str) -> Result<Vec<Article>, RepositoryError>;
    async fn delete_recursive(&self, id: &Uuid) -> Result<(), RepositoryError>;
    // Draft System
    async fn find_drafts_by_article_ids(&self, article_ids: Vec<Uuid>) -> Result<Vec<(Uuid, String, serde_json::Value, chrono::DateTime<chrono::Utc>)>, RepositoryError>;
    async fn find_draft_by_id(&self, article_id: &Uuid) -> Result<Option<(String, serde_json::Value)>, RepositoryError>;
    async fn save_draft(&self, article_id: Uuid, title: String, body: serde_json::Value) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait VocabularyRepository: Send + Sync {
    async fn save(&self, vocab: Vocabulary) -> Result<Uuid, RepositoryError>;
    async fn find_by_word(&self, user_id: &UserId, word: &str) -> Result<Option<Vocabulary>, RepositoryError>; 
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Vocabulary>, RepositoryError>;
    async fn list(&self, user_id: &UserId, limit: u64, offset: u64, query: Option<String>, sort_by: Option<String>, order: Option<String>, knowledge_base_id: Option<Uuid>) -> Result<Vec<Vocabulary>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
    // Batch Operations
    async fn delete_many(&self, ids: &[Uuid]) -> Result<(), RepositoryError>; 
    async fn increment_query_count(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn set_importance(&self, id: &Uuid, is_important: bool) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait MemoRepository: Send + Sync {
    async fn save(&self, memo: Memo) -> Result<Uuid, RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Memo>, RepositoryError>;
    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<Memo>, RepositoryError>;
    async fn find_by_date_range(&self, author_id: UserId, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Result<Vec<Memo>, RepositoryError>;
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
#[allow(dead_code)]
pub trait DraftRepository: Send + Sync {
    async fn save_draft(&self, draft: crate::domain::models::UserDraft) -> Result<(), RepositoryError>;
    async fn get_draft(&self, user_id: &UserId) -> Result<Option<crate::domain::models::UserDraft>, RepositoryError>;
    async fn delete_draft(&self, user_id: &UserId) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait PermissionRepository: Send + Sync {
    // ReBAC Fundamentals: Tuple Operations
    async fn add_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid, subject_type: &str) -> Result<(), RepositoryError>;
    async fn remove_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid, subject_type: &str) -> Result<(), RepositoryError>;
    
    // Check Existence (Direct Lookup)
    async fn has_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid, subject_type: &str) -> Result<bool, RepositoryError>;
    
    // Discovery (Reverse Lookup for Graph Walk)
    async fn get_subject_groups(&self, subject_id: Uuid) -> Result<Vec<Uuid>, RepositoryError>;
    async fn get_parents(&self, entity_id: Uuid) -> Result<Vec<Uuid>, RepositoryError>;
    
    // Metadata Management
    async fn create_group(&self, id: Uuid, name: String) -> Result<Uuid, RepositoryError>;
    
    // Querying
    async fn get_collaborators(&self, entity_id: Uuid, entity_type: &str, relation: &str) -> Result<Vec<Uuid>, RepositoryError>;
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

#[async_trait]
pub trait GraphRepository: Send + Sync {
    async fn save(&self, node: crate::domain::models::GraphNode) -> Result<Uuid, RepositoryError>;
    async fn get_tree(&self, kb_id: &Uuid) -> Result<Vec<crate::domain::models::GraphNode>, RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<crate::domain::models::GraphNode>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait VrkbRepository: Send + Sync {
    // Project
    async fn create_project(&self, project: crate::domain::models::VrkbProject) -> Result<Uuid, RepositoryError>;
    async fn get_project(&self, id: &Uuid) -> Result<Option<crate::domain::models::VrkbProject>, RepositoryError>;
    async fn list_projects(&self) -> Result<Vec<crate::domain::models::VrkbProject>, RepositoryError>;
    
    // Section
    async fn create_section(&self, section: crate::domain::models::VrkbSection) -> Result<Uuid, RepositoryError>;
    async fn list_sections(&self, project_id: &Uuid) -> Result<Vec<crate::domain::models::VrkbSection>, RepositoryError>;
    
    // Finding
    async fn create_finding(&self, finding: crate::domain::models::VrkbFinding) -> Result<Uuid, RepositoryError>;
    async fn get_finding(&self, id: &Uuid) -> Result<Option<crate::domain::models::VrkbFinding>, RepositoryError>;
    async fn list_findings(&self, section_id: Option<Uuid>, project_id: Option<Uuid>) -> Result<Vec<crate::domain::models::VrkbFinding>, RepositoryError>;
    async fn update_finding_status(&self, id: &Uuid, status: String) -> Result<(), RepositoryError>;

    // Asset
    async fn create_asset(&self, asset: crate::domain::models::VrkbAsset) -> Result<Uuid, RepositoryError>;
    async fn get_asset_by_hash(&self, hash: &str) -> Result<Option<crate::domain::models::VrkbAsset>, RepositoryError>;
    async fn link_asset_to_project(&self, project_id: Uuid, asset_id: Uuid, virtual_path: String) -> Result<(), RepositoryError>;
    async fn list_project_assets(&self, project_id: &Uuid) -> Result<Vec<crate::domain::models::VrkbAsset>, RepositoryError>;
    async fn delete_asset(&self, id: &Uuid) -> Result<(), RepositoryError>;

    // Members
    async fn add_member(&self, member: crate::domain::models::VrkbMember) -> Result<(), RepositoryError>;
    async fn remove_member(&self, project_id: &Uuid, user_id: &Uuid) -> Result<(), RepositoryError>;
    async fn list_members(&self, project_id: &Uuid) -> Result<Vec<crate::domain::models::VrkbMember>, RepositoryError>;
    async fn update_member_role(&self, project_id: &Uuid, user_id: &Uuid, role: String) -> Result<(), RepositoryError>;

    // Specs
    async fn get_specs(&self, project_id: &Uuid) -> Result<Vec<crate::domain::models::VrkbSpec>, RepositoryError>;
    async fn save_spec(&self, spec: crate::domain::models::VrkbSpec) -> Result<Uuid, RepositoryError>;

    // Docs
    async fn create_doc(&self, doc: crate::domain::models::VrkbDoc) -> Result<Uuid, RepositoryError>;
    async fn get_doc(&self, id: &Uuid) -> Result<Option<crate::domain::models::VrkbDoc>, RepositoryError>;
    async fn update_doc(&self, doc: crate::domain::models::VrkbDoc) -> Result<(), RepositoryError>;
    async fn delete_doc(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn list_docs(&self, project_id: &Uuid) -> Result<Vec<crate::domain::models::VrkbDoc>, RepositoryError>;
    
    // Trash
    async fn list_trash(&self, project_id: &Uuid) -> Result<Vec<crate::domain::models::VrkbDoc>, RepositoryError>;
    async fn restore_doc(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn permanent_delete_doc(&self, id: &Uuid) -> Result<(), RepositoryError>;
    async fn cleanup_trash(&self, days: i64) -> Result<u64, RepositoryError>;
    
    // Stats
    async fn get_project_stats(&self, project_id: &Uuid) -> Result<crate::domain::models::VrkbStats, RepositoryError>;
}
