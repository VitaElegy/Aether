use async_trait::async_trait;
use sea_orm::*;
use crate::domain::models::{ContentAggregate, ContentId, ContentStatus};
use crate::domain::ports::{ContentRepository, RepositoryError};
use super::entities::{self, Entity as ContentEntity};
use chrono::Utc;

pub struct PostgresContentRepository {
    db: DatabaseConnection,
}

impl PostgresContentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ContentRepository for PostgresContentRepository {
    async fn save(&self, content: ContentAggregate) -> Result<ContentId, RepositoryError> {
        // MAPPING: Domain -> Persistence
        // We manually map fields to ensure complete control over the storage format.
        let model = entities::ActiveModel {
            id: Set(content.id.0),
            author_id: Set(content.author_id),
            title: Set(content.title),
            slug: Set(content.slug),
            status: Set(format!("{:?}", content.status)), // Simple enum to string
            created_at: Set(content.created_at),
            updated_at: Set(Utc::now()),
            // SeaORM's JSON support makes this elegant.
            // The ContentBody enum automatically serializes to JSON.
            body: Set(serde_json::to_value(content.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
            tags: Set(content.tags),
        };

        // UPSERT Logic (Postgres specific)
        ContentEntity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(entities::Column::Id)
                    .update_columns([
                        entities::Column::Title,
                        entities::Column::Status,
                        entities::Column::Body,
                        entities::Column::UpdatedAt,
                        entities::Column::Tags,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(content.id)
    }

    async fn find_by_id(&self, id: &ContentId) -> Result<Option<ContentAggregate>, RepositoryError> {
        let model = ContentEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = model {
            // MAPPING: Persistence -> Domain
            Ok(Some(ContentAggregate {
                id: ContentId(m.id),
                author_id: m.author_id,
                title: m.title,
                slug: m.slug,
                // Robust parsing of status
                status: match m.status.as_str() {
                    "Published" => ContentStatus::Published,
                    "Archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                created_at: m.created_at,
                updated_at: m.updated_at,
                body: serde_json::from_value(m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: m.tags,
            }))
        } else {
            Ok(None)
        }
    }

    // Omitting other methods for brevity, but the pattern implies they must be implemented.
    async fn find_by_slug(&self, _slug: &str) -> Result<Option<ContentAggregate>, RepositoryError> {
        todo!("Implement slug lookup using similar pattern")
    }

    async fn list(&self, _limit: u64, _offset: u64) -> Result<Vec<ContentAggregate>, RepositoryError> {
        todo!("Implement pagination")
    }

    async fn delete(&self, _id: &ContentId) -> Result<(), RepositoryError> {
        todo!("Implement delete")
    }
}

