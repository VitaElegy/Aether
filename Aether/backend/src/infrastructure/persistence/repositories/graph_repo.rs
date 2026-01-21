use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, QueryOrder};
use uuid::Uuid;
use crate::domain::models::GraphNode;
use crate::domain::ports::{GraphRepository, RepositoryError};
use crate::infrastructure::persistence::entities::graph_node;
use crate::infrastructure::persistence::postgres::PostgresRepository;

#[async_trait]
impl GraphRepository for PostgresRepository {
    async fn save(&self, node: GraphNode) -> Result<Uuid, RepositoryError> {
        let active_node = graph_node::ActiveModel {
            id: Set(node.id),
            knowledge_base_id: Set(node.knowledge_base_id),
            parent_id: Set(node.parent_id),
            label: Set(node.label),
            data: Set(node.data),
            rank: Set(node.rank),
            created_at: Set(node.created_at.into()),
            updated_at: Set(node.updated_at.into()),
        };

        graph_node::Entity::insert(active_node)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(graph_node::Column::Id)
                    .update_columns([
                        graph_node::Column::Label,
                        graph_node::Column::ParentId,
                        graph_node::Column::Data,
                        graph_node::Column::Rank,
                        graph_node::Column::UpdatedAt,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(node.id)
    }

    async fn get_tree(&self, kb_id: &Uuid) -> Result<Vec<GraphNode>, RepositoryError> {
        let nodes = graph_node::Entity::find()
            .filter(graph_node::Column::KnowledgeBaseId.eq(*kb_id))
            .order_by_asc(graph_node::Column::Rank)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(nodes.into_iter().map(|n| GraphNode {
            id: n.id,
            knowledge_base_id: n.knowledge_base_id,
            parent_id: n.parent_id,
            label: n.label,
            data: n.data,
            rank: n.rank,
            created_at: n.created_at.into(),
            updated_at: n.updated_at.into(),
        }).collect())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<GraphNode>, RepositoryError> {
        let node = graph_node::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(node.map(|n| GraphNode {
            id: n.id,
            knowledge_base_id: n.knowledge_base_id,
            parent_id: n.parent_id,
            label: n.label,
            data: n.data,
            rank: n.rank,
            created_at: n.created_at.into(),
            updated_at: n.updated_at.into(),
        }))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        graph_node::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
