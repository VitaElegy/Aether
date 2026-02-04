use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::prkb::ports::PrkbRepository;
use crate::domain::ports::RepositoryError;
use crate::domain::prkb::models::{Paper, Feed, InboxItem};
use crate::infrastructure::persistence::postgres::PostgresRepository;
// use crate::infrastructure::persistence::entities::prkb::{feeds, inbox, papers};
use crate::infrastructure::persistence::entities::{prkb_feeds, prkb_inbox, prkb_papers}; 

#[async_trait]
impl PrkbRepository for PostgresRepository {
    // --- FEEDS ---
    async fn create_feed(&self, feed: Feed) -> Result<Uuid, RepositoryError> {
        let model = prkb_feeds::ActiveModel {
            id: Set(feed.id),
            name: Set(feed.name),
            url: Set(feed.url),
            feed_type: Set(feed.feed_type),
            last_fetched_at: Set(feed.last_fetched_at.map(|t| t.into())),
            created_at: Set(feed.created_at.into()),
        };
        prkb_feeds::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(feed.id)
    }

    async fn list_feeds(&self) -> Result<Vec<Feed>, RepositoryError> {
        let models = prkb_feeds::Entity::find()
            .order_by_asc(prkb_feeds::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| Feed {
            id: m.id,
            name: m.name,
            url: m.url,
            feed_type: m.feed_type,
            last_fetched_at: m.last_fetched_at.map(|t| t.with_timezone(&Utc)),
            created_at: m.created_at.with_timezone(&Utc),
        }).collect())
    }

    async fn get_feed(&self, id: Uuid) -> Result<Option<Feed>, RepositoryError> {
        let model = prkb_feeds::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(model.map(|m| Feed {
            id: m.id,
            name: m.name,
            url: m.url,
            feed_type: m.feed_type,
            last_fetched_at: m.last_fetched_at.map(|t| t.with_timezone(&Utc)),
            created_at: m.created_at.with_timezone(&Utc),
        }))
    }

    async fn delete_feed(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_feeds::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_feed_last_fetched(&self, id: Uuid, time: chrono::DateTime<chrono::Utc>) -> Result<(), RepositoryError> {
        let model = prkb_feeds::ActiveModel {
            id: Set(id),
            last_fetched_at: Set(Some(time.into())),
            ..Default::default() 
        };
        prkb_feeds::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // --- INBOX ---
    async fn save_inbox_items(&self, items: Vec<InboxItem>) -> Result<(), RepositoryError> {
        if items.is_empty() { return Ok(()); }
        
        let active_models: Vec<prkb_inbox::ActiveModel> = items.into_iter().map(|item| {
            prkb_inbox::ActiveModel {
                id: Set(item.id),
                feed_id: Set(item.feed_id),
                external_id: Set(item.external_id),
                title: Set(item.title),
                authors: Set(serde_json::to_value(item.authors).unwrap_or(serde_json::json!([]))),
                abstract_text: Set(item.abstract_text),
                url: Set(item.url),
                pdf_url: Set(item.pdf_url),
                publish_date: Set(item.publish_date.into()),
                is_read: Set(item.is_read),
                is_saved: Set(item.is_saved),
                fetched_at: Set(item.fetched_at.into()),
            }
        }).collect();

        // On Conflict: Do nothing or Update?
        // Usually, if we fetch again, we might want to update PDF URL or status if changed?
        // Let's use INSERT ON CONFLICT DO NOTHING (or Update basic fields).
        // SeaORM insert_many doesn't support ON CONFLICT easily without explicit construction.
        // For simplicity, let's treat external_id + feed_id as unique (defined in schema).
        // Ideally we should skip duplicates.
        
        // SeaORM doesn't natively support "ON CONFLICT DO NOTHING" across DBs easily in `insert_many`.
        // We'll iterate and use `insert` with `on_conflict` for robust handling, 
        // OR use raw query if performance matters. 
        // Given item count (20-50), loop is fine.
        
        for model in active_models {
            let res = prkb_inbox::Entity::insert(model)
                .on_conflict(
                    sea_query::OnConflict::columns([prkb_inbox::Column::FeedId, prkb_inbox::Column::ExternalId])
                        .do_nothing()
                        .to_owned()
                )
                .exec(&self.db)
                .await;
                
            if let Err(DbErr::RecordNotInserted) = res {
                // Ignore
            } else if let Err(e) = res {
                return Err(RepositoryError::DatabaseError(e.to_string()));
            }
        }
        
        Ok(())
    }

    async fn get_inbox(&self, limit: u64, offset: u64, unread_only: bool) -> Result<Vec<InboxItem>, RepositoryError> {
        let mut query = prkb_inbox::Entity::find()
            .order_by_desc(prkb_inbox::Column::PublishDate);
            
        if unread_only {
            query = query.filter(prkb_inbox::Column::IsRead.eq(false));
        }
        
        let models = query
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| InboxItem {
            id: m.id,
            feed_id: m.feed_id,
            external_id: m.external_id,
            title: m.title,
            authors: serde_json::from_value(m.authors).unwrap_or_default(),
            abstract_text: m.abstract_text,
            url: m.url,
            pdf_url: m.pdf_url,
            publish_date: m.publish_date.with_timezone(&Utc),
            is_read: m.is_read,
            is_saved: m.is_saved,
            fetched_at: m.fetched_at.with_timezone(&Utc),
        }).collect())
    }

    async fn markup_inbox_item_read(&self, id: Uuid) -> Result<(), RepositoryError> {
        let model = prkb_inbox::ActiveModel {
            id: Set(id),
            is_read: Set(true),
            ..Default::default()
        };
        prkb_inbox::Entity::update(model).exec(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_inbox_item(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_inbox::Entity::delete_by_id(id).exec(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_inbox_item_by_external_id(&self, external_id: &str) -> Result<Option<InboxItem>, RepositoryError> {
        let model = prkb_inbox::Entity::find()
            .filter(prkb_inbox::Column::ExternalId.eq(external_id))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(model.map(|m| InboxItem {
            id: m.id,
            feed_id: m.feed_id,
            external_id: m.external_id,
            title: m.title,
            authors: serde_json::from_value(m.authors).unwrap_or_default(),
            abstract_text: m.abstract_text,
            url: m.url,
            pdf_url: m.pdf_url,
            publish_date: m.publish_date.with_timezone(&Utc),
            is_read: m.is_read,
            is_saved: m.is_saved,
            fetched_at: m.fetched_at.with_timezone(&Utc),
        }))
    }

    // --- LIBRARY (Papers) ---
    async fn save_paper(&self, paper: Paper) -> Result<Uuid, RepositoryError> {
         let model = prkb_papers::ActiveModel {
            id: Set(paper.id),
            title: Set(paper.title),
            authors: Set(serde_json::to_value(paper.authors).unwrap_or(serde_json::json!([]))),
            abstract_text: Set(paper.abstract_text),
            url: Set(paper.url),
            pdf_url: Set(paper.pdf_url),
            publish_date: Set(paper.publish_date.into()),
            source: Set(paper.source),
            saved_at: Set(paper.saved_at.into()),
            is_read: Set(paper.is_read),
            tags: Set(serde_json::to_value(paper.tags).unwrap_or(serde_json::json!([]))),
            arxiv_id: Set(paper.arxiv_id),
        };
        prkb_papers::Entity::insert(model)
            .on_conflict(
                sea_query::OnConflict::column(prkb_papers::Column::Id)
                    .update_columns([
                        prkb_papers::Column::IsRead, prkb_papers::Column::Tags, prkb_papers::Column::PdfUrl
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(paper.id)
    }

    async fn list_papers(&self, limit: u64, offset: u64) -> Result<Vec<Paper>, RepositoryError> {
         let models = prkb_papers::Entity::find()
            .order_by_desc(prkb_papers::Column::SavedAt)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| Paper {
            id: m.id,
            title: m.title,
            authors: serde_json::from_value(m.authors).unwrap_or_default(),
            abstract_text: m.abstract_text,
            url: m.url,
            pdf_url: m.pdf_url,
            publish_date: m.publish_date.with_timezone(&Utc),
            source: m.source,
            saved_at: m.saved_at.with_timezone(&Utc),
            is_read: m.is_read,
            tags: serde_json::from_value(m.tags).unwrap_or_default(),
            arxiv_id: m.arxiv_id,
        }).collect())
    }

    async fn get_paper(&self, id: Uuid) -> Result<Option<Paper>, RepositoryError> {
        let model = prkb_papers::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(model.map(|m| Paper {
            id: m.id,
            title: m.title,
            authors: serde_json::from_value(m.authors).unwrap_or_default(),
            abstract_text: m.abstract_text,
            url: m.url,
            pdf_url: m.pdf_url,
            publish_date: m.publish_date.with_timezone(&Utc),
            source: m.source,
            saved_at: m.saved_at.with_timezone(&Utc),
            is_read: m.is_read,
            tags: serde_json::from_value(m.tags).unwrap_or_default(),
            arxiv_id: m.arxiv_id,
        }))
    }

    async fn update_paper_read_status(&self, id: Uuid, is_read: bool) -> Result<(), RepositoryError> {
        let model = prkb_papers::ActiveModel {
            id: Set(id),
            is_read: Set(is_read),
            ..Default::default()
        };
        prkb_papers::Entity::update(model).exec(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_paper(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_papers::Entity::delete_by_id(id).exec(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
