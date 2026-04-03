use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter}; // For file reading

use crate::domain::models::{
    Article, ContentBody, ContentItem, ContentStatus, KnowledgeBase, KnowledgeBaseId, Node,
    NodeType, PermissionMode, UserId, Visibility,
};
use crate::domain::ports::{ArticleRepository, KnowledgeBaseRepository, NodeRepository};
use crate::infrastructure::services::asset_manager::AssetManager;

// --- Meta Schema ---

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupMeta {
    pub version: String,
    pub exported_at: String,
    pub knowledge_base: BackupKbMeta,
    pub nodes: Vec<BackupNodeMeta>,
    pub assets_map: HashMap<Uuid, String>, // Asset UUID -> Zip Path (assets/hash.png)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupKbMeta {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub renderer_id: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupNodeMeta {
    id: Uuid,
    parent_id: Option<Uuid>,
    title: String,
    slug: String,
    r#type: String, // "Article", "Folder"
    tags: Vec<String>,
    status: String,
    created_at: String,
    updated_at: String,
    // We might store extra props here
    path: String, // Human readable path in zip: "Folder/My Article.md"
}

// --- Service ---

#[derive(Clone)]
pub struct BackupService {
    article_repo: Arc<dyn ArticleRepository>,
    kb_repo: Arc<dyn KnowledgeBaseRepository>,
    node_repo: Arc<dyn NodeRepository>,
    asset_manager: Arc<AssetManager>,
    backup_root: PathBuf,
}

#[derive(Deserialize)]
struct PortabilityExportManifest {
    snapshot_path: Option<String>,
    restorable: Option<bool>,
}

#[derive(Debug)]
struct ResolvedBackupArchive {
    path: PathBuf,
    cleanup_path: Option<PathBuf>,
}

impl ResolvedBackupArchive {
    fn direct(path: PathBuf) -> Self {
        Self {
            path,
            cleanup_path: None,
        }
    }

    fn extracted(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            cleanup_path: Some(path),
        }
    }

    fn cleanup(&self) {
        if let Some(path) = &self.cleanup_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn detect_wrong_archive_type(entry_names: &[String]) -> Option<String> {
    if entry_names.iter().any(|name| name == "aether-portability.json") {
        return Some(
            "Portability export detected: this archive does not contain an embedded snapshot.akb and cannot be restored directly".to_string(),
        );
    }

    let has_vocab_csv = entry_names.iter().any(|name| name == "vocabulary.csv");
    let has_content_exports = entry_names.iter().any(|name| name.starts_with("content/"));
    let has_only_export_like_files = !entry_names.is_empty()
        && entry_names
            .iter()
            .all(|name| name == "vocabulary.csv" || name.starts_with("content/"));

    if has_vocab_csv || (has_content_exports && has_only_export_like_files) {
        return Some(
            "Portability export detected: expected backup meta.json, but the archive contains Smart Portability export files instead".to_string(),
        );
    }

    None
}

fn read_portability_manifest(
    archive: &mut ZipArchive<std::fs::File>,
) -> Option<PortabilityExportManifest> {
    let mut manifest_file = archive.by_name("aether-portability.json").ok()?;
    let mut manifest_content = String::new();
    manifest_file.read_to_string(&mut manifest_content).ok()?;
    drop(manifest_file);
    serde_json::from_str(&manifest_content).ok()
}

fn resolve_backup_archive_path(file_path: &PathBuf) -> Result<ResolvedBackupArchive, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open uploaded backup: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Invalid backup archive: {}", e))?;

    let entry_names = archive.file_names().map(|name| name.to_string()).collect::<Vec<_>>();
    if entry_names.iter().any(|name| name == "meta.json") {
        return Ok(ResolvedBackupArchive::direct(file_path.clone()));
    }

    let manifest = read_portability_manifest(&mut archive);
    let snapshot_path = manifest
        .as_ref()
        .and_then(|manifest| manifest.snapshot_path.clone())
        .or_else(|| {
            entry_names
                .iter()
                .find(|name| name.as_str() == "snapshot.akb")
                .cloned()
        });

    if let Some(snapshot_path) = snapshot_path {
        let mut snapshot_entry = archive.by_name(&snapshot_path).map_err(|_| {
            format!(
                "Portability export detected: embedded snapshot file '{}' is missing",
                snapshot_path
            )
        })?;
        let temp_path = std::env::temp_dir()
            .join(format!("embedded_snapshot_{}_{}.akb", Uuid::new_v4(), Uuid::new_v4()));
        let mut temp_file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temporary snapshot file: {}", e))?;
        std::io::copy(&mut snapshot_entry, &mut temp_file)
            .map_err(|e| format!("Failed to extract embedded snapshot: {}", e))?;
        return Ok(ResolvedBackupArchive::extracted(temp_path));
    }

    if manifest
        .as_ref()
        .and_then(|manifest| manifest.restorable)
        .unwrap_or(false)
    {
        return Err(
            "Portability export detected: embedded snapshot is declared but missing from the archive"
                .to_string(),
        );
    }

    if let Some(message) = detect_wrong_archive_type(&entry_names) {
        return Err(message);
    }

    Ok(ResolvedBackupArchive::direct(file_path.clone()))
}

fn read_backup_meta(archive: &mut ZipArchive<std::fs::File>) -> Result<BackupMeta, String> {
    let entry_names = archive.file_names().map(|name| name.to_string()).collect::<Vec<_>>();

    let mut meta_file = archive.by_name("meta.json").map_err(|_| {
        detect_wrong_archive_type(&entry_names)
            .unwrap_or_else(|| "Invalid backup: missing meta.json".to_string())
    })?;
    let mut meta_content = String::new();
    meta_file
        .read_to_string(&mut meta_content)
        .map_err(|e| format!("Failed to read meta.json: {}", e))?;
    drop(meta_file);

    serde_json::from_str(&meta_content).map_err(|e| format!("Invalid meta.json: {}", e))
}

impl BackupService {
    pub fn new(
        article_repo: Arc<dyn ArticleRepository>,
        kb_repo: Arc<dyn KnowledgeBaseRepository>,
        node_repo: Arc<dyn NodeRepository>,
        asset_manager: Arc<AssetManager>,
        storage_root: String,
    ) -> Self {
        let backup_root = PathBuf::from(storage_root).join("backups");
        std::fs::create_dir_all(&backup_root).unwrap_or_default(); // Ensure dir exists

        Self {
            article_repo,
            kb_repo,
            node_repo,
            asset_manager,
            backup_root,
        }
    }

    pub async fn create_backup(&self, kb_id: Uuid, user_id: Uuid) -> Result<String, String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{}_{}.akb", kb_id, timestamp);
        let file_path = self.backup_root.join(&filename);

        self.write_backup_archive(kb_id, user_id, &file_path).await?;

        Ok(filename)
    }

    pub async fn create_temp_snapshot(&self, kb_id: Uuid, user_id: Uuid) -> Result<PathBuf, String> {
        let file_path = std::env::temp_dir()
            .join(format!("snapshot_{}_{}.akb", kb_id, Uuid::new_v4()));
        self.write_backup_archive(kb_id, user_id, &file_path).await?;
        Ok(file_path)
    }

    async fn write_backup_archive(
        &self,
        kb_id: Uuid,
        user_id: Uuid,
        file_path: &PathBuf,
    ) -> Result<(), String> {
        // 1. Fetch KB
        let kb = self
            .kb_repo
            .find_by_id(&KnowledgeBaseId(kb_id))
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Knowledge Base not found")?;

        if kb.author_id != user_id {
            return Err("Unauthorized".to_string());
        }

        // 2. Fetch All Nodes/Articles
        // We use list with large limit. Pagination might be needed for huge KBs, but for V1 we assume <10k items.
        let items = self
            .article_repo
            .list(
                Some(UserId(user_id)),
                None,
                Some(kb_id),
                None,
                None,
                10000,
                0,
            )
            .await
            .map_err(|e| e.to_string())?;

        // 3. Prepare ZIP
        let file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<'_, ()> = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // 4. Analyze Assets & Build Folder Tree
        let mut nodes_meta = Vec::new();
        let mut assets_to_include = HashSet::new(); // Set of Asset UUIDs
        let asset_regex = Regex::new(r"\[\[asset:([0-9a-fA-F-]+)\]\]").unwrap();

        // Build simple ID -> Title/Parent map for path generation
        let mut node_map: HashMap<Uuid, (Option<Uuid>, String)> = HashMap::new();
        for item in &items {
            let n = match item {
                ContentItem::Article(a) => &a.node,
                ContentItem::Node(n) => n,
            };
            node_map.insert(n.id, (n.parent_id, n.title.clone()));
        }

        // Process Items
        for item in &items {
            let (node, body_str) = match item {
                ContentItem::Article(a) => {
                    let s = match &a.body {
                        ContentBody::Markdown(t) => t.clone(),
                        ContentBody::CodeSnippet { code, .. } => code.clone(),
                        _ => String::new(),
                    };
                    (&a.node, s)
                }
                ContentItem::Node(n) => (n, String::new()),
            };

            // Detect Assets
            for cap in asset_regex.captures_iter(&body_str) {
                if let Some(m) = cap.get(1) {
                    if let Ok(uuid) = Uuid::parse_str(m.as_str()) {
                        assets_to_include.insert(uuid);
                    }
                }
            }

            // Generate Path
            let path_str = self.build_path(node.id, &node_map);
            let full_entry_path = if node.r#type == NodeType::Folder {
                format!("content/{}/", path_str)
            } else {
                format!("content/{}.md", path_str)
            };

            // Write Content to Zip
            if node.r#type == NodeType::Article {
                zip.start_file(&full_entry_path, options)
                    .map_err(|e| e.to_string())?;
                zip.write_all(body_str.as_bytes())
                    .map_err(|e| e.to_string())?;
            } else {
                zip.add_directory(&full_entry_path, options)
                    .map_err(|e| e.to_string())?;
            }

            // Add to Meta
            nodes_meta.push(BackupNodeMeta {
                id: node.id,
                parent_id: node.parent_id,
                title: node.title.clone(),
                slug: match item {
                    ContentItem::Article(a) => a.slug.clone(),
                    _ => "".to_string(),
                },
                r#type: format!("{:?}", node.r#type),
                tags: match item {
                    ContentItem::Article(a) => a.tags.clone(),
                    _ => vec![],
                },
                status: format!(
                    "{:?}",
                    match item {
                        ContentItem::Article(a) => a.status,
                        _ => ContentStatus::Draft,
                    }
                ),
                created_at: node.created_at.to_rfc3339(),
                updated_at: node.updated_at.to_rfc3339(),
                path: full_entry_path,
            });
        }

        // 5. Process Assets
        let mut assets_map = HashMap::new();
        for asset_id in assets_to_include {
            match self
                .asset_manager
                .get_asset_file(asset_id, None, user_id)
                .await
            {
                Ok((path, _mime)) => {
                    let entry_name = format!("assets/{}", asset_id); // Flattened assets
                                                                     // Read file
                    if let Ok(mut asset_file) = tokio::fs::File::open(&path).await {
                        let mut buffer = Vec::new();
                        if asset_file.read_to_end(&mut buffer).await.is_ok() {
                            zip.start_file(&entry_name, options)
                                .map_err(|e| e.to_string())?;
                            zip.write_all(&buffer).map_err(|e| e.to_string())?;
                            assets_map.insert(asset_id, entry_name);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Skipping asset {} in backup: {}", asset_id, e);
                }
            }
        }

        // 6. Write Meta.json
        let meta = BackupMeta {
            version: "1.0".to_string(),
            exported_at: Utc::now().to_rfc3339(),
            knowledge_base: BackupKbMeta {
                id: kb.id.0,
                title: kb.title,
                description: kb.description,
                renderer_id: kb.renderer_id,
                tags: kb.tags,
            },
            nodes: nodes_meta,
            assets_map,
        };

        let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
        zip.start_file("meta.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(meta_json.as_bytes())
            .map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;

        Ok(())
    }

    // Helper to recursive build path
    fn build_path(&self, node_id: Uuid, map: &HashMap<Uuid, (Option<Uuid>, String)>) -> String {
        let mut current = node_id;
        let mut parts = Vec::new();

        // Safety valve for loops
        let mut visited = HashSet::new();

        while let Some((parent, title)) = map.get(&current) {
            if visited.contains(&current) {
                break;
            }
            visited.insert(current);

            // Sanitize title for filesystem
            let safe_title = title.replace("/", "-").replace("\\", "-");
            parts.push(safe_title);

            if let Some(p) = parent {
                current = *p;
            } else {
                break;
            }
        }

        parts.reverse();
        parts.join("/")
    }

    pub async fn list_backups(&self) -> Result<Vec<String>, String> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.backup_root)
            .await
            .map_err(|e| e.to_string())?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".akb") {
                    files.push(name);
                }
            }
        }
        // Sort newest first
        files.sort_by(|a, b| b.cmp(a));
        Ok(files)
    }

    pub fn preview_backup(
        &self,
        file_path: &PathBuf,
    ) -> Result<crate::domain::portability::models::ImportSummary, String> {
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| format!("Failed to inspect uploaded backup: {}", e))?;
        if metadata.len() == 0 {
            return Err("Uploaded backup file is empty".to_string());
        }

        let resolved = resolve_backup_archive_path(file_path)?;
        let preview_result = (|| {
            let file = std::fs::File::open(&resolved.path)
                .map_err(|e| format!("Failed to open uploaded backup: {}", e))?;
            let mut archive =
                ZipArchive::new(file).map_err(|e| format!("Invalid backup archive: {}", e))?;

            let meta = read_backup_meta(&mut archive)?;

            let mut sections = vec![
                crate::domain::portability::models::ImportSection {
                    name: "Knowledge Base Header".to_string(),
                    count: 1,
                    action: "Create".to_string(),
                },
                crate::domain::portability::models::ImportSection {
                    name: "Articles/Folders".to_string(),
                    count: meta.nodes.len(),
                    action: "Create".to_string(),
                },
            ];

            if !meta.assets_map.is_empty() {
                sections.push(crate::domain::portability::models::ImportSection {
                    name: "Media Assets".to_string(),
                    count: meta.assets_map.len(),
                    action: "Create".to_string(),
                });
            }

            Ok(crate::domain::portability::models::ImportSummary {
                total_items: meta.nodes.len() + meta.assets_map.len() + 1,
                sections,
                conflicts: vec![],
            })
        })();
        resolved.cleanup();
        preview_result
    }

    pub async fn restore_backup(
        &self,
        file_path: PathBuf,
        user_id: Uuid,
        task_id: Option<Uuid>,
        progress: Option<
            tokio::sync::mpsc::Sender<crate::domain::portability::models::ProgressEvent>,
        >,
    ) -> Result<Uuid, String> {
        let resolved = resolve_backup_archive_path(&file_path)?;
        let resolved_path = resolved.path.clone();
        let restore_result = async {
            // 1. Open Zip
            let file = std::fs::File::open(&resolved_path).map_err(|e| e.to_string())?;
            let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

            // 2. Read Meta.json
            let meta = read_backup_meta(&mut archive)?;

            if let (Some(tid), Some(tx)) = (task_id, &progress) {
                let _ = tx
                    .send(crate::domain::portability::models::ProgressEvent {
                        task_id: tid,
                        stage: "Restoring".to_string(),
                        percent: 10,
                        message: "Creating new Knowledge Base...".to_string(),
                        error: None,
                    })
                    .await;
            }

            // 3. Create NEW Knowledge Base
            let new_kb_id = Uuid::new_v4();
            let restored_title = format!(
                "{} (Restored {})",
                meta.knowledge_base.title,
                Utc::now().format("%Y-%m-%d %H:%M")
            );

            let new_kb = KnowledgeBase {
                id: KnowledgeBaseId(new_kb_id),
                author_id: user_id,
                title: restored_title,
                description: meta.knowledge_base.description,
                tags: meta.knowledge_base.tags,
                renderer_id: meta.knowledge_base.renderer_id,
                visibility: Visibility::Private,
                cover_image: None,
                cover_offset_y: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.kb_repo.save(new_kb).await.map_err(|e| e.to_string())?;

            let mut node_id_map: HashMap<Uuid, Uuid> = HashMap::new();
            let mut asset_id_map: HashMap<Uuid, Uuid> = HashMap::new();

            let total_assets = meta.assets_map.len();
            for (i, (old_asset_uuid, zip_path)) in meta.assets_map.into_iter().enumerate() {
                if let (Some(tid), Some(tx)) = (task_id, &progress) {
                    let percent = 10 + ((i as f32 / total_assets as f32) * 30.0) as u8;
                    let _ = tx
                        .send(crate::domain::portability::models::ProgressEvent {
                            task_id: tid,
                            stage: "Restoring Assets".to_string(),
                            percent,
                            message: format!("Importing media {}/{}", i + 1, total_assets),
                            error: None,
                        })
                        .await;
                }
                let mut asset_file = archive
                    .by_name(&zip_path)
                    .map_err(|_| format!("Asset missing: {}", zip_path))?;
                let mut buffer = Vec::new();
                asset_file
                    .read_to_end(&mut buffer)
                    .map_err(|e| e.to_string())?;
                drop(asset_file);

                let mime = "application/octet-stream".to_string();

                match self
                    .asset_manager
                    .upload_asset(user_id, "restored_asset".to_string(), mime, &buffer)
                    .await
                {
                    Ok(new_asset_node) => {
                        asset_id_map.insert(old_asset_uuid, new_asset_node.node.id);
                    }
                    Err(e) => tracing::error!("Failed to restore asset {}: {}", old_asset_uuid, e),
                }
            }

            for node_meta in &meta.nodes {
                node_id_map.insert(node_meta.id, Uuid::new_v4());
            }

            let mut sorted_nodes = meta.nodes.clone();
            sorted_nodes.sort_by_key(|n| n.path.matches('/').count());

            let asset_regex = Regex::new(r"\[\[asset:([0-9a-fA-F-]+)\]\]").unwrap();

            let total_nodes = sorted_nodes.len();
            for (i, node_meta) in sorted_nodes.into_iter().enumerate() {
                if let (Some(tid), Some(tx)) = (task_id, &progress) {
                    let percent = 40 + ((i as f32 / total_nodes as f32) * 55.0) as u8;
                    let _ = tx
                        .send(crate::domain::portability::models::ProgressEvent {
                            task_id: tid,
                            stage: "Restoring Content".to_string(),
                            percent,
                            message: format!("Importing document {}/{}", i + 1, total_nodes),
                            error: None,
                        })
                        .await;
                }
                let new_id = *node_id_map.get(&node_meta.id).unwrap();

                let new_parent_id = node_meta
                    .parent_id
                    .and_then(|pid| node_id_map.get(&pid).cloned());

                let body_content = if node_meta.r#type == "Article" {
                    let mut f = archive
                        .by_name(&node_meta.path)
                        .map_err(|_| format!("Content missing: {}", node_meta.path))?;
                    let mut s = String::new();
                    f.read_to_string(&mut s).map_err(|e| e.to_string())?;
                    s
                } else {
                    String::new()
                };

                let new_body_content = asset_regex
                    .replace_all(&body_content, |caps: &regex::Captures| {
                        if let Ok(old_uuid) = Uuid::parse_str(&caps[1]) {
                            if let Some(new_uuid) = asset_id_map.get(&old_uuid) {
                                return format!("[[asset:{}]]", new_uuid);
                            }
                        }
                        caps[0].to_string()
                    })
                    .to_string();

                let node_type = match node_meta.r#type.as_str() {
                    "Folder" => NodeType::Folder,
                    _ => NodeType::Article,
                };

                let status = match node_meta.status.as_str() {
                    "Published" => ContentStatus::Published,
                    "Archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                };

                let node = Node {
                    id: new_id,
                    parent_id: new_parent_id,
                    author_id: user_id,
                    knowledge_base_id: Some(new_kb_id),
                    r#type: node_type.clone(),
                    title: node_meta.title.clone(),
                    permission_mode: PermissionMode::Private,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                if node_type == NodeType::Article {
                    let article = Article {
                        node,
                        slug: Uuid::new_v4().to_string(),
                        status,
                        category: None,
                        body: ContentBody::Markdown(new_body_content),
                        tags: node_meta.tags,
                        author_name: None,
                        author_avatar: None,
                        derived_data: None,
                        analysis_status: None,
                        analysis_diagnostics: None,
                    };
                    self.article_repo
                        .save(
                            article,
                            UserId(user_id),
                            Some("Restored from backup".to_string()),
                        )
                        .await
                        .map_err(|e| e.to_string())?;
                } else {
                    self.node_repo
                        .save(node, UserId(user_id))
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }

            Ok(new_kb_id)
        }
        .await;

        resolved.cleanup();

        restore_result
    }

    pub fn get_backup_path(&self, filename: &str) -> PathBuf {
        self.backup_root.join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_wrong_archive_type, resolve_backup_archive_path};
    use std::fs::File;
    use std::io::Write;
    use uuid::Uuid;
    use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

    #[test]
    fn detects_portability_manifest_archives() {
        let names = vec![
            "aether-portability.json".to_string(),
            "vocabulary.csv".to_string(),
        ];

        let message = detect_wrong_archive_type(&names).expect("expected portability detection");
        assert!(message.contains("embedded snapshot"));
    }

    #[test]
    fn detects_english_export_archives_without_meta() {
        let names = vec![
            "vocabulary.csv".to_string(),
            "content/example.md".to_string(),
        ];

        let message = detect_wrong_archive_type(&names).expect("expected export detection");
        assert!(message.contains("expected backup meta.json"));
    }

    #[test]
    fn ignores_regular_unknown_archives() {
        let names = vec!["random.txt".to_string()];
        assert!(detect_wrong_archive_type(&names).is_none());
    }

    #[test]
    fn extracts_embedded_snapshot_from_portability_archive() {
        let snapshot_path = std::env::temp_dir().join(format!("snapshot-{}.akb", Uuid::new_v4()));
        let export_path = std::env::temp_dir().join(format!("export-{}.zip", Uuid::new_v4()));
        write_snapshot_archive(&snapshot_path);
        write_portability_archive(&export_path, &snapshot_path, true);

        let resolved = resolve_backup_archive_path(&export_path).expect("expected embedded snapshot");
        assert_ne!(resolved.path, export_path);

        let file = File::open(&resolved.path).expect("open extracted snapshot");
        let mut archive = ZipArchive::new(file).expect("snapshot should be a zip archive");
        assert!(archive.by_name("meta.json").is_ok());

        resolved.cleanup();
        let _ = std::fs::remove_file(snapshot_path);
        let _ = std::fs::remove_file(export_path);
    }

    #[test]
    fn rejects_portability_archive_without_embedded_snapshot() {
        let snapshot_path = std::env::temp_dir().join(format!("snapshot-{}.akb", Uuid::new_v4()));
        let export_path = std::env::temp_dir().join(format!("export-{}.zip", Uuid::new_v4()));
        write_snapshot_archive(&snapshot_path);
        write_portability_archive(&export_path, &snapshot_path, false);

        let error = resolve_backup_archive_path(&export_path).expect_err("expected missing snapshot");
        assert!(error.contains("does not contain an embedded snapshot"));

        let _ = std::fs::remove_file(snapshot_path);
        let _ = std::fs::remove_file(export_path);
    }

    fn write_snapshot_archive(path: &std::path::PathBuf) {
        let file = File::create(path).expect("create snapshot");
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);
        let meta = format!(
            "{{\"version\":\"1.0\",\"exported_at\":\"2026-03-20T00:00:00Z\",\"knowledge_base\":{{\"id\":\"{}\",\"title\":\"Example\",\"description\":null,\"renderer_id\":\"english_v1\",\"tags\":[]}},\"nodes\":[],\"assets_map\":{{}}}}",
            Uuid::nil()
        );
        zip.start_file("meta.json", options).expect("start meta");
        zip.write_all(meta.as_bytes()).expect("write meta");
        zip.finish().expect("finish snapshot");
    }

    fn write_portability_archive(
        path: &std::path::PathBuf,
        snapshot_path: &std::path::PathBuf,
        include_snapshot: bool,
    ) {
        let file = File::create(path).expect("create export");
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);
        let manifest = if include_snapshot {
            "{\"format\":\"aether-portability-export\",\"provider_id\":\"english_v1\",\"restorable\":true,\"snapshot_path\":\"snapshot.akb\"}"
        } else {
            "{\"format\":\"aether-portability-export\",\"provider_id\":\"english_v1\",\"restorable\":false}"
        };
        zip.start_file("aether-portability.json", options)
            .expect("start manifest");
        zip.write_all(manifest.as_bytes()).expect("write manifest");
        zip.start_file("vocabulary.csv", options)
            .expect("start vocabulary");
        zip.write_all(b"word,definition\nexample,test\n")
            .expect("write vocabulary");
        if include_snapshot {
            let snapshot_bytes = std::fs::read(snapshot_path).expect("read snapshot");
            zip.start_file("snapshot.akb", options)
                .expect("start snapshot");
            zip.write_all(&snapshot_bytes).expect("write snapshot");
        }
        zip.finish().expect("finish export");
    }
}
