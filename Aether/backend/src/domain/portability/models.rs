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

#[derive(Debug, Clone)]
pub enum PortabilityTaskType {
    Export,
    Import,
}
