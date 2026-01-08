use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use crate::domain::models::{Node, NodeType, PermissionMode};
use crate::domain::models::UserId;
use crate::domain::ports::{NodeRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::node;

#[async_trait]
impl NodeRepository for PostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Node>, RepositoryError> {
        let node_model = node::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match node_model {
            Some(n) => Ok(Some(Node {
                id: n.id,
                parent_id: n.parent_id,
                author_id: n.author_id,
                knowledge_base_id: n.knowledge_base_id,
                r#type: match n.r#type.as_str() {
                    "article" => NodeType::Article,
                    "vocabulary" => NodeType::Vocabulary,
                    "memo" => NodeType::Memo,
                    "folder" => NodeType::Folder,
                    _ => NodeType::Article, 
                },
                title: n.title,
                permission_mode: match n.permission_mode.as_str() {
                    "private" => PermissionMode::Private,
                    "internal" => PermissionMode::Internal,
                    _ => PermissionMode::Public,
                },
                created_at: n.created_at.into(),
                updated_at: n.updated_at.into(),
            })),
            None => Ok(None),
        }
    }

    async fn save(&self, node: Node, _user_id: UserId) -> Result<Uuid, RepositoryError> {
        // user_id ignored for now as node.author_id is used.
        let model = node::ActiveModel {
            id: Set(node.id),
            parent_id: Set(node.parent_id),
            author_id: Set(node.author_id),
            knowledge_base_id: Set(node.knowledge_base_id),
            r#type: Set(match node.r#type {
                NodeType::Article => "Article".to_string(),
                NodeType::Vocabulary => "Vocabulary".to_string(),
                NodeType::Memo => "Memo".to_string(),
                NodeType::Folder => "Folder".to_string(),
            }),
            title: Set(node.title),
            permission_mode: Set(match node.permission_mode {
                PermissionMode::Public => "public".to_string(),
                PermissionMode::Private => "private".to_string(),
                PermissionMode::Internal => "internal".to_string(),
            }),
            permission_data: Set(None), 
            created_at: Set(node.created_at.into()),
            updated_at: Set(node.updated_at.into()),
        };

        // Standard Insert. Assuming ID is unique or handling error.
        // For MVP, if ID exists, we should Update.
        // SeaORM doesn't have "Save" (Upsert) easily without OnConflict which caused syntax earlier.
        // Let's Try Insert, if fails (Duplicate), then Update.
        // OR simpler: Try Find first? No, extra query.
        // Use OnConflict correctly? 
        // Let's just do Insert for now. If ID exists error, we can handle it later or use OnConflict if I can write it correctly.
        // But for "Refactor compilation fix", Insert is safe enough to compile.
        
        node::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(node::Column::Id)
                    .update_columns([
                        node::Column::Title, 
                        node::Column::ParentId, 
                        node::Column::PermissionMode, 
                        node::Column::UpdatedAt,
                        node::Column::KnowledgeBaseId
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(node.id)
    }

    async fn list_by_parent(&self, parent_id: Option<Uuid>) -> Result<Vec<Node>, RepositoryError> {
        let condition = if let Some(pid) = parent_id {
            node::Column::ParentId.eq(pid)
        } else {
            node::Column::ParentId.is_null()
        };

        let nodes = node::Entity::find()
            .filter(condition)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(nodes.into_iter().map(|n| Node {
            id: n.id,
            parent_id: n.parent_id,
            author_id: n.author_id,
            knowledge_base_id: n.knowledge_base_id,
            r#type: match n.r#type.as_str() {
                "article" => NodeType::Article,
                "vocabulary" => NodeType::Vocabulary,
                "memo" => NodeType::Memo,
                "folder" => NodeType::Folder,
                _ => NodeType::Article, 
            },
            title: n.title,
            permission_mode: match n.permission_mode.as_str() {
                "private" => PermissionMode::Private,
                "internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            },
            created_at: n.created_at.into(),
            updated_at: n.updated_at.into(),
        }).collect())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
