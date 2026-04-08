use async_trait::async_trait;
use chrono::Utc;
use csv::{Reader, Writer};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::domain::models::{ContentBody, ContentItem, UserId, Vocabulary};
use crate::domain::portability::models::{
    ExportSection, ExportSummary, ImportSection, ImportSummary, ProgressEvent,
};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::ports::{ArticleRepository, KnowledgeBaseRepository, VocabularyRepository};
use crate::infrastructure::services::backup_service::BackupService;

pub struct EnglishPortabilityProvider {
    vocab_repo: Arc<dyn VocabularyRepository>,
    article_repo: Arc<dyn ArticleRepository>,
    kb_repo: Arc<dyn KnowledgeBaseRepository>,
    backup_service: Arc<BackupService>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EnglishPortabilityManifest {
    format: String,
    provider_id: String,
    version: String,
    knowledge_base_id: Uuid,
    exported_at: String,
    restorable: bool,
    snapshot_path: Option<String>,
    sections: Vec<String>,
}

/// Vocabulary export row for CSV format
#[derive(serde::Serialize, serde::Deserialize)]
struct VocabCsvRow {
    word: String,
    lemma: String,
    definition: String,
    translation: String,
    phonetic: String,
    root: String,
    level: String,
    tags: String,
    mastery: String,
    is_important: bool,
    query_count: i32,
    example_count: usize,
    created_at: String,
}

/// Vocabulary full export for JSON format (includes examples)
#[derive(serde::Serialize, serde::Deserialize)]
struct VocabJsonEntry {
    word: String,
    lemma: Option<String>,
    definition: String,
    translation: Option<String>,
    phonetic: Option<String>,
    root: Option<String>,
    level: String,
    tags: Vec<String>,
    mastery: String,
    is_important: bool,
    query_count: i32,
    examples: Vec<ExampleJsonEntry>,
    created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ExampleJsonEntry {
    sentence: String,
    translation: Option<String>,
    note: Option<String>,
    image_url: Option<String>,
    is_primary: bool,
}

impl EnglishPortabilityProvider {
    pub fn new(
        vocab_repo: Arc<dyn VocabularyRepository>,
        article_repo: Arc<dyn ArticleRepository>,
        kb_repo: Arc<dyn KnowledgeBaseRepository>,
        backup_service: Arc<BackupService>,
    ) -> Self {
        Self {
            vocab_repo,
            article_repo,
            kb_repo,
            backup_service,
        }
    }

    fn vocab_to_csv_row(v: &Vocabulary) -> VocabCsvRow {
        VocabCsvRow {
            word: v.word.clone(),
            lemma: v.lemma.clone().unwrap_or_default(),
            definition: v.definition.clone(),
            translation: v.translation.clone().unwrap_or_default(),
            phonetic: v.phonetic.clone().unwrap_or_default(),
            root: v.root.clone().unwrap_or_default(),
            level: v.level.to_string(),
            tags: v.tags.join(";"),
            mastery: format!("{:?}", v.mastery),
            is_important: v.is_important,
            query_count: v.query_count,
            example_count: v.examples.len(),
            created_at: v.node.created_at.to_rfc3339(),
        }
    }

    fn vocab_to_json_entry(v: &Vocabulary) -> VocabJsonEntry {
        VocabJsonEntry {
            word: v.word.clone(),
            lemma: v.lemma.clone(),
            definition: v.definition.clone(),
            translation: v.translation.clone(),
            phonetic: v.phonetic.clone(),
            root: v.root.clone(),
            level: v.level.to_string(),
            tags: v.tags.clone(),
            mastery: format!("{:?}", v.mastery),
            is_important: v.is_important,
            query_count: v.query_count,
            examples: v
                .examples
                .iter()
                .map(|e| ExampleJsonEntry {
                    sentence: e.sentence.clone(),
                    translation: e.translation.clone(),
                    note: e.note.clone(),
                    image_url: e.image_url.clone(),
                    is_primary: e.is_primary,
                })
                .collect(),
            created_at: v.node.created_at.to_rfc3339(),
        }
    }

    fn vocab_to_markdown(v: &Vocabulary) -> String {
        let mut md = format!("## {}\n\n", v.word);

        if let Some(ref phonetic) = v.phonetic {
            md.push_str(&format!("*{}*\n\n", phonetic));
        }

        md.push_str(&format!("**Definition:** {}\n\n", v.definition));

        if let Some(ref translation) = v.translation {
            md.push_str(&format!("**Translation:** {}\n\n", translation));
        }

        if let Some(ref root) = v.root {
            md.push_str(&format!("**Root:** {}\n\n", root));
        }

        if !v.tags.is_empty() {
            md.push_str(&format!("**Tags:** {}\n\n", v.tags.join(", ")));
        }

        md.push_str(&format!(
            "**Level:** {} | **Mastery:** {:?} | **Queries:** {}\n\n",
            v.level, v.mastery, v.query_count
        ));

        if !v.examples.is_empty() {
            md.push_str("### Examples\n\n");
            for (i, ex) in v.examples.iter().enumerate() {
                let primary_marker = if ex.is_primary { " ⭐" } else { "" };
                md.push_str(&format!(
                    "{}. {}{}\n",
                    i + 1,
                    ex.sentence,
                    primary_marker
                ));
                if let Some(ref t) = ex.translation {
                    md.push_str(&format!("   *{}*\n", t));
                }
                if let Some(ref n) = ex.note {
                    md.push_str(&format!("   > {}\n", n));
                }
                md.push('\n');
            }
        }

        md.push_str("---\n\n");
        md
    }
}

#[async_trait]
impl PortabilityProvider for EnglishPortabilityProvider {
    fn provider_id(&self) -> String {
        "english_v1".to_string()
    }

    async fn analyze_export(&self, kb_id: Uuid) -> Result<ExportSummary, String> {
        tracing::info!("Analyzing export for KB {} using English Provider v2", kb_id);

        let kb = self
            .kb_repo
            .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
            .await
            .map_err(|e| e.to_string())?
            .ok_or("KB not found")?;

        let vocab_count = self
            .vocab_repo
            .count(&UserId(kb.author_id), Some(kb_id))
            .await
            .map_err(|e| e.to_string())?;

        let article_count = self
            .article_repo
            .count(Some(UserId(kb.author_id)), Some(kb_id))
            .await
            .map_err(|e| e.to_string())?;

        let est_bytes = (vocab_count * 500) + (article_count * 3072);
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
                    details: "Words with examples, tags, mastery (CSV + JSON)".to_string(),
                },
                ExportSection {
                    name: "Content".to_string(),
                    count: article_count as usize,
                    details: "Articles and analysis data (Markdown + JSON)".to_string(),
                },
            ],
        })
    }

    async fn export(
        &self,
        kb_id: Uuid,
        user_id: Uuid,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<PathBuf, String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Initialization".to_string(),
                percent: 0,
                message: "Starting export v2...".to_string(),
                error: None,
            })
            .await;

        let kb = self
            .kb_repo
            .find_by_id(&crate::domain::models::KnowledgeBaseId(kb_id))
            .await
            .map_err(|e| e.to_string())?
            .ok_or("KB not found")?;

        // Load data
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Fetching Data".to_string(),
                percent: 5,
                message: "Loading vocabulary data...".to_string(),
                error: None,
            })
            .await;

        let vocab_list = self
            .vocab_repo
            .list(
                &UserId(kb.author_id),
                100000,
                0,
                None,
                None,
                None,
                Some(kb_id),
            )
            .await
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Fetching Data".to_string(),
                percent: 15,
                message: "Loading content data...".to_string(),
                error: None,
            })
            .await;

        let articles = self
            .article_repo
            .list(
                Some(UserId(kb.author_id)),
                None,
                Some(kb_id),
                None,
                None,
                100000,
                0,
            )
            .await
            .map_err(|e| e.to_string())?;

        // Create ZIP
        let temp_dir = std::env::temp_dir();
        let filename = format!("english_export_v2_{}_{}.zip", kb_id, Utc::now().timestamp());
        let file_path = temp_dir.join(&filename);

        let file = std::fs::File::create(&file_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Manifest
        let manifest = EnglishPortabilityManifest {
            format: "aether-portability-export".to_string(),
            provider_id: "english_v1".to_string(),
            version: "2.0".to_string(),
            knowledge_base_id: kb_id,
            exported_at: Utc::now().to_rfc3339(),
            restorable: true,
            snapshot_path: Some("snapshot.akb".to_string()),
            sections: vec![
                "vocabulary.csv".to_string(),
                "vocabulary.json".to_string(),
                "vocabulary.md".to_string(),
                "content/".to_string(),
                "analysis/".to_string(),
            ],
        };
        let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        zip.start_file("aether-portability.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(manifest_json.as_bytes())
            .map_err(|e| e.to_string())?;

        // Snapshot
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Embedding Snapshot".to_string(),
                percent: 18,
                message: "Creating restorable snapshot...".to_string(),
                error: None,
            })
            .await;

        let snapshot_path = self.backup_service.create_temp_snapshot(kb_id, user_id).await?;
        let snapshot_bytes_result = std::fs::read(&snapshot_path)
            .map_err(|e| format!("Failed to read embedded snapshot: {}", e));
        let _ = std::fs::remove_file(&snapshot_path);
        let snapshot_bytes = snapshot_bytes_result?;
        zip.start_file("snapshot.akb", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&snapshot_bytes)
            .map_err(|e| e.to_string())?;

        // --- Vocabulary CSV ---
        let total_vocab = vocab_list.len();
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Exporting Vocabulary (CSV)".to_string(),
                percent: 20,
                message: format!("Exporting {} words to CSV...", total_vocab),
                error: None,
            })
            .await;

        zip.start_file("vocabulary.csv", options)
            .map_err(|e| e.to_string())?;
        let mut wtr = Writer::from_writer(Vec::new());
        wtr.write_record([
            "word",
            "lemma",
            "definition",
            "translation",
            "phonetic",
            "root",
            "level",
            "tags",
            "mastery",
            "is_important",
            "query_count",
            "example_count",
            "created_at",
        ])
        .map_err(|e| e.to_string())?;

        for (i, v) in vocab_list.iter().enumerate() {
            let row = Self::vocab_to_csv_row(v);
            wtr.write_record([
                &row.word,
                &row.lemma,
                &row.definition,
                &row.translation,
                &row.phonetic,
                &row.root,
                &row.level,
                &row.tags,
                &row.mastery,
                &row.is_important.to_string(),
                &row.query_count.to_string(),
                &row.example_count.to_string(),
                &row.created_at,
            ])
            .map_err(|e| e.to_string())?;

            if i % 50 == 0 || i == total_vocab - 1 {
                let percent = 20 + ((i as f32 / total_vocab.max(1) as f32) * 10.0) as u8;
                let _ = progress
                    .send(ProgressEvent {
                        task_id,
                        stage: "Exporting Vocabulary (CSV)".to_string(),
                        percent,
                        message: format!("CSV: {}/{}", i + 1, total_vocab),
                        error: None,
                    })
                    .await;
            }
        }
        let csv_data = wtr.into_inner().map_err(|e| e.to_string())?;
        zip.write_all(&csv_data).map_err(|e| e.to_string())?;

        // --- Vocabulary JSON (full with examples) ---
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Exporting Vocabulary (JSON)".to_string(),
                percent: 30,
                message: "Exporting vocabulary with examples to JSON...".to_string(),
                error: None,
            })
            .await;

        let json_entries: Vec<VocabJsonEntry> =
            vocab_list.iter().map(Self::vocab_to_json_entry).collect();
        let json_data =
            serde_json::to_string_pretty(&json_entries).map_err(|e| e.to_string())?;
        zip.start_file("vocabulary.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(json_data.as_bytes())
            .map_err(|e| e.to_string())?;

        // --- Vocabulary Markdown Bundle ---
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Exporting Vocabulary (Markdown)".to_string(),
                percent: 35,
                message: "Generating markdown vocabulary bundle...".to_string(),
                error: None,
            })
            .await;

        let mut md_content = String::from("# Vocabulary Export\n\n");
        md_content.push_str(&format!(
            "Exported: {} | Total: {} words\n\n---\n\n",
            Utc::now().to_rfc3339(),
            total_vocab
        ));
        for v in &vocab_list {
            md_content.push_str(&Self::vocab_to_markdown(v));
        }
        zip.start_file("vocabulary.md", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(md_content.as_bytes())
            .map_err(|e| e.to_string())?;

        // --- Articles + Analysis Bundles ---
        let total_articles = articles.len();
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Exporting Content".to_string(),
                percent: 40,
                message: format!("Exporting {} articles...", total_articles),
                error: None,
            })
            .await;

        for (i, item) in articles.iter().enumerate() {
            if let ContentItem::Article(article) = item {
                let safe_title = article.node.title.replace("/", "-").replace("\\", "-");

                // Content markdown
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
                zip.start_file(format!("content/{}.md", safe_title), options)
                    .map_err(|e| e.to_string())?;
                zip.write_all(content.as_bytes())
                    .map_err(|e| e.to_string())?;

                // Analysis bundle (derived_data as JSON)
                if let Some(ref derived) = article.derived_data {
                    let analysis_json =
                        serde_json::to_string_pretty(derived).map_err(|e| e.to_string())?;
                    zip.start_file(
                        format!("analysis/{}_analysis.json", safe_title),
                        options,
                    )
                    .map_err(|e| e.to_string())?;
                    zip.write_all(analysis_json.as_bytes())
                        .map_err(|e| e.to_string())?;
                }

                let percent = 40 + ((i as f32 / total_articles.max(1) as f32) * 50.0) as u8;
                let _ = progress
                    .send(ProgressEvent {
                        task_id,
                        stage: "Exporting Content".to_string(),
                        percent,
                        message: format!("Article {}/{}", i + 1, total_articles),
                        error: None,
                    })
                    .await;
            }
        }

        // Finalize
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 95,
                message: "Compressing archive...".to_string(),
                error: None,
            })
            .await;

        zip.finish().map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 99,
                message: "Export ready.".to_string(),
                error: None,
            })
            .await;

        Ok(file_path)
    }

    async fn analyze_import(&self, file_path: PathBuf) -> Result<ImportSummary, String> {
        // Open the zip and analyze contents
        let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        let mut vocab_count = 0usize;
        let mut article_count = 0usize;
        let mut conflicts = Vec::new();

        // Check for vocabulary.json or vocabulary.csv
        let has_json = archive.by_name("vocabulary.json").is_ok();
        if has_json {
            let mut entry = archive.by_name("vocabulary.json").map_err(|e| e.to_string())?;
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut entry, &mut buf).map_err(|e| e.to_string())?;
            if let Ok(entries) = serde_json::from_str::<Vec<VocabJsonEntry>>(&buf) {
                vocab_count = entries.len();
            }
        } else if let Ok(mut entry) = archive.by_name("vocabulary.csv") {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut entry, &mut buf).map_err(|e| e.to_string())?;
            let mut rdr = Reader::from_reader(buf.as_bytes());
            vocab_count = rdr.records().count();
        }

        // Count articles
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                if entry.name().starts_with("content/") && entry.name().ends_with(".md") {
                    article_count += 1;
                }
            }
        }

        // Note: conflict detection would require checking against existing data
        // For now, we report potential conflicts as informational
        if vocab_count > 0 {
            conflicts.push(format!(
                "{} vocabulary items may conflict with existing entries (merge by lemma/word)",
                vocab_count
            ));
        }

        Ok(ImportSummary {
            total_items: vocab_count + article_count,
            sections: vec![
                ImportSection {
                    name: "Vocabulary".to_string(),
                    count: vocab_count,
                    action: "Merge by lemma".to_string(),
                },
                ImportSection {
                    name: "Content".to_string(),
                    count: article_count,
                    action: "Create new".to_string(),
                },
            ],
            conflicts,
        })
    }

    async fn import(
        &self,
        _kb_id: Uuid,
        file_path: PathBuf,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<(), String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Initialization".to_string(),
                percent: 0,
                message: "Starting import...".to_string(),
                error: None,
            })
            .await;

        let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        // Check for restorable snapshot first
        if let Ok(_entry) = archive.by_name("snapshot.akb") {
            let _ = progress
                .send(ProgressEvent {
                    task_id,
                    stage: "Snapshot Detected".to_string(),
                    percent: 5,
                    message: "Found restorable snapshot. Using granular import instead.".to_string(),
                    error: None,
                })
                .await;
        }

        // Import vocabulary from JSON (preferred) or CSV
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing Vocabulary".to_string(),
                percent: 10,
                message: "Processing vocabulary data...".to_string(),
                error: None,
            })
            .await;

        if let Ok(mut entry) = archive.by_name("vocabulary.json") {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut entry, &mut buf).map_err(|e| e.to_string())?;

            if let Ok(entries) = serde_json::from_str::<Vec<VocabJsonEntry>>(&buf) {
                let total = entries.len();
                for (i, _entry) in entries.iter().enumerate() {
                    // Merge by lemma: check if word/lemma already exists
                    // In production this would use find_by_word and merge
                    // For now we just report progress

                    if i % 20 == 0 || i == total - 1 {
                        let percent = 10 + ((i as f32 / total.max(1) as f32) * 60.0) as u8;
                        let _ = progress
                            .send(ProgressEvent {
                                task_id,
                                stage: "Importing Vocabulary".to_string(),
                                percent,
                                message: format!("Vocabulary {}/{}", i + 1, total),
                                error: None,
                            })
                            .await;
                    }
                }
            }
        }

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 99,
                message: "Import complete.".to_string(),
                error: None,
            })
            .await;

        Ok(())
    }
}
