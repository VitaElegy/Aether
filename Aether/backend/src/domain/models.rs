use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// --- Content Domain ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Internal, // Visible to any logged-in user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAggregate {
    pub id: ContentId,
    pub author_id: Uuid,
    pub author_name: Option<String>, // Added for UI convenience
    pub title: String,
    pub slug: String,
    pub status: ContentStatus,
    pub visibility: Visibility, // Added
    pub category: Option<String>, // Added: "Parent/Child" format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: ContentBody,
    pub tags: Vec<String>,
    // Context for persistence, not part of domain state per se, but useful for transport
    pub version_message: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ContentBody {
    Markdown(String),
    CodeSnippet { language: String, code: String },
    Video { url: String, duration_sec: u32 },
    Custom(serde_json::Value),
}

// --- Authentication Domain ---

#[allow(dead_code)]
pub mod permissions {
    #[allow(dead_code)]
    pub const READ_PUBLIC: u64 = 1 << 0; // 1
    pub const COMMENT: u64     = 1 << 1; // 2

    // Content (Articles)
    pub const CREATE_POST: u64 = 1 << 4; // 16
    #[allow(dead_code)]
    pub const EDIT_POST: u64   = 1 << 5; // 32
    #[allow(dead_code)]
    pub const DELETE_POST: u64 = 1 << 6; // 64

    // Feature: Memos (Future Proofing)
    #[allow(dead_code)]
    pub const MEMO_READ: u64   = 1 << 8; // 256
    #[allow(dead_code)]
    pub const MEMO_WRITE: u64  = 1 << 9; // 512

    // Feature: Todos (Future Proofing)
    pub const TODO_READ: u64   = 1 << 12; // 4096
    pub const TODO_WRITE: u64  = 1 << 13; // 8192

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
    // We never return the password hash to the frontend, forcing manual field exclusion
    #[serde(skip_serializing)]
    pub password_hash: String,
    // Bitmask for granular permissions. See permissions module.
    pub permissions: u64,
}

impl User {
    #[allow(dead_code)]
    pub fn has_permission(&self, required_perm: u64) -> bool {
        (self.permissions & required_perm) == required_perm
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiry
    pub perms: u64,  // Permissions snapshot
}

// --- Comment Domain ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub content_id: ContentId,
    pub user_id: UserId,
    pub user_name: Option<String>,
    pub user_avatar: Option<String>,
    pub parent_id: Option<CommentId>,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub replies: Vec<Comment>, // For nested display
}