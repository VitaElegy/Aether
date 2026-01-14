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
        let detail_model = memo_detail::ActiveModel {
            id: Set(memo.node.id),
            content: Set(memo.content),
            priority: Set(memo.priority),
            tags: Set(serde_json::to_string(&memo.tags).unwrap_or_default()),
        };
        memo_detail::Entity::insert(detail_model)
            .on_conflict(
                 sea_orm::sea_query::OnConflict::column(memo_detail::Column::Id)
                    .update_columns([memo_detail::Column::Content, memo_detail::Column::Priority, memo_detail::Column::Tags])
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
            .filter(node::Column::Type.eq("Memo"))
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
            .filter(node::Column::Type.eq("Memo"))
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
        content: d.content,
        priority: d.priority,
        tags: serde_json::from_str(&d.tags).unwrap_or_default(),
    }
}
