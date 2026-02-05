use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::prkb::ports::PrkbRepository;
use crate::domain::ports::RepositoryError;
use crate::domain::prkb::models::{Paper, Feed, InboxItem, Author, Venue, Signals};
use crate::infrastructure::persistence::postgres::PostgresRepository;
// use crate::infrastructure::persistence::entities::prkb::{feeds, inbox, papers};
// Update imports to include new entities
use crate::infrastructure::persistence::entities::{
    prkb_feeds, prkb_inbox, prkb_papers, prkb_authors, prkb_venues, prkb_signals, prkb_papers_authors
}; 

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
                authors: Set(serde_json::to_value(item.authors).unwrap_or(serde_json::json!([]))), // Legacy JSON field
                abstract_text: Set(item.abstract_text),
                url: Set(item.url),
                pdf_url: Set(item.pdf_url),
                publish_date: Set(item.publish_date.into()),
                is_read: Set(item.is_read),
                is_saved: Set(item.is_saved),
                fetched_at: Set(item.fetched_at.into()),
                publication: Set(item.publication),
                state: Set("Inbox".to_string()),
            }
        }).collect();

        for model in active_models {
            let res = prkb_inbox::Entity::insert(model)
                .on_conflict(
                    sea_query::OnConflict::columns([prkb_inbox::Column::FeedId, prkb_inbox::Column::ExternalId])
                        .update_columns([
                            prkb_inbox::Column::Title,
                            prkb_inbox::Column::AbstractText,
                            prkb_inbox::Column::PdfUrl,
                            prkb_inbox::Column::Publication,
                            prkb_inbox::Column::PublishDate,
                        ])
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

    async fn get_inbox(&self, limit: u64, offset: u64, unread_only: bool, publication: Option<String>) -> Result<Vec<InboxItem>, RepositoryError> {
        let mut query = prkb_inbox::Entity::find()
            .order_by_desc(prkb_inbox::Column::PublishDate);
            
        if unread_only {
            query = query.filter(prkb_inbox::Column::IsRead.eq(false));
        }
        
        if let Some(pub_name) = publication {
            query = query.filter(prkb_inbox::Column::Publication.eq(pub_name));
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
            authors: serde_json::from_value(m.authors).unwrap_or_default(), // Legacy: returns strings
            abstract_text: m.abstract_text,
            url: m.url,
            pdf_url: m.pdf_url,
            publish_date: m.publish_date.with_timezone(&Utc),
            is_read: m.is_read,
            is_saved: m.is_saved,
            fetched_at: m.fetched_at.with_timezone(&Utc),
            publication: m.publication,
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
            publication: m.publication,
        }))
    }

    async fn count_inbox(&self, unread_only: bool, publication: Option<String>) -> Result<u64, RepositoryError> {
        let mut query = prkb_inbox::Entity::find();
        if unread_only {
            query = query.filter(prkb_inbox::Column::IsRead.eq(false));
        }
        if let Some(pub_name) = publication {
            query = query.filter(prkb_inbox::Column::Publication.eq(pub_name));
        }
        query.count(&self.db).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn update_inbox_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError> {
        let model = prkb_inbox::ActiveModel {
            id: Set(id),
            state: Set(state),
            ..Default::default()
        };
        prkb_inbox::Entity::update(model).exec(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_unique_publications(&self) -> Result<Vec<String>, RepositoryError> {
        let publications: Vec<String> = prkb_inbox::Entity::find()
            .select_only()
            .column(prkb_inbox::Column::Publication)
            .distinct()
            .filter(prkb_inbox::Column::Publication.is_not_null())
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(publications)
    }

    // --- LIBRARY (Papers) ---
    async fn save_paper(&self, paper: Paper) -> Result<Uuid, RepositoryError> {
        // 1. Transaction Start
        let txn = self.db.begin().await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // 2. Upsert Venue
        let mut venue_id = None;
        if let Some(venue) = paper.venue {
            let v_model = prkb_venues::ActiveModel {
                id: Set(venue.id),
                name: Set(venue.name),
                tier: Set(venue.tier),
                created_at: Set(Utc::now().into()), // Or preserve?
            };
            // Simplistic upsert or insert
             let res = prkb_venues::Entity::insert(v_model.clone())
                .on_conflict(
                    sea_query::OnConflict::column(prkb_venues::Column::Id)
                        .update_column(prkb_venues::Column::Name) // Refresh name
                        .to_owned()
                )
                .exec(&txn)
                .await;
             
             if let Err(e) = res {
                 if !e.to_string().contains("duplicate key") { // Should be handled by on_conflict
                     tracing::error!("Venue Upsert Error: {}", e);
                 }
             }
             venue_id = Some(venue.id);
        }

        // 3. Upsert Paper
        let model = prkb_papers::ActiveModel {
            id: Set(paper.id),
            title: Set(paper.title),
            authors: Set(serde_json::json!([])), // Deprecate string json
            abstract_text: Set(paper.abstract_text),
            url: Set(paper.url),
            pdf_url: Set(paper.pdf_url),
            pdf_local_path: Set(paper.pdf_local_path),
            publish_date: Set(paper.publish_date.into()),
            source: Set(paper.source),
            saved_at: Set(paper.saved_at.into()),
            is_read: Set(paper.is_read),
            state: Set(paper.state),
            tags: Set(serde_json::to_value(paper.tags).unwrap_or(serde_json::json!([]))),
            arxiv_id: Set(paper.arxiv_id),
            venue_id: Set(venue_id),
            metadata: Set(serde_json::to_value(paper.metadata).ok()),
            publication: Set(None), // Deprecated field
        };
        
        prkb_papers::Entity::insert(model)
            .on_conflict(
                sea_query::OnConflict::column(prkb_papers::Column::Id)
                    .update_columns([
                        prkb_papers::Column::IsRead, prkb_papers::Column::Tags, prkb_papers::Column::PdfUrl, prkb_papers::Column::State
                    ])
                    .to_owned()
            )
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // 4. Authors and Relations
        for author in paper.authors {
            // Upsert Author
            let a_model = prkb_authors::ActiveModel {
                id: Set(author.id),
                name: Set(author.name),
                canonical_name: Set(author.canonical_name),
                profile_url: Set(author.profile_url),
                aliases: Set(serde_json::json!([])),
                created_at: Set(Utc::now().into()),
            };
            let _ = prkb_authors::Entity::insert(a_model)
                .on_conflict(
                    sea_query::OnConflict::column(prkb_authors::Column::Id)
                        .do_nothing()
                        .to_owned()
                )
                .exec(&txn)
                .await;
            
            // Insert Relation
            let rel_model = prkb_papers_authors::ActiveModel {
                paper_id: Set(paper.id),
                author_id: Set(author.id),
            };
             let _ = prkb_papers_authors::Entity::insert(rel_model)
                .on_conflict(sea_query::OnConflict::columns([prkb_papers_authors::Column::PaperId, prkb_papers_authors::Column::AuthorId]).do_nothing().to_owned())
                .exec(&txn)
                .await;
        }

        // 5. Signals
        if let Some(signals) = paper.signals {
            let s_model = prkb_signals::ActiveModel {
                paper_id: Set(paper.id),
                citation_count: Set(signals.citation_count),
                github_stars: Set(signals.github_stars),
                sota_rank: Set(signals.sota_rank),
                last_updated: Set(signals.last_updated.into()),
            };
             let _ = prkb_signals::Entity::insert(s_model)
                .on_conflict(
                    sea_query::OnConflict::column(prkb_signals::Column::PaperId)
                        .update_columns([prkb_signals::Column::CitationCount, prkb_signals::Column::GithubStars])
                        .to_owned()
                )
                .exec(&txn)
                .await;
        }

        txn.commit().await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(paper.id)
    }

    async fn list_papers(&self, filter: crate::domain::prkb::models::PaperFilter, limit: u64, offset: u64) -> Result<Vec<Paper>, RepositoryError> {
        // Step 1: Fetch Papers
        let mut query = prkb_papers::Entity::find()
            .order_by_desc(prkb_papers::Column::SavedAt);

        if let Some(vid) = filter.venue_id {
            query = query.filter(prkb_papers::Column::VenueId.eq(vid));
        }
        
        if let Some(read) = filter.is_read {
            query = query.filter(prkb_papers::Column::IsRead.eq(read));
        }

        if let Some(aid) = filter.author_id {
            // Join with papers_authors to filter by author
            // Note: SeaORM defines relation to papers_authors
            query = query
                .join(JoinType::InnerJoin, prkb_papers::Relation::PapersAuthors.def())
                .filter(prkb_papers_authors::Column::AuthorId.eq(aid));
        }

        let paper_models = query
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if paper_models.is_empty() {
            return Ok(vec![]);
        }

        let paper_ids: Vec<Uuid> = paper_models.iter().map(|p| p.id).collect();
        let venue_ids: Vec<Uuid> = paper_models.iter().filter_map(|p| p.venue_id).collect();

        // Step 2: Batch Load Venues
        let venues_map: std::collections::HashMap<Uuid, Venue> = if venue_ids.is_empty() {
            std::collections::HashMap::new()
        } else {
            prkb_venues::Entity::find()
                .filter(prkb_venues::Column::Id.is_in(venue_ids))
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                .into_iter()
                .map(|v| (v.id, Venue { id: v.id, name: v.name, tier: v.tier }))
                .collect()
        };

        // Step 3: Batch Load Signals
        let signals_models = prkb_signals::Entity::find()
             .filter(prkb_signals::Column::PaperId.is_in(paper_ids.clone()))
             .all(&self.db)
             .await
             .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        let signals_map: std::collections::HashMap<Uuid, Signals> = signals_models.into_iter().map(|s| (s.paper_id, Signals {
             citation_count: s.citation_count,
             github_stars: s.github_stars,
             sota_rank: s.sota_rank,
             last_updated: s.last_updated.with_timezone(&Utc),
        })).collect();

        // Step 4: Batch Load Authors via Junction
        let authors_flat: Vec<(prkb_papers_authors::Model, Option<prkb_authors::Model>)> = prkb_papers_authors::Entity::find()
            .filter(prkb_papers_authors::Column::PaperId.is_in(paper_ids.clone()))
            .find_also_related(prkb_authors::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut authors_map: std::collections::HashMap<Uuid, Vec<Author>> = std::collections::HashMap::new();
        for (rel, author_opt) in authors_flat {
             if let Some(a) = author_opt {
                 authors_map.entry(rel.paper_id).or_default().push(Author {
                     id: a.id,
                     name: a.name,
                     canonical_name: a.canonical_name,
                     profile_url: a.profile_url,
                 });
             }
        }

        // Step 5: Assemble
        let result = paper_models.into_iter().map(|p| {
            let venue = p.venue_id.and_then(|vid| venues_map.get(&vid).cloned());
            let signals = signals_map.get(&p.id).cloned();
            let authors = authors_map.remove(&p.id).unwrap_or_default();

            Paper {
                id: p.id,
                title: p.title,
                authors,
                abstract_text: p.abstract_text,
                url: p.url,
                pdf_url: p.pdf_url,
                pdf_local_path: p.pdf_local_path,
                publish_date: p.publish_date.with_timezone(&Utc),
                source: p.source,
                saved_at: p.saved_at.with_timezone(&Utc),
                is_read: p.is_read,
                state: p.state,
                tags: serde_json::from_value(p.tags).unwrap_or_default(),
                arxiv_id: p.arxiv_id,
                venue,
                signals,
                metadata: serde_json::from_value(p.metadata.unwrap_or(serde_json::json!(null))).ok(),
            }
        }).collect();

        Ok(result)
    }

    async fn get_paper(&self, id: Uuid) -> Result<Option<Paper>, RepositoryError> {
        let paper_opt = prkb_papers::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(p) = paper_opt {
             // Fetch Venue
             let venue = if let Some(vid) = p.venue_id {
                 prkb_venues::Entity::find_by_id(vid)
                     .one(&self.db)
                     .await
                     .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                     .map(|v| Venue { id: v.id, name: v.name, tier: v.tier })
             } else {
                 None
             };

             // Fetch Signals
             let signals = prkb_signals::Entity::find()
                 .filter(prkb_signals::Column::PaperId.eq(p.id))
                 .one(&self.db)
                 .await
                 .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                 .map(|s| Signals {
                     citation_count: s.citation_count,
                     github_stars: s.github_stars,
                     sota_rank: s.sota_rank,
                     last_updated: s.last_updated.with_timezone(&Utc),
                 });

             // Fetch Authors
             let authors: Vec<Author> = prkb_papers_authors::Entity::find()
                .filter(prkb_papers_authors::Column::PaperId.eq(p.id))
                .find_also_related(prkb_authors::Entity)
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                .into_iter()
                .filter_map(|(_, a_opt)| a_opt.map(|a| Author {
                     id: a.id,
                     name: a.name,
                     canonical_name: a.canonical_name,
                     profile_url: a.profile_url,
                }))
                .collect();

             Ok(Some(Paper {
                id: p.id,
                title: p.title,
                authors,
                abstract_text: p.abstract_text,
                url: p.url,
                pdf_url: p.pdf_url,
                pdf_local_path: p.pdf_local_path,
                publish_date: p.publish_date.with_timezone(&Utc),
                source: p.source,
                saved_at: p.saved_at.with_timezone(&Utc),
                is_read: p.is_read,
                state: p.state,
                tags: serde_json::from_value(p.tags).unwrap_or_default(),
                arxiv_id: p.arxiv_id,
                venue,
                signals,
                metadata: serde_json::from_value(p.metadata.unwrap_or(serde_json::json!(null))).ok(),
             }))
        } else {
            Ok(None)
        }
    }

    async fn update_paper_read_status(&self, id: Uuid, is_read: bool) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: sea_orm::ActiveValue::Set(id),
            is_read: sea_orm::ActiveValue::Set(is_read),
            ..Default::default()
        };
        update.update(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_paper_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: sea_orm::ActiveValue::Set(id),
            state: sea_orm::ActiveValue::Set(state),
            ..Default::default()
        };
        update.update(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_paper(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_papers::Entity::delete_by_id(id).exec(&self.db).await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn list_venues(&self) -> Result<Vec<Venue>, RepositoryError> {
        let models = prkb_venues::Entity::find()
            .order_by_asc(prkb_venues::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(models.into_iter().map(|v| Venue {
            id: v.id,
            name: v.name,
            tier: v.tier,
        }).collect())
    }
}
