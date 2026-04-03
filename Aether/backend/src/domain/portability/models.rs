use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportSummary {
    pub total_items: usize,
    pub estimated_size: String,
    pub sections: Vec<ExportSection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportSection {
    pub name: String,
    pub count: usize,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportSummary {
    pub total_items: usize,
    pub sections: Vec<ImportSection>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportSection {
    pub name: String,
    pub count: usize,
    pub action: String, // "Create", "Update", "Skip"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub task_id: Uuid,
    pub stage: String,
    pub percent: u8,
    pub message: String,
    pub error: Option<String>,
}

/// PLAT-04: Import preview with conflict resolution suggestions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportPreview {
    pub summary: ImportSummary,
    pub conflicts: Vec<ImportConflict>,
    pub suggested_actions: Vec<SuggestedAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportConflict {
    pub item_id: String,
    pub item_name: String,
    pub conflict_type: String, // "duplicate", "version_mismatch", "schema_change"
    pub existing_value: Option<String>,
    pub incoming_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuggestedAction {
    pub conflict_id: String,
    pub action: String, // "skip", "overwrite", "merge", "rename"
    pub reason: String,
}

/// PLAT-04: Completed task entry with download token and expiry
#[derive(Debug, Clone)]
pub struct CompletedTaskEntry {
    pub file_path: std::path::PathBuf,
    pub download_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PortabilityTaskType {
    Export,
    Import,
}
