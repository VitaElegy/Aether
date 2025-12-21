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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAggregate {
    pub id: ContentId,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub status: ContentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: ContentBody,
    pub tags: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    // We never return the password hash to the frontend, forcing manual field exclusion
    #[serde(skip_serializing)]
    pub password_hash: String,
    // Bitmask for granular permissions.
    // 0001 = Read, 0010 = Write, 0100 = Admin, etc.
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
