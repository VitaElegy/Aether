use async_trait::async_trait;
use sea_orm::*;
use crate::domain::ports::{TagRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
// We don't have a dedicated `tags` table in new schema. Tags are JSON arrays in details.
// So `get_all_tags` needs to scan ArticleDetails/MemoDetails.
// This is expensive but fine for MVP.
use crate::infrastructure::persistence::entities::{article_detail, memo_detail};

#[async_trait]
impl TagRepository for PostgresRepository {
    async fn get_all_tags(&self) -> Result<Vec<String>, RepositoryError> {
        // Collect from Articles
        let articles = article_detail::Entity::find()
            .select_only()
            .column(article_detail::Column::Tags)
            .into_tuple::<String>()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Collect from Memos
        let memos = memo_detail::Entity::find()
            .select_only()
            .column(memo_detail::Column::Tags)
            .into_tuple::<String>()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut all_tags = std::collections::HashSet::new();

        for json_str in articles.iter().chain(memos.iter()) {
             if let Ok(tags) = serde_json::from_str::<Vec<String>>(json_str) {
                 for t in tags {
                     all_tags.insert(t);
                 }
             }
        }
        
        let mut sorted: Vec<String> = all_tags.into_iter().collect();
        sorted.sort();
        Ok(sorted)
    }
}
