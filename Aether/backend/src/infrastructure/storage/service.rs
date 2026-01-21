use std::sync::Arc;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::VrkbAsset;
use crate::domain::ports::{VrkbRepository, RepositoryError};
use std::path::Path;

#[derive(Clone)]
pub struct AssetStorageService {
    repo: Arc<dyn VrkbRepository>,
    storage_root: String,
}

impl AssetStorageService {
    pub fn new(repo: Arc<dyn VrkbRepository>, storage_root: String) -> Self {
        Self { repo, storage_root }
    }

    pub async fn store_asset(&self, mapped_file_name: &str, data: &[u8], mime_type: &str) -> Result<VrkbAsset, String> {
        // 1. Compute Hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);

        // 2. Check Deduplication
        if let Ok(Some(existing)) = self.repo.get_asset_by_hash(&hash_hex).await {
            return Ok(existing);
        }

        // 3. Save File
        // Use first 2 chars for subfolder to avoid directory bloat: objects/ab/abcdef...
        let sharded_dir = format!("{}/objects/{}", self.storage_root, &hash_hex[0..2]);
        let file_path = format!("{}/{}", sharded_dir, hash_hex);
        
        tokio::fs::create_dir_all(&sharded_dir).await.map_err(|e| e.to_string())?;
        
        let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| e.to_string())?;
        file.write_all(data).await.map_err(|e| e.to_string())?;

        // 4. Create Entity
        let asset = VrkbAsset {
            id: Uuid::new_v4(),
            hash: hash_hex,
            storage_path: file_path, // Relative or Absolute? Storing full path for now or relative to root? Let's use relative to app root or whatever
            mime_type: mime_type.to_string(),
            size_bytes: data.len() as i64,
            created_at: Utc::now(),
        };

        self.repo.create_asset(asset.clone())
            .await
            .map_err(|e| e.to_string())?;

        Ok(asset)
    }
}
