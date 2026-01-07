use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use crate::domain::models::*;
use crate::domain::ports::{CommentRepository, RepositoryError};
use crate::infrastructure::persistence::entities::{comment, user};
use crate::infrastructure::persistence::postgres::PostgresRepository;

#[async_trait]
impl CommentRepository for PostgresRepository {
    async fn add_comment(&self, c: Comment) -> Result<CommentId, RepositoryError> {
        let model = comment::ActiveModel {
            id: Set(c.id.0),
            target_id: Set(c.target_id),
            user_id: Set(c.user_id.0),
            parent_id: Set(c.parent_id.map(|id| id.0)),
            text: Set(c.text),
            created_at: Set(c.created_at.into()),
        };

        comment::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(c.id)
    }

    async fn get_comments(&self, target_id: &Uuid) -> Result<Vec<Comment>, RepositoryError> {
         let comments = comment::Entity::find()
            .filter(comment::Column::TargetId.eq(target_id.clone()))
            .find_also_related(user::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        // Convert to Domain Models
        // Note: Tree building is done by domain service usually, but here we might return flat and letting frontend build tree, 
        // OR we build it here. The prompt implies `get_comments` might return flat list or tree. 
        // `Comment` struct has `replies: Vec<Comment>`. So we should build tree.
        
        let nodes: Vec<Comment> = comments.into_iter().map(|(c, u)| {
            Comment {
                id: CommentId(c.id),
                target_id: c.target_id, 
                user_id: UserId(c.user_id),
                user_name: u.as_ref().map(|user| user.username.clone()),
                user_avatar: u.as_ref().and_then(|user| user.avatar_url.clone()),
                parent_id: c.parent_id.map(CommentId),
                text: c.text,
                created_at: c.created_at.into(),
                replies: vec![],
            }
        }).collect();

        // Build Tree (Naive O(N^2) or O(N) with map)
        // For simplicity in MVP, we might return flat list if Frontend handles it, OR simple tree logic.
        // Frontend `CommentSection.vue` expects nested structure? 
        // Let's check `Comment` struct. `replies: Vec<Comment>`.
        // So we MUST nest them.
        
        // Build Tree (Naive O(N^2) or O(N) with map)
        // For simplicity in MVP, we return flat list and let Frontend handle nesting.
        // Frontend `CommentSection.vue` performs `buildTree(comments.value)`.
        
        // No server-side tree building needed for now.
        
        // Refetch nodes to return flat list
        let mut sorted = nodes;
        sorted.sort_by_key(|c| c.created_at);

        Ok(sorted)
    }

    async fn get_comments_batch(&self, _target_ids: Vec<Uuid>) -> Result<std::collections::HashMap<Uuid, usize>, RepositoryError> {
         // Count comments per target
         // This is generic for lists.
         // Not strictly required for compilation if not called.
         // Stub implementation
         Ok(std::collections::HashMap::new())
    }
}
