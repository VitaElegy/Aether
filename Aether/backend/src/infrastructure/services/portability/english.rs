use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::io::Write;
use uuid::Uuid;
use tokio::sync::mpsc::Sender;
use chrono::Utc;
use csv::Writer;

use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::portability::models::{ExportSummary, ExportSection, ImportSummary, ProgressEvent, ImportSection};
use crate::domain::ports::{VocabularyRepository, ArticleRepository, KnowledgeBaseRepository, RepositoryError};
use crate::domain::models::{UserId, ContentItem, ContentBody, Vocabulary};

pub struct EnglishPortabilityProvider {
    vocab_repo: Arc<dyn VocabularyRepository>,
    article_repo: Arc<dyn ArticleRepository>,
    kb_repo: Arc<dyn KnowledgeBaseRepository>,
    id_override: Option<String>,
}

impl EnglishPortabilityProvider {
    pub fn new(
        vocab_repo: Arc<dyn VocabularyRepository>,
        article_repo: Arc<dyn ArticleRepository>,
        kb_repo: Arc<dyn KnowledgeBaseRepository>,
    ) -> Self {
        Self {
            vocab_repo,
            article_repo,
            kb_repo,
            id_override: None,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id_override = Some(id);
        self
    }
}

#[async_trait]
impl PortabilityProvider for EnglishPortabilityProvider {
    fn provider_id(&self) -> String {
        self.id_override.clone().unwrap_or_else(|| "english_v1".to_string())
    }

    async fn analyze_export(&self, kb_id: Uuid) -> Result<ExportSummary, String> {
        tracing::info!("Analyzing export for KB {} using English Provider", kb_id);
        
        // 1. Fetch KB to get Author ID
        let kb = self.kb_repo.find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
            .await.map_err(|e| e.to_string())?
            .ok_or("KB not found")?;

        // 2. Count Vocab (Optimized)
        let vocab_count = self.vocab_repo.count(&UserId(kb.author_id), Some(kb_id))
            .await.map_err(|e| e.to_string())?;

        // 3. Count Articles (Optimized)
        let article_count = self.article_repo.count(Some(UserId(kb.author_id)), Some(kb_id))
            .await.map_err(|e| e.to_string())?;

        // Estimate size: 
        // Vocab: ~200 bytes per word?
        // Article: ~2KB per article?
        let est_bytes = (vocab_count * 200) + (article_count * 2048);
        let est_mb = est_bytes as f64 / 1024.0 / 1024.0;
        let est_str = if est_mb < 1.0 {
            format!("{:.1} KB", est_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", est_mb)
        };

        Ok(ExportSummary {
            total_items: (vocab_count + article_count) as usize,
            estimated_size: est_str,
            sections: vec![
                ExportSection {
                    name: "Vocabulary".to_string(),
                    count: vocab_count as usize,
                    details: "Words, definitions, and examples (CSV)".to_string(),
                },
                ExportSection {
                    name: "Content".to_string(),
                    count: article_count as usize,
                    details: "Articles and Grammar Notes (Markdown)".to_string(),
                },
            ],
        })
    }

    async fn export(&self, kb_id: Uuid, task_id: Uuid, progress: Sender<ProgressEvent>) -> Result<PathBuf, String> {
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Initialization".to_string(),
            percent: 0,
            message: "Starting export...".to_string(),
            error: None,
        }).await;

        // 1. Fetch Data
        let kb = self.kb_repo.find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
            .await.map_err(|e| e.to_string())?
            .ok_or("KB not found")?;

        // 2. Load Vocabulary (Buffered or Paged?)
        // For MVP, we load all. If >100k, we might need paging.
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Fetching Data".to_string(),
            percent: 5,
            message: "Loading vocabulary data from database...".to_string(),
            error: None,
        }).await;

        let vocab_list = self.vocab_repo.list(
            &UserId(kb.author_id), 
            100000, 0, None, None, None, Some(kb_id)
        ).await.map_err(|e| e.to_string())?;

        // 3. Load Articles
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Fetching Data".to_string(),
            percent: 15,
            message: "Loading content data from database...".to_string(),
            error: None,
        }).await;

        let articles = self.article_repo.list(
            Some(UserId(kb.author_id)), None, Some(kb_id), None, None, 100000, 0
        ).await.map_err(|e| e.to_string())?;

        // 4. Initialize Zip
        let temp_dir = std::env::temp_dir();
        let filename = format!("english_export_{}_{}.zip", kb_id, Utc::now().timestamp());
        let file_path = temp_dir.join(&filename);
        
        let file = std::fs::File::create(&file_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // 5. Process Vocabulary (20% -> 50%)
        // Progress Range: 20-50 (30 points)
        let total_vocab = vocab_list.len();
        
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Exporting Vocabulary".to_string(),
            percent: 20,
            message: format!("Preparing to export {} words...", total_vocab),
            error: None,
        }).await;

        zip.start_file("vocabulary.csv", options).map_err(|e| e.to_string())?;
        let mut wtr = Writer::from_writer(Vec::new());
        wtr.write_record(&["word", "definition", "status", "tags", "created_at"]).map_err(|e| e.to_string())?;
        
        for (i, v) in vocab_list.iter().enumerate() {
            wtr.write_record(&[
                &v.word,
                &v.definition,
                &format!("{:?}", v.status),
                "", // No tags on Vocab
                &v.node.created_at.to_rfc3339()
            ]).map_err(|e| e.to_string())?;

            // Update every 100 items or 1%
            if i % 50 == 0 || i == total_vocab - 1 {
                let percent = 20 + ((i as f32 / total_vocab as f32) * 30.0) as u8;
                let _ = progress.send(ProgressEvent {
                    task_id,
                    stage: "Exporting Vocabulary".to_string(),
                    percent,
                    message: format!("Exporting word {}/{} ({})", i + 1, total_vocab, v.word),
                    error: None,
                }).await;
            }
        }
        
        let csv_data = wtr.into_inner().map_err(|e| e.to_string())?;
        zip.write_all(&csv_data).map_err(|e| e.to_string())?;

        // 6. Process Articles (50% -> 90%)
        // Progress Range: 50-90 (40 points)
        let total_articles = articles.len();
        
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Exporting Content".to_string(),
            percent: 50,
            message: format!("Preparing to export {} articles...", total_articles),
            error: None,
        }).await;

        for (i, item) in articles.iter().enumerate() {
            if let ContentItem::Article(article) = item {
                let safe_title = article.node.title.replace("/", "-").replace("\\", "-");
                let entry_path = format!("content/{}.md", safe_title);
                
                zip.start_file(&entry_path, options).map_err(|e| e.to_string())?;
                
                let body = match &article.body {
                    ContentBody::Markdown(s) => s.clone(),
                    _ => String::new(),
                };
                
                let content = format!(
                    "---\ntitle: {}\ntags: [{}]\ncreated: {}\n---\n\n{}",
                    article.node.title,
                    article.tags.join(", "),
                    article.node.created_at.to_rfc3339(),
                    body
                );
                
                zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
                
                // Update every item (articles are heavier)
                let percent = 50 + ((i as f32 / total_articles as f32) * 40.0) as u8;
                let _ = progress.send(ProgressEvent {
                    task_id,
                    stage: "Exporting Content".to_string(),
                    percent,
                    message: format!("Exporting article {}/{} ({})", i + 1, total_articles, article.node.title),
                    error: None,
                }).await;
            } else {
                 // Skip non-article nodes or handle folders
            }
        }

        // 7. Finalizing
        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Finalizing".to_string(),
            percent: 95,
            message: "Compressing archive...".to_string(),
            error: None,
        }).await;

        zip.finish().map_err(|e| e.to_string())?;

        let _ = progress.send(ProgressEvent {
            task_id,
            stage: "Completed".to_string(),
            percent: 100,
            message: "Export ready for download.".to_string(),
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
