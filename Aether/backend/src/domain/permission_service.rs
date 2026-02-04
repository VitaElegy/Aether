use std::sync::Arc;
use uuid::Uuid;
use crate::domain::ports::{PermissionRepository, UserRepository, AuditRepository};
use async_recursion::async_recursion;


// Hardcoded UUID for the "System" pseudo-node
pub const SYSTEM_ROOT_ID: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000001); // 0...1

#[derive(Clone)]
pub struct PermissionService<R: PermissionRepository + AuditRepository> {
    pub repo: Arc<R>,
}

impl<R> PermissionService<R> 
where R: PermissionRepository + UserRepository + AuditRepository + Send + Sync + 'static 
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
        // 1. Super Admin Bypass REMOVED (Private by Default)
        // Access is strictly controlled by ReBAC Tuples.
        // Admins must use 'break_glass_access' to gain access to private content.


        // 2. Map Action to Required Relations
        let required_relations = match action {
            "read" => vec!["viewer", "editor", "owner", "author", "parent"], // parent allows inheritance
            "write" => vec!["editor", "owner", "author"],
            "delete" => vec!["owner", "author"],
            "manage_users" => vec!["user_manager", "owner"], // Owner of SYSTEM implies management
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


    /// Admin Feature: Break Glass
    /// Allows a Super Admin to force-acquire 'owner' or 'editor' permission on ANY entity.
    /// This action is AUDITED.
    pub async fn break_glass_access(&self, admin_id: Uuid, entity_id: Uuid, relation: &str) -> Result<(), anyhow::Error> {
        // 1. Verify Admin Status (The only time we check u64::MAX)
        let admin = self.repo.find_by_id(&crate::domain::models::UserId(admin_id)).await?
            .ok_or_else(|| anyhow::anyhow!("Admin not found"))?;
            
        if admin.permissions != u64::MAX {
             return Err(anyhow::anyhow!("Unauthorized: functionality restricted to Super Admins"));
        }

        // 2. Log Critical Audit Event
        self.repo.log_event(
            "break_glass", 
            admin_id, 
            &entity_id.to_string(), 
            serde_json::json!({ "granted_relation": relation })
        ).await.map_err(|e| anyhow::anyhow!("Audit failure: {}", e))?;

        // 3. Grant Permission (Tuple)
        // We add a direct tuple: (Entity, relation, AdminID)
        // We assume entity type is generic "node" for now, or we define it.
        // For safety, let's look up entity type or assume generic "break-glass" covers "node".
        // Current constraint: PermissionService check_relation uses "node".
        self.repo.add_relation(entity_id, "node", relation, admin_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to grant access: {}", e))?;

        Ok(())
        }

    // --- Management API ---

    /// Lists all explicit permissions for a user:
    /// 1. Direct Tuples (User specific grants)
    /// 2. Group Memberships (Roles)
    pub async fn get_user_explicit_permissions(&self, user_id: Uuid) -> Result<serde_json::Value, anyhow::Error> {
        // 1. Get Groups
        let groups = self.repo.get_subject_groups(user_id).await?;
        
        // 2. Get Direct Tuples
        let direct = self.repo.get_direct_relations(user_id).await?;
        
        Ok(serde_json::json!({
            "groups": groups,
            "direct_grants": direct // List of (EntityId, EntityType, Relation)
        }))
    }

    pub async fn grant_permission(&self, user_id: Uuid, entity_id: Uuid, relation: &str) -> Result<(), anyhow::Error> {
        // Enforce: only "owner", "editor", "viewer" are valid for now? Or allow flexible?
        // Let's allow flexible for extensibility.
        self.repo.add_relation(entity_id, "node", relation, user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Grant failed: {}", e))
    }

    pub async fn revoke_permission(&self, user_id: Uuid, entity_id: Uuid, relation: &str) -> Result<(), anyhow::Error> {
        self.repo.remove_relation(entity_id, "node", relation, user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Revoke failed: {}", e))
    }

    // --- Team / Group Management API ---

    pub async fn create_team(&self, name: String, owner_id: Uuid) -> Result<Uuid, anyhow::Error> {
        // 1. Create Group Entity
        let group_id = Uuid::new_v4();
        // Rely on repo specific method or generic logic? 
        // Repo has `create_group` method.
        self.repo.create_group(group_id, name).await
            .map_err(|e| anyhow::anyhow!("Failed to create group: {}", e))?;

        // 2. Add Owner Relation (Group, owner, User)
        self.repo.add_relation(group_id, "group", "owner", owner_id, "user").await?;

        // 3. Add Member Relation (Group, member, User) - Owner is implicitly member? 
        // Let's make it explicit so queries are simpler.
        self.repo.add_relation(group_id, "group", "member", owner_id, "user").await?;

        Ok(group_id)
    }

    pub async fn add_team_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), anyhow::Error> {
        self.repo.add_relation(group_id, "group", "member", user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add member: {}", e))
    }

    pub async fn remove_team_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), anyhow::Error> {
        self.repo.remove_relation(group_id, "group", "member", user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove member: {}", e))
    }
}




