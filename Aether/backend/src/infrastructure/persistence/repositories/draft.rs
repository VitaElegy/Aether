use async_trait::async_trait;
use sea_orm::*;
use crate::domain::models::{UserDraft, UserId};
use crate::domain::ports::{DraftRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::draft; // SeaORM entity
// use uuid::Uuid;

#[async_trait]
impl DraftRepository for PostgresRepository {
    async fn save_draft(&self, draft: UserDraft) -> Result<(), RepositoryError> {
         // Conversion: Domain -> ActiveModel
         // Use serde_json for tags
         let tags_json = draft.tags.map(|t| serde_json::to_string(&t).unwrap_or("[]".to_string()));

         let active_model = draft::ActiveModel {
             user_id: Set(draft.user_id.0),
             target_article_id: Set(draft.target_article_id),
             title: Set(draft.title),
             body: Set(draft.body),
             tags: Set(tags_json),
             category: Set(draft.category),
             knowledge_base_id: Set(draft.knowledge_base_id),
             updated_at: Set(chrono::Utc::now().into()), // Force update time
         };

         // Upsert logic
         draft::Entity::insert(active_model)
             .on_conflict(
                 sea_orm::sea_query::OnConflict::column(draft::Column::UserId)
                     .update_columns([
                         draft::Column::TargetArticleId,
                         draft::Column::Title,
                         draft::Column::Body,
                         draft::Column::Tags,
                         draft::Column::Category,
                         draft::Column::KnowledgeBaseId,
                         draft::Column::UpdatedAt,
                     ])
                     .to_owned()
             )
             .exec(&self.db)
             .await
             .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

         Ok(())
    }

    async fn get_draft(&self, user_id: &UserId) -> Result<Option<UserDraft>, RepositoryError> {
        let model = draft::Entity::find_by_id(user_id.0)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match model {
            Some(m) => Ok(Some(UserDraft {
                user_id: UserId(m.user_id),
                target_article_id: m.target_article_id,
                title: m.title,
                body: m.body,
                tags: m.tags.map(|t| serde_json::from_str(&t).unwrap_or_default()),
                category: m.category,
                knowledge_base_id: m.knowledge_base_id,
                updated_at: m.updated_at.into(), // Convert back to UTC
            })),
            None => Ok(None),
        }
    }

    async fn delete_draft(&self, user_id: &UserId) -> Result<(), RepositoryError> {
        draft::Entity::delete_by_id(user_id.0)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
