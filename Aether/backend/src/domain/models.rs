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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    #[serde(flatten)]
    pub node: Node,
    pub word: String,
    pub definition: String,
    pub translation: Option<String>,
    pub phonetic: Option<String>,
    pub context_sentence: Option<String>, 
    pub image_url: Option<String>, 
    pub language: String,
    pub status: String,
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
pub struct ContentVersionSnapshot {
    pub id: String,
    pub version: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub editor_id: Uuid,
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
}

impl User {
    pub fn has_permission(&self, required_perm: u64) -> bool {
        (self.permissions & required_perm) == required_perm
    }
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

