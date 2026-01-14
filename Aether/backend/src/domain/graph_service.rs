use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use crate::domain::models::GraphNode;
use crate::domain::ports::{GraphRepository, RepositoryError};

#[derive(Clone)]
pub struct GraphService {
    repo: Arc<dyn GraphRepository>,
}

impl GraphService {
    pub fn new(repo: Arc<dyn GraphRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_knowledge_tree(&self, kb_id: Uuid) -> Result<Vec<GraphNode>, RepositoryError> {
        self.repo.get_tree(&kb_id).await
    }

    pub async fn add_node(
        &self,
        kb_id: Uuid,
        parent_id: Option<Uuid>,
        label: String,
        data: serde_json::Value
    ) -> Result<Uuid, RepositoryError> {
        let node = GraphNode {
            id: Uuid::new_v4(),
            knowledge_base_id: kb_id,
            parent_id,
            label,
            data,
            rank: 0, // Default rank
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.save(node).await
    }

    pub async fn delete_node(&self, id: Uuid) -> Result<(), RepositoryError> {
        // TODO: Check for children and warn? Or just let DB set NULL (Configured in Entity).
        self.repo.delete(&id).await
    }
}
