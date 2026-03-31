use chrono::Utc;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::domain::models::{
    Article, ContentBody, ContentItem, ContentStatus, KnowledgeBase, KnowledgeBaseId, Node,
    NodeType, PermissionMode, UserId, Visibility,
};
use crate::domain::permission_service::PermissionService;
use crate::domain::ports::{
    ArticleRepository, KnowledgeBaseRepository, NodeRepository, RepositoryError,
};
use crate::domain::special_kb::{is_assets_renderer, ASSETS_RENDERER_ID};
use crate::infrastructure::persistence::postgres::PostgresRepository;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredAssetType {
    Image,
    Pdf,
    File,
}

impl StoredAssetType {
    fn schema_id(self) -> &'static str {
        match self {
            Self::Image => "image_asset",
            Self::Pdf => "pdf_asset",
            Self::File => "file_asset",
        }
    }

    fn preview_kind(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Pdf => "document",
            Self::File => "file",
        }
    }
}

fn filename_extension(filename: &str) -> Option<String> {
    PathBuf::from(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim().to_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn classify_asset_type(filename: &str, mime_type: &str) -> (StoredAssetType, &'static str) {
    let normalized_mime = mime_type.trim().to_lowercase();

    if normalized_mime.starts_with("image/") {
        return (StoredAssetType::Image, "mime_type");
    }

    if normalized_mime == "application/pdf" {
        return (StoredAssetType::Pdf, "mime_type");
    }

    let extension = filename_extension(filename);
    if extension.as_deref() == Some("pdf") {
        return (StoredAssetType::Pdf, "extension");
    }

    (StoredAssetType::File, "fallback")
}

fn build_asset_payload(
    filename: &str,
    mime_type: &str,
    relative_path: &str,
    hash_hex: &str,
    size_bytes: usize,
) -> serde_json::Value {
    let extension = filename_extension(filename);
    let (asset_type, classification_source) = classify_asset_type(filename, mime_type);
    let normalized_mime = mime_type.trim().to_lowercase();

    json!({
        "version": 2,
        "asset_type": asset_type.schema_id(),
        "display_name": filename,
        "file_path": relative_path,
        "original_filename": filename,
        "mime_type": normalized_mime,
        "hash": hash_hex,
        "size_bytes": size_bytes,
        "metadata": {
            "extension": extension,
            "preview_kind": asset_type.preview_kind(),
            "classification_source": classification_source,
        },
    })
}

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
        let kb_id = self
            .ensure_my_assets_kb(user_id)
            .await
            .map_err(|e| e.to_string())?;

        // 2. Compute Hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);

        // 3. Save File to Disk
        // Structure: uploads/ab/abcdef...
        let sharded_dir = self.storage_root.join("uploads").join(&hash_hex[0..2]);
        let file_path = sharded_dir.join(&hash_hex);

        tokio::fs::create_dir_all(&sharded_dir)
            .await
            .map_err(|e| e.to_string())?;

        // We overwrite if exists (content addressable, so it's safe)
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|e| e.to_string())?;
        file.write_all(data).await.map_err(|e| e.to_string())?;

        // 4. Create Article (Asset Entity)

        // Relative path for storage in DB
        let relative_path = format!("uploads/{}/{}", &hash_hex[0..2], hash_hex);

        let payload =
            build_asset_payload(&filename, &mime_type, &relative_path, &hash_hex, data.len());
        let asset_type = payload
            .get("asset_type")
            .and_then(|value| value.as_str())
            .unwrap_or("file_asset")
            .to_string();
        let normalized_mime = payload
            .get("mime_type")
            .and_then(|value| value.as_str())
            .unwrap_or("application/octet-stream")
            .to_string();

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
            tags: vec![normalized_mime.clone(), asset_type],
            author_name: None,
            author_avatar: None,
            derived_data: None,
        };

        self.article_repo
            .save(article.clone(), UserId(user_id), None)
            .await
            .map_err(|e| e.to_string())?;

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
        let item = self
            .article_repo
            .find_by_id(&asset_id)
            .await
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
                let can_read_context = self
                    .perm_service
                    .check_permission(user_id, ctx_id, "read")
                    .await
                    .map_err(|e| e.to_string())?;

                if !can_read_context {
                    return Err("Access denied to context article".to_string());
                }

                // Verify Context actually references Asset
                let context_item = self
                    .article_repo
                    .find_by_id(&ctx_id)
                    .await
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
                    }
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

        let relative_path = payload
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Invalid asset payload: missing file_path")?;

        let mime_type = payload
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream")
            .to_string();

        let full_path = self.storage_root.join(relative_path);

        Ok((full_path, mime_type))
    }

    /// Public method: Ensure "My Assets" KB exists for a user.
    /// Returns the KB ID if it exists or was created.
    pub async fn ensure_my_assets_kb(&self, user_id: Uuid) -> Result<Uuid, RepositoryError> {
        let kbs = self
            .kb_repo
            .list(Some(UserId(user_id)), Some(UserId(user_id)))
            .await?;

        for kb in kbs {
            if is_assets_renderer(kb.renderer_id.as_deref()) {
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
            renderer_id: Some(ASSETS_RENDERER_ID.to_string()),
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

#[cfg(test)]
mod tests {
    use super::{build_asset_payload, classify_asset_type, StoredAssetType};

    #[test]
    fn classifies_assets_by_mime_and_extension() {
        assert_eq!(
            classify_asset_type("diagram.png", "image/png"),
            (StoredAssetType::Image, "mime_type"),
        );
        assert_eq!(
            classify_asset_type("paper.bin", "application/pdf"),
            (StoredAssetType::Pdf, "mime_type"),
        );
        assert_eq!(
            classify_asset_type("paper.PDF", "application/octet-stream"),
            (StoredAssetType::Pdf, "extension"),
        );
        assert_eq!(
            classify_asset_type("archive.zip", "application/zip"),
            (StoredAssetType::File, "fallback"),
        );
    }

    #[test]
    fn builds_payload_with_typed_metadata() {
        let payload =
            build_asset_payload("diagram.png", "image/png", "uploads/ab/hash", "hash", 2048);

        assert_eq!(
            payload.get("asset_type").and_then(|value| value.as_str()),
            Some("image_asset")
        );
        assert_eq!(
            payload.get("display_name").and_then(|value| value.as_str()),
            Some("diagram.png")
        );
        assert_eq!(
            payload.get("mime_type").and_then(|value| value.as_str()),
            Some("image/png")
        );
        assert_eq!(
            payload
                .pointer("/metadata/preview_kind")
                .and_then(|value| value.as_str()),
            Some("image")
        );
        assert_eq!(
            payload
                .pointer("/metadata/extension")
                .and_then(|value| value.as_str()),
            Some("png")
        );
    }
}
