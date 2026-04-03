use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::domain::portability::models::{
    CompletedTaskEntry, ExportSummary, ImportPreview, ImportConflict as DomainImportConflict,
    ImportSummary, ProgressEvent, SuggestedAction,
};
use crate::domain::portability::ports::PortabilityProvider;
use chrono::Utc;
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;

/// Default download token expiry duration (24 hours)
const DOWNLOAD_TOKEN_EXPIRY_HOURS: i64 = 24;

pub struct PortabilityService {
    providers: HashMap<String, Arc<dyn PortabilityProvider>>,
    aliases: HashMap<String, String>,
    // Simple in-memory task tracking for MVP.
    // In production, this might need Redis or DB to survive restarts,
    // but for "Download" tasks, memory is usually fine.
    active_tasks: Arc<RwLock<HashMap<Uuid, Receiver<ProgressEvent>>>>,
    // PLAT-04: Track finished export files with token/expiry
    completed_tasks: Arc<RwLock<HashMap<Uuid, CompletedTaskEntry>>>,
    // PLAT-04: Track uploaded import files pending analysis
    pending_imports: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::portability::models::{ExportSection, ImportSummary};
    use crate::domain::portability::ports::PortabilityProvider;
    use async_trait::async_trait;
    use tokio::time::{timeout, Duration};

    struct FakeProvider {
        id: String,
        should_fail: bool,
    }

    #[async_trait]
    impl PortabilityProvider for FakeProvider {
        fn provider_id(&self) -> String {
            self.id.clone()
        }

        async fn analyze_export(&self, _kb_id: Uuid) -> Result<ExportSummary, String> {
            Ok(ExportSummary {
                total_items: 3,
                estimated_size: "1 MB".to_string(),
                sections: vec![ExportSection {
                    name: "Docs".to_string(),
                    count: 3,
                    details: "Test payload".to_string(),
                }],
            })
        }

        async fn export(
            &self,
            _kb_id: Uuid,
            _user_id: Uuid,
            task_id: Uuid,
            progress: tokio::sync::mpsc::Sender<ProgressEvent>,
        ) -> Result<PathBuf, String> {
            let _ = progress
                .send(ProgressEvent {
                    task_id,
                    stage: "Exporting".to_string(),
                    percent: 50,
                    message: "Halfway".to_string(),
                    error: None,
                })
                .await;

            if self.should_fail {
                Err("simulated failure".to_string())
            } else {
                Ok(PathBuf::from("/tmp/fake-export.zip"))
            }
        }

        async fn analyze_import(&self, _file_path: PathBuf) -> Result<ImportSummary, String> {
            Ok(ImportSummary {
                total_items: 0,
                sections: vec![],
                conflicts: vec![],
            })
        }

        async fn import(
            &self,
            _kb_id: Uuid,
            _file_path: PathBuf,
            _task_id: Uuid,
            _progress: tokio::sync::mpsc::Sender<ProgressEvent>,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn start_export_stores_result_and_emits_completed_event() {
        let mut service = PortabilityService::new();
        service.register_provider(Arc::new(FakeProvider {
            id: "fake".to_string(),
            should_fail: false,
        }));

        let task_id = service
            .start_export("fake", Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
        let mut rx = service
            .get_task_receiver(task_id)
            .expect("receiver should exist");

        let first = timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();
        let second = timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();

        expect_stage(&first, "Exporting");
        expect_stage(&second, "Completed");
        assert_eq!(
            service.get_task_result(task_id),
            Some(PathBuf::from("/tmp/fake-export.zip"))
        );
    }

    #[tokio::test]
    async fn start_export_emits_error_event_on_failure() {
        let mut service = PortabilityService::new();
        service.register_provider(Arc::new(FakeProvider {
            id: "fake".to_string(),
            should_fail: true,
        }));

        let task_id = service
            .start_export("fake", Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
        let mut rx = service
            .get_task_receiver(task_id)
            .expect("receiver should exist");

        let _ = timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();
        let final_event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();

        expect_stage(&final_event, "Error");
        assert_eq!(final_event.error.as_deref(), Some("simulated failure"));
        assert_eq!(service.get_task_result(task_id), None);
    }

    #[test]
    fn provider_aliases_resolve_to_canonical_provider() {
        let mut service = PortabilityService::new();
        service.register_provider(Arc::new(FakeProvider {
            id: "english_v1".to_string(),
            should_fail: false,
        }));
        service.register_alias("english", "english_v1");
        service.register_alias("vocabulary", "english_v1");

        assert_eq!(
            service.resolve_provider_id("english").as_deref(),
            Some("english_v1")
        );
        assert_eq!(
            service.resolve_provider_id("vocabulary").as_deref(),
            Some("english_v1")
        );
        assert_eq!(
            service.resolve_provider_id("english_v1").as_deref(),
            Some("english_v1")
        );
        assert!(service.get_provider("english").is_ok());
    }

    fn expect_stage(event: &ProgressEvent, stage: &str) {
        assert_eq!(event.stage, stage);
    }
}

impl PortabilityService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            aliases: HashMap::new(),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            pending_imports: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn PortabilityProvider>) {
        let provider_id = Self::normalize_provider_id(&provider.provider_id());
        self.providers.insert(provider_id, provider);
    }

    pub fn register_alias(&mut self, alias: &str, provider_id: &str) {
        self.aliases.insert(
            Self::normalize_provider_id(alias),
            Self::normalize_provider_id(provider_id),
        );
    }

    fn normalize_provider_id(provider_id: &str) -> String {
        provider_id.trim().to_lowercase()
    }

    fn resolve_provider_id(&self, renderer_id: &str) -> Option<String> {
        let normalized = Self::normalize_provider_id(renderer_id);
        if self.providers.contains_key(&normalized) {
            return Some(normalized);
        }

        self.aliases.get(&normalized).cloned()
    }

    fn get_provider(&self, renderer_id: &str) -> Result<Arc<dyn PortabilityProvider>, String> {
        let resolved_provider_id = self
            .resolve_provider_id(renderer_id)
            .ok_or_else(|| format!("No portability provider found for type: {}", renderer_id))?;

        self.providers
            .get(&resolved_provider_id)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "No portability provider found for type: {} (resolved: {})",
                    renderer_id, resolved_provider_id
                )
            })
    }

    pub async fn analyze_export(
        &self,
        kb_type: &str,
        kb_id: Uuid,
    ) -> Result<ExportSummary, String> {
        let provider = self.get_provider(kb_type)?;
        provider.analyze_export(kb_id).await
    }

    pub async fn start_export(
        &self,
        kb_type: &str,
        kb_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, String> {
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
                    tracing::info!(
                        "Export task {} completed. File waiting at {:?}",
                        task_id,
                        path
                    );
                    // PLAT-04: Store with download token and expiry
                    let now = Utc::now();
                    let entry = CompletedTaskEntry {
                        file_path: path,
                        download_token: Uuid::new_v4().to_string(),
                        created_at: now,
                        expires_at: now + chrono::Duration::hours(DOWNLOAD_TOKEN_EXPIRY_HOURS),
                    };
                    completed_map.write().unwrap().insert(task_id, entry);

                    // Dispatch Completed only AFTER the path is safely tucked into the completed map.
                    let _ = tx
                        .send(ProgressEvent {
                            task_id,
                            stage: "Completed".to_string(),
                            percent: 100,
                            message: "Export ready for download.".to_string(),
                            error: None,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!("Export task {} failed: {}", task_id, e);
                    let _ = tx
                        .send(ProgressEvent {
                            task_id,
                            stage: "Error".to_string(),
                            percent: 100,
                            message: "Export failed".to_string(),
                            error: Some(e),
                        })
                        .await;
                }
            }
        });

        Ok(task_id)
    }

    pub fn get_task_receiver(&self, task_id: Uuid) -> Option<Receiver<ProgressEvent>> {
        self.active_tasks.write().unwrap().remove(&task_id)
    }

    pub fn get_task_result(&self, task_id: Uuid) -> Option<PathBuf> {
        let tasks = self.completed_tasks.read().unwrap();
        let entry = tasks.get(&task_id)?;
        // PLAT-04: Check expiry
        if Utc::now() > entry.expires_at {
            return None;
        }
        Some(entry.file_path.clone())
    }

    /// PLAT-04: Get the download token for a completed task
    pub fn get_download_token(&self, task_id: Uuid) -> Option<String> {
        let tasks = self.completed_tasks.read().unwrap();
        let entry = tasks.get(&task_id)?;
        if Utc::now() > entry.expires_at {
            return None;
        }
        Some(entry.download_token.clone())
    }

    /// PLAT-04: Validate a download token against a task
    pub fn validate_download_token(&self, task_id: Uuid, token: &str) -> Result<PathBuf, String> {
        let tasks = self.completed_tasks.read().unwrap();
        let entry = tasks.get(&task_id).ok_or("Task not found".to_string())?;
        if Utc::now() > entry.expires_at {
            return Err("Download link has expired".to_string());
        }
        if entry.download_token != token {
            return Err("Invalid download token".to_string());
        }
        Ok(entry.file_path.clone())
    }

    /// PLAT-04: Analyze an import file using the appropriate provider
    pub async fn analyze_import(
        &self,
        kb_type: &str,
        file_path: PathBuf,
    ) -> Result<ImportPreview, String> {
        let provider = self.get_provider(kb_type)?;
        let summary = provider.analyze_import(file_path.clone()).await?;

        // Store the file path for later import start
        let analyze_id = Uuid::new_v4();
        self.pending_imports.write().unwrap().insert(analyze_id, file_path);

        // Build ImportPreview from the provider's ImportSummary
        let conflicts: Vec<DomainImportConflict> = summary
            .conflicts
            .iter()
            .enumerate()
            .map(|(i, desc)| DomainImportConflict {
                item_id: format!("conflict_{}", i),
                item_name: desc.clone(),
                conflict_type: "duplicate".to_string(),
                existing_value: None,
                incoming_value: None,
            })
            .collect();

        let suggested_actions: Vec<SuggestedAction> = conflicts
            .iter()
            .map(|c| SuggestedAction {
                conflict_id: c.item_id.clone(),
                action: "skip".to_string(),
                reason: "Default: skip duplicates".to_string(),
            })
            .collect();

        Ok(ImportPreview {
            summary,
            conflicts,
            suggested_actions,
        })
    }

    /// PLAT-04: Start import using the appropriate provider (not backup_service)
    pub async fn start_import_with_provider(
        &self,
        kb_type: &str,
        kb_id: Uuid,
        file_path: PathBuf,
    ) -> Result<Uuid, String> {
        let provider = self.get_provider(kb_type)?;
        let task_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(100);

        self.active_tasks.write().unwrap().insert(task_id, rx);

        tokio::spawn(async move {
            match provider
                .import(kb_id, file_path, task_id, tx.clone())
                .await
            {
                Ok(()) => {
                    tracing::info!(
                        "Import task {} completed successfully for KB {}",
                        task_id,
                        kb_id
                    );
                    let _ = tx
                        .send(ProgressEvent {
                            task_id,
                            stage: "Completed".to_string(),
                            percent: 100,
                            message: "Import completed successfully.".to_string(),
                            error: None,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!("Import task {} failed: {}", task_id, e);
                    let _ = tx
                        .send(ProgressEvent {
                            task_id,
                            stage: "Error".to_string(),
                            percent: 100,
                            message: "Import failed".to_string(),
                            error: Some(e),
                        })
                        .await;
                }
            }
        });

        Ok(task_id)
    }

    /// Legacy: start_import using backup_service (kept for backward compatibility)
    pub async fn start_import_legacy(
        &self,
        backup_service: Arc<crate::infrastructure::services::backup_service::BackupService>,
        file_path: PathBuf,
        user_id: Uuid,
    ) -> Result<Uuid, String> {
        let task_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(100);

        self.active_tasks.write().unwrap().insert(task_id, rx);

        tokio::spawn(async move {
            match backup_service
                .restore_backup(file_path, user_id, Some(task_id), Some(tx.clone()))
                .await
            {
                Ok(new_kb_id) => {
                    tracing::info!(
                        "Import task {} completed successfully. New KB: {}",
                        task_id,
                        new_kb_id
                    );
                    let _ = tx
                        .send(ProgressEvent {
                            task_id,
                            stage: "Completed".to_string(),
                            percent: 100,
                            message: "Knowledge Base successfully imported.".to_string(),
                            error: None,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!("Import task {} failed: {}", task_id, e);
                    let _ = tx
                        .send(ProgressEvent {
                            task_id,
                            stage: "Error".to_string(),
                            percent: 100,
                            message: "Import Failed".to_string(),
                            error: Some(e),
                        })
                        .await;
                }
            }
        });

        Ok(task_id)
    }
}
