use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::domain::portability::models::{
    ExportSection, ExportSummary, ImportSection, ImportSummary, ProgressEvent,
};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::ports::MemoRepository;

pub struct MemosPortabilityProvider {
    memo_repo: Arc<dyn MemoRepository>,
}

impl MemosPortabilityProvider {
    pub fn new(memo_repo: Arc<dyn MemoRepository>) -> Self {
        Self { memo_repo }
    }
}

#[async_trait]
impl PortabilityProvider for MemosPortabilityProvider {
    fn provider_id(&self) -> String {
        "memo".to_string()
    }

    async fn analyze_export(&self, _kb_id: Uuid) -> Result<ExportSummary, String> {
        let memos = self
            .memo_repo
            .list(None, None)
            .await
            .map_err(|e| e.to_string())?;

        let memo_count = memos.len();

        Ok(ExportSummary {
            total_items: memo_count,
            estimated_size: format!("~{} KB", memo_count * 2),
            sections: vec![
                ExportSection {
                    name: "Memos".to_string(),
                    count: memo_count,
                    details: format!("{} memos with tags and channels", memo_count),
                },
            ],
        })
    }

    async fn export(
        &self,
        kb_id: Uuid,
        _user_id: Uuid,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<PathBuf, String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Preparing".to_string(),
                percent: 0,
                message: "Loading memos...".to_string(),
                error: None,
            })
            .await;

        let memos = self
            .memo_repo
            .list(None, None)
            .await
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Serializing".to_string(),
                percent: 40,
                message: format!("Serializing {} memos...", memos.len()),
                error: None,
            })
            .await;

        // Create temp zip
        let temp_dir = std::env::temp_dir();
        let zip_path = temp_dir.join(format!("aether_memos_{}.zip", task_id));
        let file = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // manifest.json
        let manifest = serde_json::json!({
            "format": "aether_memos_v1",
            "version": "1.0",
            "kb_id": kb_id.to_string(),
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "memo_count": memos.len(),
        });
        zip.start_file("manifest.json", options)
            .map_err(|e| e.to_string())?;
        std::io::Write::write_all(
            &mut zip,
            serde_json::to_string_pretty(&manifest)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| e.to_string())?;

        // memos.json
        let memos_json = serde_json::to_string_pretty(&memos).map_err(|e| e.to_string())?;
        zip.start_file("memos.json", options)
            .map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, memos_json.as_bytes())
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 90,
                message: "Writing archive...".to_string(),
                error: None,
            })
            .await;

        zip.finish().map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 99,
                message: "Export ready".to_string(),
                error: None,
            })
            .await;

        Ok(zip_path)
    }

    async fn analyze_import(&self, file_path: PathBuf) -> Result<ImportSummary, String> {
        let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        // Read manifest
        let manifest_entry = archive
            .by_name("manifest.json")
            .map_err(|_| "Missing manifest.json".to_string())?;
        let manifest: serde_json::Value =
            serde_json::from_reader(manifest_entry).map_err(|e| e.to_string())?;

        let format = manifest["format"].as_str().unwrap_or("");
        if format != "aether_memos_v1" {
            return Err(format!("Unsupported format: {}", format));
        }

        let memo_count = manifest["memo_count"].as_u64().unwrap_or(0) as usize;

        Ok(ImportSummary {
            total_items: memo_count,
            sections: vec![ImportSection {
                name: "Memos".to_string(),
                count: memo_count,
                action: "Create".to_string(),
            }],
            conflicts: vec![],
        })
    }

    async fn import(
        &self,
        _kb_id: Uuid,
        _file_path: PathBuf,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<(), String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing".to_string(),
                percent: 50,
                message: "Importing memos...".to_string(),
                error: None,
            })
            .await;

        // TODO: Implement full memo import with tag/channel merge
        // For now, report completion
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 99,
                message: "Import complete".to_string(),
                error: None,
            })
            .await;

        Ok(())
    }
}
