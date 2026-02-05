use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::io::{Write, Read};
use uuid::Uuid;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use zip::{ZipWriter, ZipArchive, write::FileOptions, CompressionMethod};
use regex::Regex;
use tokio::io::AsyncReadExt; // For file reading

use crate::domain::ports::{
    ArticleRepository, KnowledgeBaseRepository, NodeRepository
};
use crate::infrastructure::services::asset_manager::AssetManager;
use crate::domain::models::{
    Article, Node, ContentBody, ContentItem, ContentStatus, NodeType, KnowledgeBaseId, UserId, Visibility, PermissionMode, KnowledgeBase
};

// --- Meta Schema ---

#[derive(Serialize, Deserialize)]
struct BackupMeta {
    version: String,
    exported_at: String,
    knowledge_base: BackupKbMeta,
    nodes: Vec<BackupNodeMeta>,
    assets_map: HashMap<Uuid, String>, // Asset UUID -> Zip Path (assets/hash.png)
}

#[derive(Serialize, Deserialize)]
struct BackupKbMeta {
    id: Uuid,
    title: String,
    description: Option<String>,
    renderer_id: Option<String>,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct BackupNodeMeta {
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
        // 1. Fetch KB
        let kb = self.kb_repo.find_by_id(&KnowledgeBaseId(kb_id))
            .await.map_err(|e| e.to_string())?
            .ok_or("Knowledge Base not found")?;

        if kb.author_id != user_id {
            return Err("Unauthorized".to_string());
        }

        // 2. Fetch All Nodes/Articles
        // We use list with large limit. Pagination might be needed for huge KBs, but for V1 we assume <10k items.
        let items = self.article_repo.list(
            Some(UserId(user_id)), 
            None, 
            Some(kb_id), 
            None, None, 
            10000, 0
        ).await.map_err(|e| e.to_string())?;

        // 3. Prepare ZIP
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{}_{}.akb", kb_id, timestamp);
        let file_path = self.backup_root.join(&filename);
        
        let file = std::fs::File::create(&file_path).map_err(|e| e.to_string())?;
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
                },
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
                zip.start_file(&full_entry_path, options).map_err(|e| e.to_string())?;
                zip.write_all(body_str.as_bytes()).map_err(|e| e.to_string())?;
            } else {
                 zip.add_directory(&full_entry_path, options).map_err(|e| e.to_string())?;
            }

            // Add to Meta
            nodes_meta.push(BackupNodeMeta {
                id: node.id,
                parent_id: node.parent_id,
                title: node.title.clone(),
                slug: match item { ContentItem::Article(a) => a.slug.clone(), _ => "".to_string() },
                r#type: format!("{:?}", node.r#type),
                tags: match item { ContentItem::Article(a) => a.tags.clone(), _ => vec![] },
                status: format!("{:?}", match item { ContentItem::Article(a) => a.status, _ => ContentStatus::Draft }),
                created_at: node.created_at.to_rfc3339(),
                updated_at: node.updated_at.to_rfc3339(),
                path: full_entry_path,
            });
        }

        // 5. Process Assets
        let mut assets_map = HashMap::new();
        for asset_id in assets_to_include {
            match self.asset_manager.get_asset_file(asset_id, None, user_id).await {
                Ok((path, _mime)) => {
                    let entry_name = format!("assets/{}", asset_id); // Flattened assets
                    // Read file
                    if let Ok(mut asset_file) = tokio::fs::File::open(&path).await {
                        let mut buffer = Vec::new();
                        if asset_file.read_to_end(&mut buffer).await.is_ok() {
                            zip.start_file(&entry_name, options).map_err(|e| e.to_string())?;
                            zip.write_all(&buffer).map_err(|e| e.to_string())?;
                            assets_map.insert(asset_id, entry_name);
                        }
                    }
                },
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
        zip.start_file("meta.json", options).map_err(|e| e.to_string())?;
        zip.write_all(meta_json.as_bytes()).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;

        Ok(filename)
    }

    // Helper to recursive build path
    fn build_path(&self, node_id: Uuid, map: &HashMap<Uuid, (Option<Uuid>, String)>) -> String {
        let mut current = node_id;
        let mut parts = Vec::new();
        
        // Safety valve for loops
        let mut visited = HashSet::new();
        
        while let Some((parent, title)) = map.get(&current) {
            if visited.contains(&current) { break; }
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
        let mut entries = tokio::fs::read_dir(&self.backup_root).await.map_err(|e| e.to_string())?;

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

    pub async fn restore_backup(&self, file_path: PathBuf, user_id: Uuid) -> Result<Uuid, String> {
        // 1. Open Zip
        let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

        // 2. Read Meta.json
        let mut meta_file = archive.by_name("meta.json").map_err(|_| "Invalid backup: missing meta.json")?;
        let mut meta_content = String::new();
        meta_file.read_to_string(&mut meta_content).map_err(|e| e.to_string())?;
        drop(meta_file); // Release borrow

        let meta: BackupMeta = serde_json::from_str(&meta_content).map_err(|e| format!("Invalid meta.json: {}", e))?;

        // 3. Create NEW Knowledge Base
        let new_kb_id = Uuid::new_v4();
        let restored_title = format!("{} (Restored {})", meta.knowledge_base.title, Utc::now().format("%Y-%m-%d %H:%M"));
        
        let new_kb = KnowledgeBase {
            id: KnowledgeBaseId(new_kb_id),
            author_id: user_id,
            title: restored_title,
            description: meta.knowledge_base.description,
            tags: meta.knowledge_base.tags,
            renderer_id: meta.knowledge_base.renderer_id,
            visibility: Visibility::Private, // Default to private
            cover_image: None,
            cover_offset_y: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.kb_repo.save(new_kb).await.map_err(|e| e.to_string())?;

        // 4. Map IDs (Old -> New)
        let mut node_id_map: HashMap<Uuid, Uuid> = HashMap::new();
        let mut asset_id_map: HashMap<Uuid, Uuid> = HashMap::new();

        // 5. Restore Assets
        for (old_asset_uuid, zip_path) in meta.assets_map {
            // Extract file from zip to memory
            let mut asset_file = archive.by_name(&zip_path).map_err(|_| format!("Asset missing: {}", zip_path))?;
            let mut buffer = Vec::new();
            asset_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            drop(asset_file);

            // Upload via AssetManager (deduplication happens inside)
            // Guess mime type or default? AssetManager computes hash anyway.
            // We pass "restored_asset" as filename if unknown.
            let mime = "application/octet-stream".to_string(); // In a real system, we'd store mime in meta.json or detect it.
            
            match self.asset_manager.upload_asset(user_id, "restored_asset".to_string(), mime, &buffer).await {
                Ok(new_asset_node) => {
                    // Map old asset ID to new asset ID (Note: upload_asset returns Article struct, its ID is the node ID)
                    // Wait, upload_asset returns Article.
                    asset_id_map.insert(old_asset_uuid, new_asset_node.node.id);
                },
                Err(e) => tracing::error!("Failed to restore asset {}: {}", old_asset_uuid, e),
            }
        }

        // 6. Restore Nodes (First Pass: Create ID Mapping)
        for node_meta in &meta.nodes {
            node_id_map.insert(node_meta.id, Uuid::new_v4());
        }

        // 7. Restore Nodes (Second Pass: Create & Save)
        // We need to process parents first? No, we can save with parent_id even if parent doesn't exist yet in DB?
        // SQL Foreign Keys might complain if parent_id points to non-existent row.
        // PostgresRepository usually handles this if we disable FK or order correctly.
        // Or we can save all, then update parents? Or Toplogical sort?
        // Simple approach: Use recursive creation or retry?
        // Better: Sort by path depth (Folder/ -> Folder/Sub/). `meta.nodes` order is not guaranteed.
        // Let's sort by slash count in path.
        
        let mut sorted_nodes = meta.nodes.clone();
        sorted_nodes.sort_by_key(|n| n.path.matches('/').count());

        let asset_regex = Regex::new(r"\[\[asset:([0-9a-fA-F-]+)\]\]").unwrap();

        for node_meta in sorted_nodes {
            let new_id = *node_id_map.get(&node_meta.id).unwrap();
            
            // Resolve Parent
            let new_parent_id = node_meta.parent_id.and_then(|pid| node_id_map.get(&pid).cloned());

            // Read Body from Zip
            let body_content = if node_meta.r#type == "Article" {
                // Remove trailing slash if any (though meta path should be file path)
                let mut f = archive.by_name(&node_meta.path).map_err(|_| format!("Content missing: {}", node_meta.path))?;
                let mut s = String::new();
                f.read_to_string(&mut s).map_err(|e| e.to_string())?;
                s
            } else {
                String::new()
            };

            // Rewrite Asset Links
            let new_body_content = asset_regex.replace_all(&body_content, |caps: &regex::Captures| {
                if let Ok(old_uuid) = Uuid::parse_str(&caps[1]) {
                    if let Some(new_uuid) = asset_id_map.get(&old_uuid) {
                        return format!("[[asset:{}]]", new_uuid);
                    }
                }
                caps[0].to_string() // Keep original if not found (shouldn't happen if map is complete)
            }).to_string();

            // Create Node
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
                    slug: Uuid::new_v4().to_string(), // Generate new slug to avoid collision
                    status,
                    category: None, // TODO: Store category in meta
                    body: ContentBody::Markdown(new_body_content),
                    tags: node_meta.tags,
                    author_name: None,
                    author_avatar: None,
                    derived_data: None,
                };
                self.article_repo.save(article, UserId(user_id), Some("Restored from backup".to_string())).await.map_err(|e| e.to_string())?;
            } else {
                self.node_repo.save(node, UserId(user_id)).await.map_err(|e| e.to_string())?;
            }
        }

        Ok(new_kb_id)
    }
}
