use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{KnowledgeBase, KnowledgeBaseId, Visibility, UserId};
use crate::domain::ports::{KnowledgeBaseRepository, PermissionRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::knowledge_base;

#[async_trait]
impl KnowledgeBaseRepository for PostgresRepository {
    async fn save(&self, kb: KnowledgeBase) -> Result<KnowledgeBaseId, RepositoryError> {
        let model = knowledge_base::ActiveModel {
            id: Set(kb.id.0),
            author_id: Set(kb.author_id),
            title: Set(kb.title),
            description: Set(kb.description),
            tags: Set(serde_json::to_value(kb.tags).unwrap_or(serde_json::json!([]))),
            cover_image: Set(kb.cover_image),
            visibility: Set(match kb.visibility {
                Visibility::Public => "Public".to_string(),
                Visibility::Private => "Private".to_string(),
                Visibility::Internal => "Internal".to_string(),
            }),
            created_at: Set(kb.created_at.into()),
            updated_at: Set(kb.updated_at.into()),
        };

        knowledge_base::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(knowledge_base::Column::Id)
                    .update_columns([
                        knowledge_base::Column::Title,
                        knowledge_base::Column::Description,
                        knowledge_base::Column::Tags,
                        knowledge_base::Column::CoverImage,
                        knowledge_base::Column::Visibility,
                        knowledge_base::Column::UpdatedAt,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // ReBAC Permissions
        let _ = self.add_relation(kb.id.0, "node", "owner", kb.author_id, "user").await;
        
        if let Visibility::Public = kb.visibility {
             let public_group_id = Uuid::nil();
             let _ = self.add_relation(kb.id.0, "node", "viewer", public_group_id, "group").await;
        }

        Ok(kb.id)
    }

    async fn find_by_id(&self, id: &KnowledgeBaseId) -> Result<Option<KnowledgeBase>, RepositoryError> {
        let result = knowledge_base::Entity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(result.map(|m| map_to_domain(m)))
    }

    async fn find_by_title(&self, author_id: &UserId, title: &str) -> Result<Option<KnowledgeBase>, RepositoryError> {
        let result = knowledge_base::Entity::find()
            .filter(knowledge_base::Column::AuthorId.eq(author_id.0))
            .filter(knowledge_base::Column::Title.eq(title))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(result.map(|m| map_to_domain(m)))
    }

    async fn list(&self, _viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<KnowledgeBase>, RepositoryError> {
        let mut query = knowledge_base::Entity::find();

        if let Some(uid) = author_id {
            query = query.filter(knowledge_base::Column::AuthorId.eq(uid.0));
        }

        // TODO: Visibility filtering (Private vs Public)

        let results = query
            .order_by_desc(knowledge_base::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(results.into_iter().map(map_to_domain).collect())
    }

    async fn delete(&self, id: &KnowledgeBaseId) -> Result<(), RepositoryError> {
        knowledge_base::Entity::delete_by_id(id.0)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}

fn map_to_domain(model: knowledge_base::Model) -> KnowledgeBase {
    KnowledgeBase {
        id: KnowledgeBaseId(model.id),
        author_id: model.author_id,
        title: model.title,
        description: model.description,
        tags: serde_json::from_value(model.tags).unwrap_or_default(),
        cover_image: model.cover_image,
        visibility: match model.visibility.as_str() {
            "Internal" => Visibility::Internal,
            "Private" => Visibility::Private,
            _ => Visibility::Public,
        },
        created_at: model.created_at.with_timezone(&Utc),
        updated_at: model.updated_at.with_timezone(&Utc),
    }
}
