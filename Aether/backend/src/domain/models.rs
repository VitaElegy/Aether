use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// --- core Node Domain ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Article,
    Vocabulary,
    Memo,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentItem {
    Article(Article),
    Node(Node),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Uuid,
    pub knowledge_base_id: Option<Uuid>,
    pub r#type: NodeType,
    pub title: String,
    pub permission_mode: PermissionMode,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- Specific Domains (Article, Vocabulary, Memo) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(flatten)]
    pub node: Node, // Flattened so JSON looks like {id:..., title:..., slug:...}
    pub slug: String,
    pub status: ContentStatus,
    pub category: Option<String>,
    pub body: ContentBody,
    pub tags: Vec<String>,
    pub author_name: Option<String>,
    pub author_avatar: Option<String>,
    pub derived_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyExample {
    pub id: Uuid,
    pub sentence: String,
    pub translation: Option<String>,
    pub note: Option<String>,
    pub image_url: Option<String>,
    pub article_id: Option<Uuid>,
    pub sentence_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    #[serde(flatten)]
    pub node: Node,
    pub word: String,
    pub definition: String,
    pub translation: Option<String>,
    pub phonetic: Option<String>,
    // Removed old context_sentence/image_url in favor of examples list, 
    // but keeping for backward compat if needed, or just deprecate.
    // User wants "multiple examples". 
    // Let's deprecate single context_sentence/image_url or map the first example to them.
    // Ideally we return the full objects.
    pub context_sentence: Option<String>, 
    pub image_url: Option<String>, 
    
    pub language: String,
    pub status: String,
    
    // New Fields
    pub root: Option<String>, // The actual root string, e.g. "spec"
    pub examples: Vec<VocabularyExample>,
    #[serde(default)]
    pub query_count: i32,
    #[serde(default)]
    pub is_important: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    #[serde(flatten)]
    pub node: Node,
    pub content: String,
    pub priority: Option<String>, // High, Medium, Low
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub tag: String, // "Equal", "Insert", "Delete"
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDiff {
    pub old_version: String,
    pub new_version: String,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVersionSnapshot {
    pub id: String,
    pub version: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub editor_id: Uuid,
    pub body: Option<ContentBody>, 
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ContentBody {
    Markdown(String),
    CodeSnippet { language: String, code: String },
    Video { url: String, duration_sec: u32 },
    Custom(serde_json::Value),
}

// --- Legacy Types / Auth ---

#[allow(dead_code)]
pub mod permissions {
    pub const READ_PUBLIC: u64 = 1 << 0;
    pub const COMMENT: u64     = 1 << 1;
    pub const CREATE_NODE: u64 = 1 << 4; // Generic Create
    pub const ADMIN: u64       = 1 << 63;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub permissions: u64,
    pub experience: Option<Vec<ExperienceItem>>,
}

impl User {
    pub fn has_permission(&self, required_perm: u64) -> bool {
        (self.permissions & required_perm) == required_perm
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceItem {
    pub id: String,
    pub title: String,
    pub organization: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    pub exp: usize,
    pub perms: u64,
}

// --- Comment Domain (Generic) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CommentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub target_id: Uuid, // Generic Link to Node
    pub user_id: UserId,
    pub user_name: Option<String>,
    pub user_avatar: Option<String>,
    pub parent_id: Option<CommentId>,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub replies: Vec<Comment>,
}


// --- Knowledge Base Domain ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KnowledgeBaseId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: KnowledgeBaseId,
    pub author_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub cover_image: Option<String>,
    pub cover_offset_y: i32,
    pub renderer_id: Option<String>, 
    pub visibility: Visibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDraft {
    pub user_id: UserId,
    pub target_article_id: Option<Uuid>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub knowledge_base_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub knowledge_base_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub label: String,
    pub data: serde_json::Value,
    pub rank: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- VRKB Domain ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbProject {
    pub id: Uuid,
    pub name: String,
    pub repository_url: Option<String>,
    pub quota_bytes: i64,
    pub settings: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbSection {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub checklist: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbFinding {
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub content: Option<serde_json::Value>,
    pub is_triage: bool,
    pub author_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbAsset {
    pub id: Uuid,
    pub hash: String,
    pub storage_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbMember {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    // Optional Join for Listing
    pub user: Option<User>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbSpec {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub version: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbDoc {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub content: Option<serde_json::Value>,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbStats {
    pub metrics: VrkbMetrics,
    pub modules: Vec<VrkbModuleStat>,
    pub heatmap: Vec<VrkbHeatmapItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbMetrics {
    pub total: i64,
    pub critical: i64,
    pub triage: i64,
    pub fixed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbModuleStat {
    pub name: String,
    pub status: String,
    pub progress: u32,
    pub bugs: i64,
    pub last_audit: String, // Simplified string for now, could be timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrkbHeatmapItem {
    pub path: String,
    pub name: String,
    pub r#type: String, // "folder" or "file"
    pub level: i32,
    pub vulns: i64,
}
