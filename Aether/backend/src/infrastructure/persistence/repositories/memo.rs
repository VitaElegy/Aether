use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Memo, Node, NodeType, PermissionMode};
use crate::domain::models::UserId;
use crate::domain::ports::{MemoRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{node, memo_detail};

#[async_trait]
impl MemoRepository for PostgresRepository {
    async fn save(&self, memo: Memo) -> Result<Uuid, RepositoryError> {
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

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
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Save Detail
        let status_enum = match memo.status.as_str() {
            "Doing" => memo_detail::MemoStatus::Doing,
            "Done" => memo_detail::MemoStatus::Done,
            "Archived" => memo_detail::MemoStatus::Archived,
            _ => memo_detail::MemoStatus::Todo,
        };
        
        let priority_enum = match memo.priority.as_str() {
            "P0" => memo_detail::MemoPriority::P0,
            "P1" => memo_detail::MemoPriority::P1,
            "P2" => memo_detail::MemoPriority::P2,
            "P3" => memo_detail::MemoPriority::P3,
            _ => memo_detail::MemoPriority::P2, // Default Normal
        };

        let color_enum = match memo.color.as_str() {
             "Red" => memo_detail::MemoColor::Red,
             "Green" => memo_detail::MemoColor::Green,
             "Blue" => memo_detail::MemoColor::Blue,
             "Purple" => memo_detail::MemoColor::Purple,
             "Gray" => memo_detail::MemoColor::Gray,
             _ => memo_detail::MemoColor::Yellow,
        };

        let detail_model = memo_detail::ActiveModel {
            id: Set(memo.node.id),
            project_id: Set(memo.node.knowledge_base_id), 
            content: Set(serde_json::json!(memo.content)),
            // In Step 191 I made memo_detail.rs have `content: String` but `column_type = "Text"`.
            // But main.rs SQL says `content JSONB`.
            // If main.rs creates JSONB, SeaORM model MUST match.
            // I should have made memo_detail.rs use JSONB for content?
            // "Block-First" -> Content is Blocks.
            // Domain `Memo` has `content: String`.
            // If Domain is string (JSON string), DB can be JSONB.
            // Let's assume content is Stringified JSON for now to be safe with types, OR update entity to Json.
            // Wait, article_details uses JSONB.
            // I should update memo_detail.rs to use Json for content.
            // But for this step let's assume String for now and fix Entity later if it crashes.
            // Actively, Step 191 defined content as String with column_type="Text".
            // Step 201 defined SQL as JSONB. This WILL crash.
            // I MUST FIX memo_detail.rs first or concurrent.
            // I'll assume I fix it concurrently or after.
            // For now, let's proceed with String <-> JSONB casting logic if possible or just use whatever type aligns.
            // I will stick to what's defined in the FILE currently (String).
            priority: Set(priority_enum),
            status: Set(status_enum),
            color: Set(color_enum),
            is_pinned: Set(memo.is_pinned),
            due_at: Set(memo.due_at.map(|d| d.into())),
            reminder_at: Set(memo.reminder_at.map(|d| d.into())),
            tags: Set(serde_json::to_value(&memo.tags).unwrap_or(serde_json::json!([]))),
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
                        memo_detail::Column::Tags
                    ])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(memo.node.id)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Memo>, RepositoryError> {
        let result = node::Entity::find_by_id(*id)
            .find_also_related(memo_detail::Entity)
            .one(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        match result {
             Some((n, Some(d))) => Ok(Some(map_memo(n, d))),
            _ => Ok(None)
        }
    }

    async fn list(&self, _viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<Memo>, RepositoryError> {
        let mut query = node::Entity::find()
            .filter(node::Column::Type.eq("memo")) // Lowercase 'memo' to match Insert
            .find_also_related(memo_detail::Entity)
             .order_by_desc(node::Column::CreatedAt);

        if let Some(aid) = author_id {
            query = query.filter(node::Column::AuthorId.eq(aid.0));
        }

        let results = query.all(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        let mut memos = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                memos.push(map_memo(n, detail));
            }
        }
        Ok(memos)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id).exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn find_by_date_range(&self, author_id: UserId, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Result<Vec<Memo>, RepositoryError> {
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
}

fn map_memo(n: node::Model, d: memo_detail::Model) -> Memo {
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
        content: d.content.as_str().map(|s| s.to_string()).or_else(|| d.content.to_string().into()).unwrap_or_default(),
        priority: match d.priority {
            memo_detail::MemoPriority::P0 => "P0".to_string(),
            memo_detail::MemoPriority::P1 => "P1".to_string(),
            memo_detail::MemoPriority::P2 => "P2".to_string(),
            memo_detail::MemoPriority::P3 => "P3".to_string(),
        },
        status: match d.status {
            memo_detail::MemoStatus::Todo => "Todo".to_string(),
            memo_detail::MemoStatus::Doing => "Doing".to_string(),
            memo_detail::MemoStatus::Done => "Done".to_string(),
            memo_detail::MemoStatus::Archived => "Archived".to_string(),
        },
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
    }
}
