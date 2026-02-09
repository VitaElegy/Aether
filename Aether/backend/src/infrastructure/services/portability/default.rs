use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use tokio::sync::mpsc::Sender;

use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::portability::models::{ExportSummary, ImportSummary, ProgressEvent};
use crate::infrastructure::services::backup_service::BackupService;

pub struct DefaultPortabilityProvider {
    backup_service: Arc<BackupService>,
    id_override: Option<String>,
}

impl DefaultPortabilityProvider {
    pub fn new(backup_service: Arc<BackupService>) -> Self {
        Self { backup_service, id_override: None }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id_override = Some(id);
        self
    }
}

#[async_trait]
impl PortabilityProvider for DefaultPortabilityProvider {
    fn provider_id(&self) -> String {
        self.id_override.clone().unwrap_or_else(|| "default".to_string())
    }

    async fn analyze_export(&self, _kb_id: Uuid) -> Result<ExportSummary, String> {
        // Generic backup doesn't have detailed analysis yet, just standard backup
        Ok(ExportSummary {
            total_items: 0,
            estimated_size: "Unknown".to_string(),
            sections: vec![],
        })
    }

    async fn export(&self, kb_id: Uuid, user_id: Uuid, task_id: Uuid, progress: Sender<ProgressEvent>) -> Result<PathBuf, String> {
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Backup".to_string(),
            percent: 0,
            message: "Starting standard backup...".to_string(),
            error: None,
        }).await;

        // Execute Backup
        // Note: create_backup is synchronous-ish but returns a future. It might take time.
        // We should probably report progress inside create_backup if possible, but for now we just wrap it.
        
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Backup".to_string(),
            percent: 10,
            message: "Archiving data...".to_string(),
            error: None,
        }).await;

        let filename = self.backup_service.create_backup(kb_id, user_id).await
            .map_err(|e| e.to_string())?;

        let file_path = self.backup_service.get_backup_path(&filename);

        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Completed".to_string(),
            percent: 100,
            message: "Backup ready.".to_string(),
            error: None,
        }).await;

        Ok(file_path)
    }

    async fn analyze_import(&self, _file_path: PathBuf) -> Result<ImportSummary, String> {
        Ok(ImportSummary {
            total_items: 0,
            sections: vec![],
            conflicts: vec![],
        })
    }

    async fn import(&self, _kb_id: Uuid, _file_path: PathBuf, _task_id: Uuid, _progress: Sender<ProgressEvent>) -> Result<(), String> {
        Ok(())
    }
}
