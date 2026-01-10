use std::sync::Arc;
use uuid::Uuid;
use crate::domain::ports::{PermissionRepository, UserRepository};
use async_recursion::async_recursion;

#[derive(Clone)]
pub struct PermissionService<R> {
    pub repo: Arc<R>,
}

impl<R> PermissionService<R> 
where R: PermissionRepository + UserRepository + Send + Sync + 'static 
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Primary Entry Point: Checks if User can perform Action on Node
    /// Mapped Actions:
    /// - "read" -> requires "viewer", "editor", "owner"
    /// - "write" -> requires "editor", "owner"
    /// - "delete" -> requires "owner"
    pub async fn check_permission(&self, user_id: Uuid, node_id: Uuid, action: &str) -> Result<bool, anyhow::Error> {
        // 1. Super Admin Bypass (Optimization)
        // If user is admin (permissions=u64::MAX), allow all.
        // We need to fetch user to know this.
        if let Ok(Some(user)) = self.repo.find_by_id(&crate::domain::models::UserId(user_id)).await {
            if user.permissions == u64::MAX {
                return Ok(true);
            }
        }

        // 2. Map Action to Required Relations
        let required_relations = match action {
            "read" => vec!["viewer", "editor", "owner", "author", "parent"], // parent allows inheritance
            "write" => vec!["editor", "owner", "author"],
            "delete" => vec!["owner", "author"],
            _ => return Ok(false),
        };

        // 3. Start Graph Walk
        for relation in required_relations {
             if self.check_relation(node_id, "node", relation, user_id).await? {
                 return Ok(true);
             }
        }
        
        Ok(false)
    }

    #[async_recursion]
    async fn check_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid) -> Result<bool, anyhow::Error> {
        // A. Direct Tuple Check: (Entity, Relation, User:SubjectID)
        if self.repo.has_relation(entity_id, entity_type, relation, subject_id, "user").await? {
            return Ok(true);
        }

        // B. Group Membership Check: (Entity, Relation, Group:G) AND (Group:G, member, User:SubjectID)
        // 1. Get all groups the user is a member of
        let mut user_groups = self.repo.get_subject_groups(subject_id).await?;
        
        // Always include Public Group (Nil UUID)
        // This ensures that if the entity has (Entity, viewer, Group:Public), everyone (including Guest if we treat them as Nil User) has access.
        user_groups.push(Uuid::nil());
        
        for group_id in user_groups {
             // 2. Check if any group has the relation to the entity
             if self.repo.has_relation(entity_id, entity_type, relation, group_id, "group").await? {
                 return Ok(true);
             }
        }

        // C. Inheritance Check (e.g. Node inside a KB)
        // Check if Entity has parents, and if so, check if we have access to them.
        let parents = self.repo.get_parents(entity_id).await?;
        for parent_id in parents {
             // If we have access to the parent (via same relation), we have access to child.
             // Note: usually, having 'owner' of Parent implies 'owner' of Child.
             // But 'viewer' of Parent implies 'viewer' of Child.
             // So we pass the SAME relation up.
             if self.check_relation(parent_id, "node", relation, subject_id).await? {
                 return Ok(true);
             }
        }
        
        Ok(false)
    }
}
