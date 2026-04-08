use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use crate::domain::ports::RepositoryError;
use crate::domain::prkb::models::{
    Author, Collection, Feed, InboxItem, Paper, Signals, Venue,
};
use crate::domain::prkb::ports::PrkbRepository;
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{
    prkb_authors, prkb_collection_items, prkb_collections, prkb_feeds, prkb_inbox, prkb_papers,
    prkb_papers_authors, prkb_signals, prkb_venues,
};

// Helper: map Feed entity -> domain
fn feed_from_model(m: prkb_feeds::Model) -> Feed {
    Feed {
        id: m.id,
        name: m.name,
        url: m.url,
        feed_type: m.feed_type,
        enabled: m.enabled,
        last_fetched_at: m.last_fetched_at.map(|t| t.with_timezone(&Utc)),
        created_at: m.created_at.with_timezone(&Utc),
        health_status: m.health_status,
        total_fetched: m.total_fetched,
        parse_errors: m.parse_errors,
        last_error: m.last_error,
    }
}

fn inbox_from_model(m: prkb_inbox::Model) -> InboxItem {
    InboxItem {
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
        state: m.state,
        priority: m.priority,
        note: m.note,
    }
}

fn signals_from_model(s: prkb_signals::Model) -> Signals {
    Signals {
        citation_count: s.citation_count,
        github_stars: s.github_stars,
        sota_rank: s.sota_rank,
        last_updated: s.last_updated.with_timezone(&Utc),
        feed_freshness: s.feed_freshness,
        venue_tier: s.venue_tier,
        author_recurrence: s.author_recurrence,
        custom_importance: s.custom_importance,
    }
}

fn venue_from_model(v: prkb_venues::Model) -> Venue {
    Venue {
        id: v.id,
        name: v.name,
        tier: v.tier,
    }
}

fn author_from_model(a: prkb_authors::Model) -> Author {
    Author {
        id: a.id,
        name: a.name,
        canonical_name: a.canonical_name,
        profile_url: a.profile_url,
    }
}

fn paper_from_model(
    p: prkb_papers::Model,
    venue: Option<Venue>,
    signals: Option<Signals>,
    authors: Vec<Author>,
) -> Paper {
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
        pdf_status: p.pdf_status,
        notes: p.notes,
    }
}

#[async_trait]
impl PrkbRepository for PostgresRepository {
    // ===== PRKB-01: FEEDS =====
    async fn create_feed(&self, feed: Feed) -> Result<Uuid, RepositoryError> {
        let model = prkb_feeds::ActiveModel {
            id: Set(feed.id),
            name: Set(feed.name),
            url: Set(feed.url),
            feed_type: Set(feed.feed_type),
            enabled: Set(feed.enabled),
            last_fetched_at: Set(feed.last_fetched_at),
            created_at: Set(feed.created_at),
            health_status: Set("unknown".to_string()),
            total_fetched: Set(0),
            parse_errors: Set(0),
            last_error: Set(None),
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
        Ok(models.into_iter().map(feed_from_model).collect())
    }

    async fn get_feed(&self, id: Uuid) -> Result<Option<Feed>, RepositoryError> {
        let model = prkb_feeds::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(model.map(feed_from_model))
    }

    async fn delete_feed(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_feeds::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_feed_last_fetched(
        &self,
        id: Uuid,
        time: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError> {
        let model = prkb_feeds::ActiveModel {
            id: Set(id),
            last_fetched_at: Set(Some(time)),
            ..Default::default()
        };
        prkb_feeds::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_feed_enabled(&self, id: Uuid, enabled: bool) -> Result<(), RepositoryError> {
        let model = prkb_feeds::ActiveModel {
            id: Set(id),
            enabled: Set(enabled),
            ..Default::default()
        };
        prkb_feeds::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_feed_health(
        &self,
        id: Uuid,
        status: String,
        error: Option<String>,
    ) -> Result<(), RepositoryError> {
        let model = prkb_feeds::ActiveModel {
            id: Set(id),
            health_status: Set(status),
            last_error: Set(error),
            ..Default::default()
        };
        prkb_feeds::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn increment_feed_stats(
        &self,
        id: Uuid,
        fetched: i64,
        errors: i64,
    ) -> Result<(), RepositoryError> {
        // Use raw SQL for atomic increment
        let sql = format!(
            "UPDATE prkb_feeds SET total_fetched = total_fetched + {}, parse_errors = parse_errors + {} WHERE id = '{}'",
            fetched, errors, id
        );
        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                sql,
            ))
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ===== PRKB-02: INBOX =====
    async fn save_inbox_items(&self, items: Vec<InboxItem>) -> Result<(), RepositoryError> {
        if items.is_empty() {
            return Ok(());
        }

        let active_models: Vec<prkb_inbox::ActiveModel> = items
            .into_iter()
            .map(|item| prkb_inbox::ActiveModel {
                id: Set(item.id),
                feed_id: Set(item.feed_id),
                external_id: Set(item.external_id),
                title: Set(item.title),
                authors: Set(
                    serde_json::to_value(item.authors).unwrap_or(serde_json::json!([]))
                ),
                abstract_text: Set(item.abstract_text),
                url: Set(item.url),
                pdf_url: Set(item.pdf_url),
                publish_date: Set(item.publish_date),
                is_read: Set(item.is_read),
                is_saved: Set(item.is_saved),
                fetched_at: Set(item.fetched_at),
                publication: Set(item.publication),
                state: Set(item.state),
                priority: Set(item.priority),
                note: Set(item.note),
            })
            .collect();

        for model in active_models {
            let res = prkb_inbox::Entity::insert(model)
                .on_conflict(
                    sea_query::OnConflict::columns([
                        prkb_inbox::Column::FeedId,
                        prkb_inbox::Column::ExternalId,
                    ])
                    .update_columns([
                        prkb_inbox::Column::Title,
                        prkb_inbox::Column::AbstractText,
                        prkb_inbox::Column::PdfUrl,
                        prkb_inbox::Column::Publication,
                        prkb_inbox::Column::PublishDate,
                    ])
                    .to_owned(),
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

    async fn get_inbox(
        &self,
        limit: u64,
        offset: u64,
        unread_only: bool,
        publication: Option<String>,
    ) -> Result<Vec<InboxItem>, RepositoryError> {
        let mut query = prkb_inbox::Entity::find()
            .order_by_desc(prkb_inbox::Column::PublishDate);

        if unread_only {
            query = query.filter(prkb_inbox::Column::IsRead.eq(false));
        }

        if let Some(pub_name) = publication {
            query = query.filter(prkb_inbox::Column::Publication.eq(pub_name));
        }

        // Filter out trashed by default
        query = query.filter(prkb_inbox::Column::State.ne("trashed"));

        let models = query
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(models.into_iter().map(inbox_from_model).collect())
    }

    async fn markup_inbox_item_read(&self, id: Uuid) -> Result<(), RepositoryError> {
        let model = prkb_inbox::ActiveModel {
            id: Set(id),
            is_read: Set(true),
            state: Set("read".to_string()),
            ..Default::default()
        };
        prkb_inbox::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_inbox_item(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_inbox::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_inbox_item_by_external_id(
        &self,
        external_id: &str,
    ) -> Result<Option<InboxItem>, RepositoryError> {
        let model = prkb_inbox::Entity::find()
            .filter(prkb_inbox::Column::ExternalId.eq(external_id))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(model.map(inbox_from_model))
    }

    async fn count_inbox(
        &self,
        unread_only: bool,
        publication: Option<String>,
    ) -> Result<u64, RepositoryError> {
        let mut query = prkb_inbox::Entity::find();
        if unread_only {
            query = query.filter(prkb_inbox::Column::IsRead.eq(false));
        }
        if let Some(pub_name) = publication {
            query = query.filter(prkb_inbox::Column::Publication.eq(pub_name));
        }
        query = query.filter(prkb_inbox::Column::State.ne("trashed"));
        query
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn update_inbox_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError> {
        let mut model = prkb_inbox::ActiveModel {
            id: Set(id),
            state: Set(state.clone()),
            ..Default::default()
        };
        // Auto-set flags based on state
        match state.as_str() {
            "read" => { model.is_read = Set(true); }
            "saved" => { model.is_saved = Set(true); model.is_read = Set(true); }
            _ => {}
        }
        prkb_inbox::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_inbox_priority(
        &self,
        id: Uuid,
        priority: Option<i32>,
    ) -> Result<(), RepositoryError> {
        let model = prkb_inbox::ActiveModel {
            id: Set(id),
            priority: Set(priority),
            ..Default::default()
        };
        prkb_inbox::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_inbox_note(
        &self,
        id: Uuid,
        note: Option<String>,
    ) -> Result<(), RepositoryError> {
        let model = prkb_inbox::ActiveModel {
            id: Set(id),
            note: Set(note),
            ..Default::default()
        };
        prkb_inbox::Entity::update(model)
            .exec(&self.db)
            .await
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

    // ===== PRKB-03/06: PAPERS =====
    async fn save_paper(&self, paper: Paper) -> Result<Uuid, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Upsert Venue
        let mut venue_id = None;
        if let Some(venue) = paper.venue {
            let v_model = prkb_venues::ActiveModel {
                id: Set(venue.id),
                name: Set(venue.name),
                tier: Set(venue.tier),
                created_at: Set(Utc::now()),
            };
            let _ = prkb_venues::Entity::insert(v_model)
                .on_conflict(
                    sea_query::OnConflict::column(prkb_venues::Column::Id)
                        .update_column(prkb_venues::Column::Name)
                        .to_owned(),
                )
                .exec(&txn)
                .await;
            venue_id = Some(venue.id);
        }

        // Upsert Paper
        let model = prkb_papers::ActiveModel {
            id: Set(paper.id),
            title: Set(paper.title),
            authors: Set(serde_json::json!([])),
            abstract_text: Set(paper.abstract_text),
            url: Set(paper.url),
            pdf_url: Set(paper.pdf_url),
            pdf_local_path: Set(paper.pdf_local_path),
            publish_date: Set(paper.publish_date),
            source: Set(paper.source),
            saved_at: Set(paper.saved_at),
            is_read: Set(paper.is_read),
            state: Set(paper.state),
            tags: Set(serde_json::to_value(paper.tags).unwrap_or(serde_json::json!([]))),
            arxiv_id: Set(paper.arxiv_id),
            venue_id: Set(venue_id),
            metadata: Set(serde_json::to_value(paper.metadata).ok()),
            publication: Set(None),
            pdf_status: Set(paper.pdf_status),
            notes: Set(paper.notes),
        };

        prkb_papers::Entity::insert(model)
            .on_conflict(
                sea_query::OnConflict::column(prkb_papers::Column::Id)
                    .update_columns([
                        prkb_papers::Column::IsRead,
                        prkb_papers::Column::Tags,
                        prkb_papers::Column::PdfUrl,
                        prkb_papers::Column::State,
                        prkb_papers::Column::PdfStatus,
                        prkb_papers::Column::Notes,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Authors and Relations
        for author in paper.authors {
            let a_model = prkb_authors::ActiveModel {
                id: Set(author.id),
                name: Set(author.name),
                canonical_name: Set(author.canonical_name),
                profile_url: Set(author.profile_url),
                aliases: Set(serde_json::json!([])),
                created_at: Set(Utc::now()),
            };
            let _ = prkb_authors::Entity::insert(a_model)
                .on_conflict(
                    sea_query::OnConflict::column(prkb_authors::Column::Id)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&txn)
                .await;

            let rel_model = prkb_papers_authors::ActiveModel {
                paper_id: Set(paper.id),
                author_id: Set(author.id),
            };
            let _ = prkb_papers_authors::Entity::insert(rel_model)
                .on_conflict(
                    sea_query::OnConflict::columns([
                        prkb_papers_authors::Column::PaperId,
                        prkb_papers_authors::Column::AuthorId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(&txn)
                .await;
        }

        // Signals
        if let Some(signals) = paper.signals {
            let s_model = prkb_signals::ActiveModel {
                paper_id: Set(paper.id),
                citation_count: Set(signals.citation_count),
                github_stars: Set(signals.github_stars),
                sota_rank: Set(signals.sota_rank),
                last_updated: Set(signals.last_updated),
                feed_freshness: Set(signals.feed_freshness),
                venue_tier: Set(signals.venue_tier),
                author_recurrence: Set(signals.author_recurrence),
                custom_importance: Set(signals.custom_importance),
            };
            let _ = prkb_signals::Entity::insert(s_model)
                .on_conflict(
                    sea_query::OnConflict::column(prkb_signals::Column::PaperId)
                        .update_columns([
                            prkb_signals::Column::CitationCount,
                            prkb_signals::Column::GithubStars,
                            prkb_signals::Column::FeedFreshness,
                            prkb_signals::Column::VenueTier,
                            prkb_signals::Column::AuthorRecurrence,
                            prkb_signals::Column::CustomImportance,
                        ])
                        .to_owned(),
                )
                .exec(&txn)
                .await;
        }

        txn.commit()
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(paper.id)
    }

    async fn list_papers(
        &self,
        filter: crate::domain::prkb::models::PaperFilter,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Paper>, RepositoryError> {
        let mut query = prkb_papers::Entity::find()
            .order_by_desc(prkb_papers::Column::SavedAt);

        if let Some(vid) = filter.venue_id {
            query = query.filter(prkb_papers::Column::VenueId.eq(vid));
        }
        if let Some(read) = filter.is_read {
            query = query.filter(prkb_papers::Column::IsRead.eq(read));
        }
        if let Some(state) = filter.state {
            query = query.filter(prkb_papers::Column::State.eq(state));
        }
        if let Some(year) = filter.year {
            let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            query = query
                .filter(prkb_papers::Column::PublishDate.gte(start))
                .filter(prkb_papers::Column::PublishDate.lt(end));
        }
        if let Some(has_pdf) = filter.has_pdf {
            if has_pdf {
                query = query.filter(prkb_papers::Column::PdfUrl.is_not_null());
            } else {
                query = query.filter(prkb_papers::Column::PdfUrl.is_null());
            }
        }
        if let Some(pdf_status) = filter.pdf_status {
            query = query.filter(prkb_papers::Column::PdfStatus.eq(pdf_status));
        }
        if let Some(q) = &filter.query {
            let like_pattern = format!("%{}%", q);
            query = query.filter(
                Condition::any()
                    .add(prkb_papers::Column::Title.like(&like_pattern))
                    .add(prkb_papers::Column::AbstractText.like(&like_pattern)),
            );
        }
        if let Some(aid) = filter.author_id {
            query = query
                .join(
                    JoinType::InnerJoin,
                    prkb_papers::Relation::PapersAuthors.def(),
                )
                .filter(prkb_papers_authors::Column::AuthorId.eq(aid));
        }
        if let Some(cid) = filter.collection_id {
            query = query
                .join(
                    JoinType::InnerJoin,
                    prkb_papers::Relation::CollectionItems.def(),
                )
                .filter(prkb_collection_items::Column::CollectionId.eq(cid));
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

        // Batch load venues
        let venues_map: std::collections::HashMap<Uuid, Venue> = if venue_ids.is_empty() {
            std::collections::HashMap::new()
        } else {
            prkb_venues::Entity::find()
                .filter(prkb_venues::Column::Id.is_in(venue_ids))
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                .into_iter()
                .map(|v| (v.id, venue_from_model(v)))
                .collect()
        };

        // Batch load signals
        let signals_map: std::collections::HashMap<Uuid, Signals> =
            prkb_signals::Entity::find()
                .filter(prkb_signals::Column::PaperId.is_in(paper_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                .into_iter()
                .map(|s| (s.paper_id, signals_from_model(s)))
                .collect();

        // Batch load authors
        let authors_flat: Vec<(prkb_papers_authors::Model, Option<prkb_authors::Model>)> =
            prkb_papers_authors::Entity::find()
                .filter(prkb_papers_authors::Column::PaperId.is_in(paper_ids.clone()))
                .find_also_related(prkb_authors::Entity)
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut authors_map: std::collections::HashMap<Uuid, Vec<Author>> =
            std::collections::HashMap::new();
        for (rel, author_opt) in authors_flat {
            if let Some(a) = author_opt {
                authors_map
                    .entry(rel.paper_id)
                    .or_default()
                    .push(author_from_model(a));
            }
        }

        let result = paper_models
            .into_iter()
            .map(|p| {
                let venue = p.venue_id.and_then(|vid| venues_map.get(&vid).cloned());
                let signals = signals_map.get(&p.id).cloned();
                let authors = authors_map.remove(&p.id).unwrap_or_default();
                paper_from_model(p, venue, signals, authors)
            })
            .collect();

        Ok(result)
    }

    async fn get_paper(&self, id: Uuid) -> Result<Option<Paper>, RepositoryError> {
        let paper_opt = prkb_papers::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(p) = paper_opt {
            let venue = if let Some(vid) = p.venue_id {
                prkb_venues::Entity::find_by_id(vid)
                    .one(&self.db)
                    .await
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                    .map(venue_from_model)
            } else {
                None
            };

            let signals = prkb_signals::Entity::find()
                .filter(prkb_signals::Column::PaperId.eq(p.id))
                .one(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                .map(signals_from_model);

            let authors: Vec<Author> = prkb_papers_authors::Entity::find()
                .filter(prkb_papers_authors::Column::PaperId.eq(p.id))
                .find_also_related(prkb_authors::Entity)
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
                .into_iter()
                .filter_map(|(_, a_opt)| a_opt.map(author_from_model))
                .collect();

            Ok(Some(paper_from_model(p, venue, signals, authors)))
        } else {
            Ok(None)
        }
    }

    async fn update_paper_read_status(
        &self,
        id: Uuid,
        is_read: bool,
    ) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: Set(id),
            is_read: Set(is_read),
            ..Default::default()
        };
        update
            .update(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_paper_state(&self, id: Uuid, state: String) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: Set(id),
            state: Set(state),
            ..Default::default()
        };
        update
            .update(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_paper_tags(&self, id: Uuid, tags: Vec<String>) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: Set(id),
            tags: Set(serde_json::to_value(tags).unwrap_or(serde_json::json!([]))),
            ..Default::default()
        };
        update
            .update(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_paper_notes(
        &self,
        id: Uuid,
        notes: Option<String>,
    ) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: Set(id),
            notes: Set(notes),
            ..Default::default()
        };
        update
            .update(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_paper_pdf_status(
        &self,
        id: Uuid,
        status: String,
        local_path: Option<String>,
    ) -> Result<(), RepositoryError> {
        let update = prkb_papers::ActiveModel {
            id: Set(id),
            pdf_status: Set(status),
            pdf_local_path: Set(local_path),
            ..Default::default()
        };
        update
            .update(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_paper(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_papers::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn list_venues(&self) -> Result<Vec<Venue>, RepositoryError> {
        let models = prkb_venues::Entity::find()
            .order_by_asc(prkb_venues::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(models.into_iter().map(venue_from_model).collect())
    }

    // ===== PRKB-04: SEARCH =====
    async fn search_papers(
        &self,
        query: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Paper>, RepositoryError> {
        let filter = crate::domain::prkb::models::PaperFilter {
            query: Some(query.to_string()),
            ..Default::default()
        };
        self.list_papers(filter, limit, offset).await
    }

    async fn count_papers(
        &self,
        filter: crate::domain::prkb::models::PaperFilter,
    ) -> Result<u64, RepositoryError> {
        let mut query = prkb_papers::Entity::find();
        if let Some(state) = filter.state {
            query = query.filter(prkb_papers::Column::State.eq(state));
        }
        if let Some(q) = &filter.query {
            let like = format!("%{}%", q);
            query = query.filter(
                Condition::any()
                    .add(prkb_papers::Column::Title.like(&like))
                    .add(prkb_papers::Column::AbstractText.like(&like)),
            );
        }
        query
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    // ===== PRKB-05: COLLECTIONS =====
    async fn create_collection(&self, collection: Collection) -> Result<Uuid, RepositoryError> {
        let model = prkb_collections::ActiveModel {
            id: Set(collection.id),
            name: Set(collection.name),
            collection_type: Set(collection.collection_type),
            description: Set(collection.description),
            created_at: Set(collection.created_at),
            updated_at: Set(collection.updated_at),
        };
        prkb_collections::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(collection.id)
    }

    async fn list_collections(&self) -> Result<Vec<Collection>, RepositoryError> {
        let models = prkb_collections::Entity::find()
            .order_by_asc(prkb_collections::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut result = Vec::new();
        for m in models {
            let count = prkb_collection_items::Entity::find()
                .filter(prkb_collection_items::Column::CollectionId.eq(m.id))
                .count(&self.db)
                .await
                .unwrap_or(0);
            result.push(Collection {
                id: m.id,
                name: m.name,
                collection_type: m.collection_type,
                description: m.description,
                paper_count: count as i64,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
            });
        }
        Ok(result)
    }

    async fn get_collection(&self, id: Uuid) -> Result<Option<Collection>, RepositoryError> {
        let model = prkb_collections::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(m) = model {
            let count = prkb_collection_items::Entity::find()
                .filter(prkb_collection_items::Column::CollectionId.eq(m.id))
                .count(&self.db)
                .await
                .unwrap_or(0);
            Ok(Some(Collection {
                id: m.id,
                name: m.name,
                collection_type: m.collection_type,
                description: m.description,
                paper_count: count as i64,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_collection(
        &self,
        id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<(), RepositoryError> {
        let model = prkb_collections::ActiveModel {
            id: Set(id),
            name: Set(name),
            description: Set(description),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        prkb_collections::Entity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_collection(&self, id: Uuid) -> Result<(), RepositoryError> {
        prkb_collections::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn add_paper_to_collection(
        &self,
        collection_id: Uuid,
        paper_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let count = prkb_collection_items::Entity::find()
            .filter(prkb_collection_items::Column::CollectionId.eq(collection_id))
            .count(&self.db)
            .await
            .unwrap_or(0);

        let model = prkb_collection_items::ActiveModel {
            collection_id: Set(collection_id),
            paper_id: Set(paper_id),
            added_at: Set(Utc::now()),
            sort_order: Set(count as i32),
        };
        let _ = prkb_collection_items::Entity::insert(model)
            .on_conflict(
                sea_query::OnConflict::columns([
                    prkb_collection_items::Column::CollectionId,
                    prkb_collection_items::Column::PaperId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(&self.db)
            .await;
        Ok(())
    }

    async fn remove_paper_from_collection(
        &self,
        collection_id: Uuid,
        paper_id: Uuid,
    ) -> Result<(), RepositoryError> {
        prkb_collection_items::Entity::delete_many()
            .filter(prkb_collection_items::Column::CollectionId.eq(collection_id))
            .filter(prkb_collection_items::Column::PaperId.eq(paper_id))
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn list_collection_papers(
        &self,
        collection_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Paper>, RepositoryError> {
        let filter = crate::domain::prkb::models::PaperFilter {
            collection_id: Some(collection_id),
            ..Default::default()
        };
        self.list_papers(filter, limit, offset).await
    }

    // ===== PRKB-07: SIGNALS =====
    async fn update_paper_signals(
        &self,
        paper_id: Uuid,
        signals: crate::domain::prkb::models::Signals,
    ) -> Result<(), RepositoryError> {
        let s_model = prkb_signals::ActiveModel {
            paper_id: Set(paper_id),
            citation_count: Set(signals.citation_count),
            github_stars: Set(signals.github_stars),
            sota_rank: Set(signals.sota_rank),
            last_updated: Set(Utc::now()),
            feed_freshness: Set(signals.feed_freshness),
            venue_tier: Set(signals.venue_tier),
            author_recurrence: Set(signals.author_recurrence),
            custom_importance: Set(signals.custom_importance),
        };
        let _ = prkb_signals::Entity::insert(s_model)
            .on_conflict(
                sea_query::OnConflict::column(prkb_signals::Column::PaperId)
                    .update_columns([
                        prkb_signals::Column::CitationCount,
                        prkb_signals::Column::GithubStars,
                        prkb_signals::Column::SotaRank,
                        prkb_signals::Column::FeedFreshness,
                        prkb_signals::Column::VenueTier,
                        prkb_signals::Column::AuthorRecurrence,
                        prkb_signals::Column::CustomImportance,
                        prkb_signals::Column::LastUpdated,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ===== PRKB-08: PORTABILITY =====
    async fn find_paper_by_doi(&self, doi: &str) -> Result<Option<Paper>, RepositoryError> {
        // DOI is stored in metadata.bibtex.doi
        let models = prkb_papers::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        for p in models {
            if let Some(meta_val) = &p.metadata {
                if let Ok(meta) =
                    serde_json::from_value::<crate::domain::prkb::models::PaperMetadata>(
                        meta_val.clone(),
                    )
                {
                    if let Some(bib) = &meta.bibtex {
                        if bib.doi.as_deref() == Some(doi) {
                            return self.get_paper(p.id).await;
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    async fn find_paper_by_external_id(
        &self,
        external_id: &str,
    ) -> Result<Option<Paper>, RepositoryError> {
        let model = prkb_papers::Entity::find()
            .filter(prkb_papers::Column::ArxivId.eq(external_id))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(p) = model {
            self.get_paper(p.id).await
        } else {
            Ok(None)
        }
    }

    async fn find_paper_by_title(&self, title: &str) -> Result<Option<Paper>, RepositoryError> {
        let model = prkb_papers::Entity::find()
            .filter(prkb_papers::Column::Title.eq(title))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(p) = model {
            self.get_paper(p.id).await
        } else {
            Ok(None)
        }
    }
}
