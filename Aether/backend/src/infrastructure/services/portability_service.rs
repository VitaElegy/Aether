use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;

use uuid::Uuid;
use tokio::sync::mpsc::{self, Receiver};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::portability::models::{ExportSummary, ProgressEvent};

pub struct PortabilityService {
    providers: HashMap<String, Arc<dyn PortabilityProvider>>,
    // Simple in-memory task tracking for MVP. 
    // In production, this might need Redis or DB to survive restarts, 
    // but for "Download" tasks, memory is usually fine.
    active_tasks: Arc<RwLock<HashMap<Uuid, Receiver<ProgressEvent>>>>,
    // Track finished export files
    completed_tasks: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
}

impl PortabilityService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn PortabilityProvider>) {
        self.providers.insert(provider.provider_id(), provider);
    }

    fn get_provider(&self, renderer_id: &str) -> Result<Arc<dyn PortabilityProvider>, String> {
        self.providers.get(renderer_id)
            .cloned()
            .ok_or_else(|| format!("No portability provider found for type: {}", renderer_id))
    }

    pub async fn analyze_export(&self, kb_type: &str, kb_id: Uuid) -> Result<ExportSummary, String> {
        let provider = self.get_provider(kb_type)?;
        provider.analyze_export(kb_id).await
    }

    pub async fn start_export(&self, kb_type: &str, kb_id: Uuid, user_id: Uuid) -> Result<Uuid, String> {
        let provider = self.get_provider(kb_type)?;
        let task_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(100);

        // Store receiver
        self.active_tasks.write().unwrap().insert(task_id, rx);

        let completed_map = Arc::clone(&self.completed_tasks);

        // Spawn task
        tokio::spawn(async move {
            match provider.export(kb_id, user_id, task_id, tx.clone()).await {
                Ok(path) => {
                    tracing::info!("Export task {} completed. File waiting at {:?}", task_id, path);
                    completed_map.write().unwrap().insert(task_id, path);
                    
                    // Dispatch Completed only AFTER the path is safely tucked into the completed map.
                    let _ = tx.send(ProgressEvent {
                        task_id,
                        stage: "Completed".to_string(),
                        percent: 100,
                        message: "Export ready for download.".to_string(),
                        error: None,
                    }).await;
                },
                Err(e) => {
                    tracing::error!("Export task {} failed: {}", task_id, e);
                    let _ = tx.send(ProgressEvent {
                        task_id,
                        stage: "Error".to_string(),
                        percent: 100,
                        message: "Export failed".to_string(),
                        error: Some(e),
                    }).await;
                }
            }
        });

        Ok(task_id)
    }

    pub fn get_task_receiver(&self, task_id: Uuid) -> Option<Receiver<ProgressEvent>> {
        self.active_tasks.write().unwrap().remove(&task_id)
    }

    pub fn get_task_result(&self, task_id: Uuid) -> Option<PathBuf> {
        self.completed_tasks.read().unwrap().get(&task_id).cloned()
    }

    pub async fn start_import(
        &self,
        backup_service: Arc<crate::infrastructure::services::backup_service::BackupService>,
        file_path: PathBuf,
        user_id: Uuid,
    ) -> Result<Uuid, String> {
        let task_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(100);

        self.active_tasks.write().unwrap().insert(task_id, rx);

        tokio::spawn(async move {
            match backup_service.restore_backup(file_path, user_id, Some(task_id), Some(tx.clone())).await {
                Ok(new_kb_id) => {
                    tracing::info!("Import task {} completed successfully. New KB: {}", task_id, new_kb_id);
                    let _ = tx.send(ProgressEvent {
                        task_id,
                        stage: "Completed".to_string(),
                        percent: 100,
                        message: "Knowledge Base successfully imported.".to_string(),
                        error: None,
                    }).await;
                },
                Err(e) => {
                    tracing::error!("Import task {} failed: {}", task_id, e);
                    let _ = tx.send(ProgressEvent {
                        task_id,
                        stage: "Error".to_string(),
                        percent: 100,
                        message: "Import Failed".to_string(),
                        error: Some(e),
                    }).await;
                }
            }
        });

        Ok(task_id)
    }
}
