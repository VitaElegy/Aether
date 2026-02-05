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
}

impl DefaultPortabilityProvider {
    pub fn new(backup_service: Arc<BackupService>) -> Self {
        Self { backup_service }
    }
}

#[async_trait]
impl PortabilityProvider for DefaultPortabilityProvider {
    fn provider_id(&self) -> String {
        "default".to_string()
    }

    async fn analyze_export(&self, _kb_id: Uuid) -> Result<ExportSummary, String> {
        // Generic backup doesn't have detailed analysis yet, just standard backup
        Ok(ExportSummary {
            total_items: 0,
            estimated_size: "Unknown".to_string(),
            sections: vec![],
        })
    }

    async fn export(&self, kb_id: Uuid, task_id: Uuid, progress: Sender<ProgressEvent>) -> Result<PathBuf, String> {
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Backup".to_string(),
            percent: 0,
            message: "Starting standard backup...".to_string(),
            error: None,
        }).await;

        // We need user_id for backup service, but export() trait doesn't have it.
        // This is a flaw in the trait design if we reuse BackupService which requires user_id.
        // However, we can fetch the KB to get the owner_id, assuming the requester is authorized.
        // But wait, the Service layer calling this should have verified auth.
        // We'll need to refactor BackupService or pass a dummy ID if we trust the caller?
        // Actually, BackupService::create_backup checks `kb.author_id == user_id`.
        // So we MUST pass the correct user_id.
        
        // FIX: Trait should probably accept `user_id` or `context`.
        // For now, I'll fail or hack it. 
        // Let's assume the caller (PortabilityService) will handle auth, but we need to pass it down.
        
        Err("Default Provider not fully implemented for new trait signature".to_string())
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
