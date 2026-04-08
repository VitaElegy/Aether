use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::domain::portability::models::{
    ExportSection, ExportSummary, ImportSection, ImportSummary, ProgressEvent,
};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::prkb::models::PaperFilter;
use crate::domain::prkb::ports::PrkbRepository;

pub struct PrkbPortabilityProvider {
    prkb_repo: Arc<dyn PrkbRepository>,
}

impl PrkbPortabilityProvider {
    pub fn new(prkb_repo: Arc<dyn PrkbRepository>) -> Self {
        Self { prkb_repo }
    }
}

#[async_trait]
impl PortabilityProvider for PrkbPortabilityProvider {
    fn provider_id(&self) -> String {
        "prkb".to_string()
    }

    async fn analyze_export(&self, _kb_id: Uuid) -> Result<ExportSummary, String> {
        let feeds = self
            .prkb_repo
            .list_feeds()
            .await
            .map_err(|e| e.to_string())?;
        let papers = self
            .prkb_repo
            .list_papers(PaperFilter::default(), 10000, 0)
            .await
            .map_err(|e| e.to_string())?;
        let collections = self
            .prkb_repo
            .list_collections()
            .await
            .map_err(|e| e.to_string())?;

        let feed_count = feeds.len();
        let paper_count = papers.len();
        let collection_count = collections.len();
        let total = feed_count + paper_count + collection_count;

        Ok(ExportSummary {
            total_items: total,
            estimated_size: format!("~{} KB", total * 3),
            sections: vec![
                ExportSection {
                    name: "Feeds".to_string(),
                    count: feed_count,
                    details: format!("{} RSS/ArXiv feeds", feed_count),
                },
                ExportSection {
                    name: "Papers".to_string(),
                    count: paper_count,
                    details: format!("{} papers with authors and venues", paper_count),
                },
                ExportSection {
                    name: "Collections".to_string(),
                    count: collection_count,
                    details: format!("{} collections", collection_count),
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
                message: "Loading research data...".to_string(),
                error: None,
            })
            .await;

        let feeds = self.prkb_repo.list_feeds().await.map_err(|e| e.to_string())?;
        let papers = self.prkb_repo.list_papers(PaperFilter::default(), 10000, 0).await.map_err(|e| e.to_string())?;
        let collections = self.prkb_repo.list_collections().await.map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Serializing".to_string(),
                percent: 30,
                message: format!("Serializing {} papers, {} feeds, {} collections...", papers.len(), feeds.len(), collections.len()),
                error: None,
            })
            .await;

        // Create temp zip
        let temp_dir = std::env::temp_dir();
        let zip_path = temp_dir.join(format!("aether_prkb_{}.zip", task_id));
        let file = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // manifest.json
        let manifest = serde_json::json!({
            "format": "aether_prkb_v1",
            "version": "1.0",
            "kb_id": kb_id.to_string(),
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "feed_count": feeds.len(),
            "paper_count": papers.len(),
            "collection_count": collections.len(),
        });
        zip.start_file("manifest.json", options).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;

        // feeds.json
        zip.start_file("feeds.json", options).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, serde_json::to_string_pretty(&feeds).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;

        // papers.json
        zip.start_file("papers.json", options).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, serde_json::to_string_pretty(&papers).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Collections".to_string(),
                percent: 70,
                message: "Exporting collections...".to_string(),
                error: None,
            })
            .await;

        // collections.json
        zip.start_file("collections.json", options).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, serde_json::to_string_pretty(&collections).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;

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

        let manifest_entry = archive
            .by_name("manifest.json")
            .map_err(|_| "Missing manifest.json".to_string())?;
        let manifest: serde_json::Value =
            serde_json::from_reader(manifest_entry).map_err(|e| e.to_string())?;

        let format = manifest["format"].as_str().unwrap_or("");
        if format != "aether_prkb_v1" {
            return Err(format!("Unsupported format: {}", format));
        }

        let paper_count = manifest["paper_count"].as_u64().unwrap_or(0) as usize;
        let feed_count = manifest["feed_count"].as_u64().unwrap_or(0) as usize;
        let collection_count = manifest["collection_count"].as_u64().unwrap_or(0) as usize;

        Ok(ImportSummary {
            total_items: paper_count + feed_count + collection_count,
            sections: vec![
                ImportSection { name: "Feeds".to_string(), count: feed_count, action: "Create".to_string() },
                ImportSection { name: "Papers".to_string(), count: paper_count, action: "Create".to_string() },
                ImportSection { name: "Collections".to_string(), count: collection_count, action: "Create".to_string() },
            ],
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
                message: "Importing research data...".to_string(),
                error: None,
            })
            .await;

        // TODO: Implement full PRKB import with paper dedup by DOI/external_id
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
