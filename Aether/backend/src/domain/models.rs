use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Article analysis status for the English workspace state machine.
/// Transitions: Pending → Analyzing → Analyzed | Failed → Archived
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisStatus {
    /// Article is in inbox, not yet analyzed
    Pending,
    /// Analysis is in progress (NLP pipeline running)
    Analyzing,
    /// Analysis completed successfully
    Analyzed,
    /// Analysis failed — stores diagnostic info
    Failed,
    /// Article has been archived (soft-delete from active view)
    Archived,
}

impl AnalysisStatus {
    /// Returns true if a transition from self → target is valid.
    pub fn can_transition_to(&self, target: &AnalysisStatus) -> bool {
        matches!(
            (self, target),
            (AnalysisStatus::Pending, AnalysisStatus::Analyzing)
                | (AnalysisStatus::Analyzing, AnalysisStatus::Analyzed)
                | (AnalysisStatus::Analyzing, AnalysisStatus::Failed)
                | (AnalysisStatus::Analyzed, AnalysisStatus::Analyzing) // re-analyze
                | (AnalysisStatus::Analyzed, AnalysisStatus::Archived)
                | (AnalysisStatus::Failed, AnalysisStatus::Analyzing) // retry
                | (AnalysisStatus::Failed, AnalysisStatus::Archived)
                | (AnalysisStatus::Archived, AnalysisStatus::Pending) // restore
        )
    }
}

impl Default for AnalysisStatus {
    fn default() -> Self {
        AnalysisStatus::Pending
    }
}

impl std::fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisStatus::Pending => write!(f, "pending"),
            AnalysisStatus::Analyzing => write!(f, "analyzing"),
            AnalysisStatus::Analyzed => write!(f, "analyzed"),
            AnalysisStatus::Failed => write!(f, "failed"),
            AnalysisStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for AnalysisStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(AnalysisStatus::Pending),
            "analyzing" => Ok(AnalysisStatus::Analyzing),
            "analyzed" => Ok(AnalysisStatus::Analyzed),
            "failed" => Ok(AnalysisStatus::Failed),
            "archived" => Ok(AnalysisStatus::Archived),
            _ => Err(format!("Unknown analysis status: {}", s)),
        }
    }
}

/// Failure diagnostics for article analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDiagnostics {
    pub error_code: Option<String>,
    pub error_message: String,
    pub failed_at: DateTime<Utc>,
    pub retry_count: u32,
}

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
    /// Analysis status for the English article workspace state machine
    #[serde(default)]
    pub analysis_status: Option<AnalysisStatus>,
    /// Failure diagnostics when analysis_status is Failed
    #[serde(default)]
    pub analysis_diagnostics: Option<AnalysisDiagnostics>,
}

/// CEFR-aligned proficiency level for vocabulary items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VocabularyLevel {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
    Unknown,
}

impl Default for VocabularyLevel {
    fn default() -> Self {
        VocabularyLevel::Unknown
    }
}

impl std::fmt::Display for VocabularyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VocabularyLevel::A1 => write!(f, "A1"),
            VocabularyLevel::A2 => write!(f, "A2"),
            VocabularyLevel::B1 => write!(f, "B1"),
            VocabularyLevel::B2 => write!(f, "B2"),
            VocabularyLevel::C1 => write!(f, "C1"),
            VocabularyLevel::C2 => write!(f, "C2"),
            VocabularyLevel::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Mastery status tracking for spaced repetition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MasteryStatus {
    /// First seen, never reviewed
    New,
    /// Seen a few times, still learning
    Learning,
    /// Comfortable but not automatic
    Familiar,
    /// Well known, recall is fast
    Mastered,
}

impl Default for MasteryStatus {
    fn default() -> Self {
        MasteryStatus::New
    }
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
    pub global_sentence_id: Option<Uuid>,
    /// Whether this is the primary/featured example for the word
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    #[serde(flatten)]
    pub node: Node,
    /// The canonical lemma form (e.g. "run" for "running")
    pub lemma: Option<String>,
    /// The actual word as queried/saved by user
    pub word: String,
    pub definition: String,
    pub translation: Option<String>,
    pub phonetic: Option<String>,

    // Deprecated fields kept for backward compat
    pub context_sentence: Option<String>,
    pub image_url: Option<String>,

    pub language: String,
    pub status: String,

    /// Word root / morphology string (e.g. "[Root] spec : to look | [Suffix] tion : noun form")
    pub root: Option<String>,
    pub examples: Vec<VocabularyExample>,
    #[serde(default)]
    pub query_count: i32,
    #[serde(default)]
    pub is_important: bool,

    // ENG-03: New formal fields
    /// CEFR proficiency level
    #[serde(default)]
    pub level: VocabularyLevel,
    /// User-defined tags for grouping (e.g. "academic", "ielts", "tech")
    #[serde(default)]
    pub tags: Vec<String>,
    /// Mastery status for spaced repetition
    #[serde(default)]
    pub mastery: MasteryStatus,
    /// Source KB ID where this word was first encountered
    pub source_kb_id: Option<Uuid>,
    /// Soft-delete: archived words are hidden from default view
    #[serde(default)]
    pub is_archived: bool,
}

// --- Memo Linked Entity (MEMO-05: Backlinks) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedEntity {
    pub target_id: Uuid,
    pub target_type: String, // "article", "asset", "paper", "finding", "doc", "memo"
    pub target_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_text: Option<String>,
}

// --- Memo Reference (MEMO-05: mention / backlink) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoReference {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub ref_type: String, // "mention", "backlink", "embed"
    pub context: Option<String>,
}

// --- Saved View (MEMO-03) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub filters: SavedViewFilters,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub view_mode: Option<String>, // stream, masonry, kanban, timeline, calendar
    pub pinned: bool,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedViewFilters {
    #[serde(default)]
    pub tags: Vec<String>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub is_pinned: Option<bool>,
    // MEMO-06: review queue filters
    pub queue: Option<String>, // "due_today", "overdue", "stale", "unresolved"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    #[serde(flatten)]
    pub node: Node,
    pub content: String,
    pub priority: String, // P0, P1...
    pub status: String,   // Todo, Doing...
    pub color: String,    // Yellow, Red...
    pub is_pinned: bool,
    pub due_at: Option<DateTime<Utc>>,
    pub reminder_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    // MEMO-01: Stream Core
    pub channel: Option<String>,
    #[serde(default)]
    pub excerpt: Option<String>,
    // MEMO-05: Backlinks and References
    #[serde(default)]
    pub linked_entities: Vec<LinkedEntity>,
    // MEMO-06: Rhythm and Review
    pub scheduled_at: Option<DateTime<Utc>>,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
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
    pub const COMMENT: u64 = 1 << 1;
    pub const CREATE_NODE: u64 = 1 << 4; // Generic Create
    pub const ADMIN: u64 = 1 << 63;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTemplate {
    pub id: Uuid,
    pub renderer_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: Option<String>,
    pub tags: Vec<String>,
    #[serde(default = "default_config")]
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_config() -> serde_json::Value {
    serde_json::json!({})
}
