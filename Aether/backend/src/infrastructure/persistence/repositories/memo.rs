use crate::domain::models::{LinkedEntity, UserId};
use crate::domain::models::{Memo, Node, NodeType, PermissionMode};
use crate::domain::ports::{MemoBulkUpdate, MemoRepository, RepositoryError};
use crate::infrastructure::persistence::entities::{memo_detail, node};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

#[async_trait]
impl MemoRepository for PostgresRepository {
    async fn save(&self, memo: Memo) -> Result<Uuid, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 1. Save Node
        let node_model = node::ActiveModel {
            id: Set(memo.node.id),
            parent_id: Set(memo.node.parent_id),
            author_id: Set(memo.node.author_id),
            knowledge_base_id: Set(memo.node.knowledge_base_id),
            r#type: Set("memo".to_string()),
            title: Set(memo.node.title.clone()),
            permission_mode: Set(match memo.node.permission_mode {
                PermissionMode::Public => "Public".to_string(),
                PermissionMode::Private => "Private".to_string(),
                PermissionMode::Internal => "Internal".to_string(),
            }),
            permission_data: Set(None),
            created_at: Set(memo.node.created_at.into()),
            updated_at: Set(memo.node.updated_at.into()),
        };
        node::Entity::insert(node_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(node::Column::Id)
                    .update_columns([node::Column::Title, node::Column::UpdatedAt])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Save Detail
        let status_val = memo.status.clone();

        let priority_enum = match memo.priority.as_str() {
            "P0" => memo_detail::MemoPriority::P0,
            "P1" => memo_detail::MemoPriority::P1,
            "P2" => memo_detail::MemoPriority::P2,
            "P3" => memo_detail::MemoPriority::P3,
            _ => memo_detail::MemoPriority::P2,
        };

        let color_enum = match memo.color.as_str() {
            "Red" => memo_detail::MemoColor::Red,
            "Green" => memo_detail::MemoColor::Green,
            "Blue" => memo_detail::MemoColor::Blue,
            "Purple" => memo_detail::MemoColor::Purple,
            "Gray" => memo_detail::MemoColor::Gray,
            _ => memo_detail::MemoColor::Yellow,
        };

        let linked_entities_json = if memo.linked_entities.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&memo.linked_entities).unwrap_or(serde_json::json!([])))
        };

        let detail_model = memo_detail::ActiveModel {
            id: Set(memo.node.id),
            project_id: Set(memo.node.knowledge_base_id),
            content: Set(serde_json::json!(memo.content)),
            priority: Set(priority_enum),
            status: Set(status_val),
            color: Set(color_enum),
            is_pinned: Set(memo.is_pinned),
            due_at: Set(memo.due_at.map(|d| d.into())),
            reminder_at: Set(memo.reminder_at.map(|d| d.into())),
            tags: Set(serde_json::to_value(&memo.tags).unwrap_or(serde_json::json!([]))),
            channel: Set(memo.channel),
            linked_entities: Set(linked_entities_json),
            scheduled_at: Set(memo.scheduled_at.map(|d| d.into())),
            snoozed_until: Set(memo.snoozed_until.map(|d| d.into())),
            reviewed_at: Set(memo.reviewed_at.map(|d| d.into())),
        };
        memo_detail::Entity::insert(detail_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(memo_detail::Column::Id)
                    .update_columns([
                        memo_detail::Column::Content,
                        memo_detail::Column::Priority,
                        memo_detail::Column::Status,
                        memo_detail::Column::Color,
                        memo_detail::Column::IsPinned,
                        memo_detail::Column::DueAt,
                        memo_detail::Column::ReminderAt,
                        memo_detail::Column::Tags,
                        memo_detail::Column::Channel,
                        memo_detail::Column::LinkedEntities,
                        memo_detail::Column::ScheduledAt,
                        memo_detail::Column::SnoozedUntil,
                        memo_detail::Column::ReviewedAt,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(memo.node.id)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Memo>, RepositoryError> {
        let result = node::Entity::find_by_id(*id)
            .find_also_related(memo_detail::Entity)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        match result {
            Some((n, Some(d))) => Ok(Some(map_memo(n, d))),
            _ => Ok(None),
        }
    }

    async fn list(
        &self,
        _viewer_id: Option<UserId>,
        author_id: Option<UserId>,
    ) -> Result<Vec<Memo>, RepositoryError> {
        let mut query = node::Entity::find()
            .filter(node::Column::Type.eq("memo"))
            .find_also_related(memo_detail::Entity)
            .order_by_desc(node::Column::CreatedAt);

        if let Some(aid) = author_id {
            query = query.filter(node::Column::AuthorId.eq(aid.0));
        }

        let results = query
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                memos.push(map_memo(n, detail));
            }
        }
        Ok(memos)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn find_by_date_range(
        &self,
        author_id: UserId,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Memo>, RepositoryError> {
        let results = node::Entity::find()
            .filter(node::Column::Type.eq("memo"))
            .filter(node::Column::AuthorId.eq(author_id.0))
            .filter(node::Column::CreatedAt.gte(start))
            .filter(node::Column::CreatedAt.lte(end))
            .find_also_related(memo_detail::Entity)
            .order_by_desc(node::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                memos.push(map_memo(n, detail));
            }
        }
        Ok(memos)
    }

    // MEMO-04: Bulk Update
    async fn bulk_update(
        &self,
        ids: Vec<Uuid>,
        update: MemoBulkUpdate,
    ) -> Result<usize, RepositoryError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut count = 0usize;
        for id in &ids {
            let existing = node::Entity::find_by_id(*id)
                .find_also_related(memo_detail::Entity)
                .one(&txn)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

            if let Some((_n, Some(d))) = existing {
                let mut active: memo_detail::ActiveModel = d.into();

                if let Some(ref status) = update.status {
                    active.status = Set(status.clone());
                }
                if let Some(ref channel) = update.channel {
                    active.channel = Set(Some(channel.clone()));
                }
                if let Some(pinned) = update.is_pinned {
                    active.is_pinned = Set(pinned);
                }
                if let Some(ref priority) = update.priority {
                    let p = match priority.as_str() {
                        "P0" => memo_detail::MemoPriority::P0,
                        "P1" => memo_detail::MemoPriority::P1,
                        "P3" => memo_detail::MemoPriority::P3,
                        _ => memo_detail::MemoPriority::P2,
                    };
                    active.priority = Set(p);
                }
                if let Some(ref snoozed) = update.snoozed_until {
                    active.snoozed_until = Set(Some((*snoozed).into()));
                }

                // Handle tag add/remove
                if update.tags_add.is_some() || update.tags_remove.is_some() {
                    let current_tags_val = active.tags.clone().unwrap();
                    let mut current_tags: Vec<String> =
                        serde_json::from_value(current_tags_val).unwrap_or_default();
                    if let Some(ref add) = update.tags_add {
                        for t in add {
                            if !current_tags.contains(t) {
                                current_tags.push(t.clone());
                            }
                        }
                    }
                    if let Some(ref remove) = update.tags_remove {
                        current_tags.retain(|t| !remove.contains(t));
                    }
                    active.tags =
                        Set(serde_json::to_value(&current_tags).unwrap_or(serde_json::json!([])));
                }

                active
                    .update(&txn)
                    .await
                    .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

                // Also bump node.updated_at
                let mut node_active = node::ActiveModel {
                    id: Set(*id),
                    ..Default::default()
                };
                node_active.updated_at = Set(Utc::now().into());
                node_active
                    .update(&txn)
                    .await
                    .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

                count += 1;
            }
        }

        txn.commit()
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(count)
    }

    // MEMO-04: Bulk Delete
    async fn bulk_delete(&self, ids: Vec<Uuid>) -> Result<usize, RepositoryError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let result = node::Entity::delete_many()
            .filter(node::Column::Id.is_in(ids))
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(result.rows_affected as usize)
    }

    // MEMO-05: Find backlinks — memos whose linked_entities reference target_id
    async fn find_backlinks(&self, target_id: &Uuid) -> Result<Vec<Memo>, RepositoryError> {
        // Use JSON contains query on linked_entities
        // For PostgreSQL JSONB: linked_entities @> '[{"target_id": "..."}]'
        let target_str = target_id.to_string();
        let results = node::Entity::find()
            .filter(node::Column::Type.eq("memo"))
            .find_also_related(memo_detail::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // In-memory filter for backlinks (ideal: use DB JSONB query)
        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                let has_link = detail
                    .linked_entities
                    .as_ref()
                    .map(|le| le.to_string().contains(&target_str))
                    .unwrap_or(false);
                // Also check content for @mentions
                let content_string = detail.content.to_string();
                let content_str = detail
                    .content
                    .as_str()
                    .unwrap_or(&content_string);
                let has_mention = content_str.contains(&target_str);

                if has_link || has_mention {
                    memos.push(map_memo(n, detail));
                }
            }
        }
        Ok(memos)
    }

    // MEMO-06: Review Queue — due today
    async fn find_due_today(&self, author_id: UserId) -> Result<Vec<Memo>, RepositoryError> {
        let today_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let today_end = Utc::now()
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let results = node::Entity::find()
            .filter(node::Column::Type.eq("memo"))
            .filter(node::Column::AuthorId.eq(author_id.0))
            .find_also_related(memo_detail::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                if let Some(due) = &detail.due_at {
                    let due_naive = due.naive_utc();
                    if due_naive >= today_start && due_naive <= today_end {
                        memos.push(map_memo(n, detail));
                    }
                }
            }
        }
        Ok(memos)
    }

    // MEMO-06: Review Queue — overdue
    async fn find_overdue(&self, author_id: UserId) -> Result<Vec<Memo>, RepositoryError> {
        let now = Utc::now();
        let results = node::Entity::find()
            .filter(node::Column::Type.eq("memo"))
            .filter(node::Column::AuthorId.eq(author_id.0))
            .find_also_related(memo_detail::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                if let Some(due) = &detail.due_at {
                    let due_utc = due.with_timezone(&Utc);
                    if due_utc < now && detail.status != "Done" && detail.status != "Archived" {
                        memos.push(map_memo(n, detail));
                    }
                }
            }
        }
        Ok(memos)
    }

    // MEMO-06: Review Queue — stale (not updated in N days)
    async fn find_stale(&self, author_id: UserId, days: i64) -> Result<Vec<Memo>, RepositoryError> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let results = node::Entity::find()
            .filter(node::Column::Type.eq("memo"))
            .filter(node::Column::AuthorId.eq(author_id.0))
            .filter(node::Column::UpdatedAt.lte(cutoff))
            .find_also_related(memo_detail::Entity)
            .order_by_asc(node::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                if detail.status != "Done" && detail.status != "Archived" {
                    memos.push(map_memo(n, detail));
                }
            }
        }
        Ok(memos)
    }
}

fn map_memo(n: node::Model, d: memo_detail::Model) -> Memo {
    let linked_entities: Vec<LinkedEntity> = d
        .linked_entities
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Memo {
        node: Node {
            id: n.id,
            parent_id: n.parent_id,
            author_id: n.author_id,
            knowledge_base_id: n.knowledge_base_id,
            r#type: NodeType::Memo,
            title: n.title,
            permission_mode: match n.permission_mode.as_str() {
                "Private" => PermissionMode::Private,
                "Internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            },
            created_at: n.created_at.with_timezone(&Utc),
            updated_at: n.updated_at.with_timezone(&Utc),
        },
        content: d
            .content
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| d.content.to_string().into())
            .unwrap_or_default(),
        priority: match d.priority {
            memo_detail::MemoPriority::P0 => "P0".to_string(),
            memo_detail::MemoPriority::P1 => "P1".to_string(),
            memo_detail::MemoPriority::P2 => "P2".to_string(),
            memo_detail::MemoPriority::P3 => "P3".to_string(),
        },
        status: d.status,
        color: match d.color {
            memo_detail::MemoColor::Yellow => "Yellow".to_string(),
            memo_detail::MemoColor::Red => "Red".to_string(),
            memo_detail::MemoColor::Green => "Green".to_string(),
            memo_detail::MemoColor::Blue => "Blue".to_string(),
            memo_detail::MemoColor::Purple => "Purple".to_string(),
            memo_detail::MemoColor::Gray => "Gray".to_string(),
        },
        is_pinned: d.is_pinned,
        due_at: d.due_at.map(|dt| dt.with_timezone(&Utc)),
        reminder_at: d.reminder_at.map(|dt| dt.with_timezone(&Utc)),
        tags: serde_json::from_value(d.tags).unwrap_or_default(),
        // New fields
        channel: d.channel,
        excerpt: None, // Computed from content on the fly
        linked_entities,
        scheduled_at: d.scheduled_at.map(|dt| dt.with_timezone(&Utc)),
        snoozed_until: d.snoozed_until.map(|dt| dt.with_timezone(&Utc)),
        reviewed_at: d.reviewed_at.map(|dt| dt.with_timezone(&Utc)),
    }
}
