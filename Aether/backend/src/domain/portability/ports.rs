use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;
use tokio::sync::mpsc::Sender;
use super::models::{ExportSummary, ImportSummary, ProgressEvent};

#[async_trait]
pub trait PortabilityProvider: Send + Sync {
    /// Return the unique ID of the renderer/kb_type this provider handles (e.g., "english_v1", "math_v1")
    fn provider_id(&self) -> String;

    /// Return a human-readable summary of what will be exported
    async fn analyze_export(&self, kb_id: Uuid) -> Result<ExportSummary, String>;
    
    /// Execute the export, reporting progress via the channel
    /// Returns the path to the generated file (usually a temp zip)
    async fn export(&self, kb_id: Uuid, user_id: Uuid, task_id: Uuid, progress: Sender<ProgressEvent>) -> Result<PathBuf, String>;
    
    /// Analyze an uploaded file for import
    async fn analyze_import(&self, file_path: PathBuf) -> Result<ImportSummary, String>;
    
    /// Execute import
    async fn import(&self, kb_id: Uuid, file_path: PathBuf, task_id: Uuid, progress: Sender<ProgressEvent>) -> Result<(), String>;
}
