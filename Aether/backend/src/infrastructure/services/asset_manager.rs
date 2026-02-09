use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use serde_json::json;
use chrono::Utc;

use crate::domain::ports::{NodeRepository, ArticleRepository, KnowledgeBaseRepository, RepositoryError};
use crate::domain::models::{
    Node, Article, KnowledgeBase, ContentItem, NodeType, PermissionMode, ContentStatus, ContentBody,
    UserId, KnowledgeBaseId, Visibility
};
use crate::domain::permission_service::PermissionService;
use crate::infrastructure::persistence::postgres::PostgresRepository;

#[derive(Clone)]
pub struct AssetManager {
    _node_repo: Arc<dyn NodeRepository>,
    article_repo: Arc<dyn ArticleRepository>,
    kb_repo: Arc<dyn KnowledgeBaseRepository>,
    perm_service: Arc<PermissionService<PostgresRepository>>,
    storage_root: PathBuf,
}

impl AssetManager {
    pub fn new(
        node_repo: Arc<dyn NodeRepository>,
        article_repo: Arc<dyn ArticleRepository>,
        kb_repo: Arc<dyn KnowledgeBaseRepository>,
        perm_service: Arc<PermissionService<PostgresRepository>>,
        storage_root: String,
    ) -> Self {
        Self {
            _node_repo: node_repo,
            article_repo,
            kb_repo,
            perm_service,
            storage_root: PathBuf::from(storage_root),
        }
    }

    /// Uploads an asset (file) and creates a Node in the user's "My Assets" KB.
    pub async fn upload_asset(
        &self,
        user_id: Uuid,
        filename: String,
        mime_type: String,
        data: &[u8],
    ) -> Result<Article, String> {
        // 1. Ensure "My Assets" KB exists for this user
        let kb_id = self.ensure_my_assets_kb(user_id).await.map_err(|e| e.to_string())?;

        // 2. Compute Hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);

        // 3. Save File to Disk
        // Structure: uploads/ab/abcdef...
        let sharded_dir = self.storage_root.join("uploads").join(&hash_hex[0..2]);
        let file_path = sharded_dir.join(&hash_hex);
        
        tokio::fs::create_dir_all(&sharded_dir).await.map_err(|e| e.to_string())?;
        
        // We overwrite if exists (content addressable, so it's safe)
        let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| e.to_string())?;
        file.write_all(data).await.map_err(|e| e.to_string())?;

        // 4. Create Article (Asset Entity)
        
        // Relative path for storage in DB
        let relative_path = format!("uploads/{}/{}", &hash_hex[0..2], hash_hex);

        let payload = json!({
            "file_path": relative_path,
            "original_filename": filename,
            "mime_type": mime_type,
            "hash": hash_hex,
            "size_bytes": data.len(),
        });

        let id = Uuid::new_v4();
        let article = Article {
            node: Node {
                id,
                parent_id: None,
                author_id: user_id,
                knowledge_base_id: Some(kb_id),
                r#type: NodeType::Article, // Using Article type to wrap Asset
                title: filename.clone(),
                permission_mode: PermissionMode::Private, // Assets are private by default
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            slug: hash_hex.clone(), // Slug is hash
            status: ContentStatus::Published,
            category: Some("Asset".to_string()),
            body: ContentBody::Custom(payload),
            tags: vec![mime_type.clone()],
            author_name: None,
            author_avatar: None,
            derived_data: None,
        };

        self.article_repo.save(article.clone(), UserId(user_id), None).await.map_err(|e| e.to_string())?;

        Ok(article)
    }

    /// Retrieves an asset file stream IF the user has access to the context.
    /// Returns (File Path, Mime Type).
    pub async fn get_asset_file(
        &self,
        asset_id: Uuid,
        context_id: Option<Uuid>,
        user_id: Uuid,
    ) -> Result<(PathBuf, String), String> {
        // 1. Fetch the Asset Article
        let item = self.article_repo.find_by_id(&asset_id).await
            .map_err(|e| e.to_string())?
            .ok_or("Asset not found")?;
            
        let asset_article = match item {
            ContentItem::Article(a) => a,
            _ => return Err("Asset is not an article".to_string()),
        };

        // 2. Check Permissions
        let is_author = asset_article.node.author_id == user_id;
        
        if !is_author {
            if let Some(ctx_id) = context_id {
                // Check Read Access to Context Article
                let can_read_context = self.perm_service.check_permission(user_id, ctx_id, "read").await
                    .map_err(|e| e.to_string())?;
                
                if !can_read_context {
                    return Err("Access denied to context article".to_string());
                }

                // Verify Context actually references Asset
                let context_item = self.article_repo.find_by_id(&ctx_id).await
                    .map_err(|e| e.to_string())?
                    .ok_or("Context article not found")?;

                let references = match context_item {
                    ContentItem::Article(a) => {
                        let body_str = match a.body {
                            ContentBody::Markdown(s) => s,
                            ContentBody::CodeSnippet { code, .. } => code,
                            ContentBody::Custom(v) => v.to_string(),
                            _ => "".to_string(),
                        };
                        body_str.contains(&asset_id.to_string())
                    },
                    _ => false,
                };

                if !references {
                    return Err("Context does not reference this asset".to_string());
                }

            } else {
                return Err("Access denied: No context provided and not owner".to_string());
            }
        }

        // 3. Resolve Path from Body Payload
        let payload = match asset_article.body {
            ContentBody::Custom(v) => v,
            _ => return Err("Invalid asset body format".to_string()),
        };

        let relative_path = payload.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Invalid asset payload: missing file_path")?;
        
        let mime_type = payload.get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream")
            .to_string();

        let full_path = self.storage_root.join(relative_path);

        Ok((full_path, mime_type))
    }

    /// Public method: Ensure "My Assets" KB exists for a user.
    /// Returns the KB ID if it exists or was created.
    pub async fn ensure_my_assets_kb(&self, user_id: Uuid) -> Result<Uuid, RepositoryError> {
        // Convention: renderer_id is "assets_v1"
        
        let kbs = self.kb_repo.list(Some(UserId(user_id)), Some(UserId(user_id))).await?;
        
        for kb in kbs {
            if kb.renderer_id.as_deref() == Some("assets_v1") {
                return Ok(kb.id.0);
            }
        }

        // Not found, create new
        let kb_id = Uuid::new_v4();
        let kb = KnowledgeBase {
            id: KnowledgeBaseId(kb_id),
            author_id: user_id,
            title: "My Assets".to_string(),
            description: Some("System managed asset repository".to_string()),
            renderer_id: Some("assets_v1".to_string()),
            visibility: Visibility::Private,
            tags: vec![],
            cover_image: None,
            cover_offset_y: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.kb_repo.save(kb).await?;
        Ok(kb_id)
    }
}
