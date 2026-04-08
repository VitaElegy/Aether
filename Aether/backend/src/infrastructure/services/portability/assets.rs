use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::domain::models::{
    Article, ContentBody, ContentItem, ContentStatus, Node, NodeType, PermissionMode, UserId,
};
use crate::domain::portability::models::{
    ExportSection, ExportSummary, ImportSection, ImportSummary, ProgressEvent,
};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::ports::{ArticleRepository, KnowledgeBaseRepository, NodeRepository};

/// Manifest structure for the assets export ZIP
#[derive(serde::Serialize, serde::Deserialize)]
struct AssetsExportManifest {
    format: String,
    provider_id: String,
    version: String,
    knowledge_base_id: Uuid,
    exported_at: String,
    item_count: usize,
}

/// Metadata entry for a single asset in the export
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct AssetMetadataEntry {
    id: Uuid,
    title: String,
    asset_type: String,
    file_path: String,
    original_filename: String,
    mime_type: String,
    hash: String,
    size_bytes: u64,
    author_id: Uuid,
    permission_mode: String,
    created_at: String,
    updated_at: String,
}

/// Usage edge: which articles reference this asset
#[derive(serde::Serialize, serde::Deserialize)]
struct UsageEdge {
    asset_id: Uuid,
    referenced_by: Vec<UsageReference>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UsageReference {
    article_id: Uuid,
    title: String,
    reference_type: String,
}

/// Permission hint for each asset
#[derive(serde::Serialize, serde::Deserialize)]
struct PermissionHint {
    asset_id: Uuid,
    author_id: Uuid,
    permission_mode: String,
}

pub struct AssetsPortabilityProvider {
    article_repo: Arc<dyn ArticleRepository>,
    _kb_repo: Arc<dyn KnowledgeBaseRepository>,
    _node_repo: Arc<dyn NodeRepository>,
    storage_root: PathBuf,
}

impl AssetsPortabilityProvider {
    pub fn new(
        article_repo: Arc<dyn ArticleRepository>,
        kb_repo: Arc<dyn KnowledgeBaseRepository>,
        node_repo: Arc<dyn NodeRepository>,
        storage_root: String,
    ) -> Self {
        Self {
            article_repo,
            _kb_repo: kb_repo,
            _node_repo: node_repo,
            storage_root: PathBuf::from(storage_root),
        }
    }

    fn extract_asset_metadata(article: &Article) -> Option<AssetMetadataEntry> {
        let payload = match &article.body {
            ContentBody::Custom(v) => v.clone(),
            _ => return None,
        };

        let asset_type = payload
            .get("asset_type")
            .and_then(|v| v.as_str())
            .unwrap_or("file_asset")
            .to_string();
        let file_path = payload
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let original_filename = payload
            .get("original_filename")
            .and_then(|v| v.as_str())
            .unwrap_or(&article.node.title)
            .to_string();
        let mime_type = payload
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream")
            .to_string();
        let hash = payload
            .get("hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let size_bytes = payload
            .get("size_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Some(AssetMetadataEntry {
            id: article.node.id,
            title: article.node.title.clone(),
            asset_type,
            file_path,
            original_filename,
            mime_type,
            hash,
            size_bytes,
            author_id: article.node.author_id,
            permission_mode: format!("{:?}", article.node.permission_mode),
            created_at: article.node.created_at.to_rfc3339(),
            updated_at: article.node.updated_at.to_rfc3339(),
        })
    }
}

#[async_trait]
impl PortabilityProvider for AssetsPortabilityProvider {
    fn provider_id(&self) -> String {
        "assets_v1".to_string()
    }

    async fn analyze_export(&self, kb_id: Uuid) -> Result<ExportSummary, String> {
        let articles = self
            .article_repo
            .list(
                None,               // viewer_id
                None,               // author_id
                Some(kb_id),        // knowledge_base_id
                None,               // tag
                Some("Asset".to_string()), // category
                1000,               // limit
                0,                  // offset
            )
            .await
            .map_err(|e| format!("Failed to list assets: {}", e))?;

        let mut total_items = 0usize;
        let mut total_binary_size = 0u64;
        let mut image_count = 0usize;
        let mut pdf_count = 0usize;
        let mut file_count = 0usize;
        let mut structured_count = 0usize;

        for item in &articles {
            if let ContentItem::Article(article) = item {
                if let Some(meta) = Self::extract_asset_metadata(article) {
                    total_items += 1;
                    total_binary_size += meta.size_bytes;
                    match meta.asset_type.as_str() {
                        "image_asset" => image_count += 1,
                        "pdf_asset" => pdf_count += 1,
                        "ip_asset" | "domain_asset" | "credential_stub" | "snippet_asset" => {
                            structured_count += 1
                        }
                        _ => file_count += 1,
                    }
                }
            }
        }

        let estimated_size = if total_binary_size > 1024 * 1024 * 1024 {
            format!("{:.1} GB", total_binary_size as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if total_binary_size > 1024 * 1024 {
            format!("{:.1} MB", total_binary_size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} KB", total_binary_size as f64 / 1024.0)
        };

        let binary_details = format!("Total binary data: {}", estimated_size);

        Ok(ExportSummary {
            total_items,
            estimated_size,
            sections: vec![
                ExportSection {
                    name: "Metadata".to_string(),
                    count: total_items,
                    details: format!(
                        "{} images, {} PDFs, {} files, {} structured",
                        image_count, pdf_count, file_count, structured_count
                    ),
                },
                ExportSection {
                    name: "Binaries".to_string(),
                    count: total_items,
                    details: binary_details,
                },
                ExportSection {
                    name: "Usage Edges".to_string(),
                    count: total_items,
                    details: "Cross-references from other knowledge bases".to_string(),
                },
                ExportSection {
                    name: "Permission Hints".to_string(),
                    count: total_items,
                    details: "Author and permission metadata for each asset".to_string(),
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
                message: "Collecting asset metadata...".to_string(),
                error: None,
            })
            .await;

        // 1. Collect all assets
        let articles = self
            .article_repo
            .list(
                None,               // viewer_id
                None,               // author_id
                Some(kb_id),        // knowledge_base_id
                None,               // tag
                Some("Asset".to_string()), // category
                1000,               // limit
                0,                  // offset
            )
            .await
            .map_err(|e| format!("Failed to list assets: {}", e))?;

        let mut metadata_entries: Vec<AssetMetadataEntry> = Vec::new();
        let mut permission_hints: Vec<PermissionHint> = Vec::new();

        for item in &articles {
            if let ContentItem::Article(article) = item {
                if let Some(meta) = Self::extract_asset_metadata(article) {
                    permission_hints.push(PermissionHint {
                        asset_id: meta.id,
                        author_id: meta.author_id,
                        permission_mode: meta.permission_mode.clone(),
                    });
                    metadata_entries.push(meta);
                }
            }
        }

        let total_assets = metadata_entries.len();

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Metadata".to_string(),
                percent: 10,
                message: format!("Found {} assets. Building archive...", total_assets),
                error: None,
            })
            .await;

        // 2. Create ZIP archive
        let temp_dir = std::env::temp_dir();
        let zip_filename = format!("assets_export_{}.zip", Uuid::new_v4());
        let zip_path = temp_dir.join(&zip_filename);

        let file = std::fs::File::create(&zip_path)
            .map_err(|e| format!("Failed to create ZIP: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // 3. Write manifest
        let manifest = AssetsExportManifest {
            format: "aether_assets_v1".to_string(),
            provider_id: self.provider_id(),
            version: "1.0.0".to_string(),
            knowledge_base_id: kb_id,
            exported_at: Utc::now().to_rfc3339(),
            item_count: total_assets,
        };
        let manifest_json =
            serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        zip.start_file("manifest.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(manifest_json.as_bytes())
            .map_err(|e| e.to_string())?;

        // 4. Write metadata
        let metadata_json =
            serde_json::to_string_pretty(&metadata_entries).map_err(|e| e.to_string())?;
        zip.start_file("metadata.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(metadata_json.as_bytes())
            .map_err(|e| e.to_string())?;

        // 5. Write binaries
        for (idx, entry) in metadata_entries.iter().enumerate() {
            let binary_path = self.storage_root.join(&entry.file_path);
            if binary_path.exists() {
                let mut buf = Vec::new();
                if let Ok(mut f) = std::fs::File::open(&binary_path) {
                    if f.read_to_end(&mut buf).is_ok() {
                        let archive_path = format!("binaries/{}", entry.hash);
                        zip.start_file(&archive_path, options)
                            .map_err(|e| e.to_string())?;
                        zip.write_all(&buf).map_err(|e| e.to_string())?;
                    }
                }
            }

            // Report progress (10% to 80% for binaries)
            if total_assets > 0 {
                let pct = 10 + ((idx + 1) * 70 / total_assets) as u8;
                let _ = progress
                    .send(ProgressEvent {
                        task_id,
                        stage: "Binaries".to_string(),
                        percent: pct,
                        message: format!("Archiving binary {}/{}...", idx + 1, total_assets),
                        error: None,
                    })
                    .await;
            }
        }

        // 6. Write usage edges — cross-KB reference search
        let mut usage_edges: Vec<UsageEdge> = Vec::new();
        for entry in &metadata_entries {
            let asset_marker = format!("[[asset:{}]]", entry.id);
            let embed_marker = format!("([[asset:{}]])", entry.id);

            // Search all non-asset articles for references to this asset
            let all_articles = self
                .article_repo
                .list(
                    None, // viewer_id
                    None, // author_id
                    None, // knowledge_base_id — search ALL KBs
                    None, // tag
                    None, // category — search all categories
                    2000, // limit
                    0,    // offset
                )
                .await
                .unwrap_or_default();

            let mut refs: Vec<UsageReference> = Vec::new();
            for item in &all_articles {
                if let ContentItem::Article(article) = item {
                    // Skip the asset itself and other assets
                    if article.node.id == entry.id
                        || article.category.as_deref() == Some("Asset")
                    {
                        continue;
                    }

                    let body_text = match &article.body {
                        ContentBody::Markdown(text) => text.clone(),
                        ContentBody::Custom(val) => val.to_string(),
                        _ => String::new(),
                    };

                    if body_text.is_empty() {
                        continue;
                    }

                    let reference_type = if body_text.contains(&embed_marker) {
                        "embed"
                    } else if body_text.contains(&asset_marker) {
                        "reference"
                    } else {
                        continue;
                    };

                    refs.push(UsageReference {
                        article_id: article.node.id,
                        title: article.node.title.clone(),
                        reference_type: reference_type.to_string(),
                    });
                }
            }

            usage_edges.push(UsageEdge {
                asset_id: entry.id,
                referenced_by: refs,
            });
        }
        let usage_json =
            serde_json::to_string_pretty(&usage_edges).map_err(|e| e.to_string())?;
        zip.start_file("usage_edges.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(usage_json.as_bytes())
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Usage Edges".to_string(),
                percent: 90,
                message: "Writing usage edges...".to_string(),
                error: None,
            })
            .await;

        // 7. Write permission hints
        let perm_json =
            serde_json::to_string_pretty(&permission_hints).map_err(|e| e.to_string())?;
        zip.start_file("permission_hints.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(perm_json.as_bytes())
            .map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 99,
                message: "Export archive ready.".to_string(),
                error: None,
            })
            .await;

        Ok(zip_path)
    }

    async fn analyze_import(&self, file_path: PathBuf) -> Result<ImportSummary, String> {
        let file = std::fs::File::open(&file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

        // Read manifest
        let manifest: AssetsExportManifest = {
            let mut entry = archive
                .by_name("manifest.json")
                .map_err(|_| "Missing manifest.json in archive".to_string())?;
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| e.to_string())?;
            serde_json::from_str(&buf).map_err(|e| format!("Invalid manifest: {}", e))?
        };

        if manifest.format != "aether_assets_v1" {
            return Err(format!(
                "Unsupported export format: {}",
                manifest.format
            ));
        }

        // Read metadata
        let metadata_entries: Vec<AssetMetadataEntry> = {
            let mut entry = archive
                .by_name("metadata.json")
                .map_err(|_| "Missing metadata.json in archive".to_string())?;
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| e.to_string())?;
            serde_json::from_str(&buf).map_err(|e| format!("Invalid metadata: {}", e))?
        };

        // Check for hash conflicts with existing assets
        let mut conflicts = Vec::new();
        let mut new_count = 0usize;
        let mut skip_count = 0usize;

        for entry in &metadata_entries {
            let sharded_path = self
                .storage_root
                .join("uploads")
                .join(&entry.hash[0..2.min(entry.hash.len())])
                .join(&entry.hash);

            if sharded_path.exists() {
                conflicts.push(format!(
                    "Asset '{}' (hash={}) already exists — will skip binary",
                    entry.original_filename,
                    &entry.hash[..16.min(entry.hash.len())]
                ));
                skip_count += 1;
            } else {
                new_count += 1;
            }
        }

        Ok(ImportSummary {
            total_items: metadata_entries.len(),
            sections: vec![
                ImportSection {
                    name: "New Assets".to_string(),
                    count: new_count,
                    action: "Create".to_string(),
                },
                ImportSection {
                    name: "Existing (Skip Binary)".to_string(),
                    count: skip_count,
                    action: "Skip".to_string(),
                },
            ],
            conflicts,
        })
    }

    async fn import(
        &self,
        kb_id: Uuid,
        file_path: PathBuf,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<(), String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Preparing".to_string(),
                percent: 0,
                message: "Opening archive...".to_string(),
                error: None,
            })
            .await;

        let file = std::fs::File::open(&file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

        // Read metadata
        let metadata_entries: Vec<AssetMetadataEntry> = {
            let mut entry = archive
                .by_name("metadata.json")
                .map_err(|_| "Missing metadata.json".to_string())?;
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| e.to_string())?;
            serde_json::from_str(&buf).map_err(|e| format!("Invalid metadata: {}", e))?
        };

        let total = metadata_entries.len();

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing".to_string(),
                percent: 10,
                message: format!("Importing {} assets...", total),
                error: None,
            })
            .await;

        for (idx, entry) in metadata_entries.iter().enumerate() {
            // Write binary to disk if not already present
            let shard = if entry.hash.len() >= 2 {
                &entry.hash[0..2]
            } else {
                "00"
            };
            let sharded_dir = self.storage_root.join("uploads").join(shard);
            let binary_path = sharded_dir.join(&entry.hash);

            if !binary_path.exists() {
                let archive_path = format!("binaries/{}", entry.hash);
                if let Ok(mut zip_entry) = archive.by_name(&archive_path) {
                    tokio::fs::create_dir_all(&sharded_dir)
                        .await
                        .map_err(|e| e.to_string())?;

                    let mut buf = Vec::new();
                    zip_entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;

                    tokio::fs::write(&binary_path, &buf)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }

            // Create Article node
            let relative_path = format!("uploads/{}/{}", shard, entry.hash);
            let payload = json!({
                "version": 2,
                "asset_type": entry.asset_type,
                "display_name": entry.original_filename,
                "file_path": relative_path,
                "original_filename": entry.original_filename,
                "mime_type": entry.mime_type,
                "hash": entry.hash,
                "size_bytes": entry.size_bytes,
                "metadata": {
                    "imported": true,
                    "import_source": "portability_v1",
                }
            });

            let article = Article {
                node: Node {
                    id: Uuid::new_v4(), // Generate new IDs for imported assets
                    parent_id: None,
                    author_id: entry.author_id,
                    knowledge_base_id: Some(kb_id),
                    r#type: NodeType::Article,
                    title: entry.title.clone(),
                    permission_mode: PermissionMode::Private,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                slug: entry.hash.clone(),
                status: ContentStatus::Published,
                category: Some("Asset".to_string()),
                body: ContentBody::Custom(payload),
                tags: vec![entry.mime_type.clone(), entry.asset_type.clone()],
                author_name: None,
                author_avatar: None,
                derived_data: None,
                analysis_status: None,
                analysis_diagnostics: None,
            };

            self.article_repo
                .save(article, UserId(entry.author_id), None)
                .await
                .map_err(|e| format!("Failed to save asset: {}", e))?;

            // Report progress
            if total > 0 {
                let pct = 10 + ((idx + 1) * 85 / total) as u8;
                let _ = progress
                    .send(ProgressEvent {
                        task_id,
                        stage: "Importing".to_string(),
                        percent: pct,
                        message: format!("Imported {}/{}...", idx + 1, total),
                        error: None,
                    })
                    .await;
            }
        }

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Complete".to_string(),
                percent: 99,
                message: format!("Successfully imported {} assets.", total),
                error: None,
            })
            .await;

        Ok(())
    }
}
