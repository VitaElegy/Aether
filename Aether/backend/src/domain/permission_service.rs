use crate::domain::ports::{AuditRepository, PermissionRepository, UserRepository};
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// Hardcoded UUID for the "System" pseudo-node
pub const SYSTEM_ROOT_ID: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000001); // 0...1

/// Detailed explanation of a permission check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionExplanation {
    pub allowed: bool,
    /// One of: "direct_grant", "group_membership", "parent_inheritance",
    /// "author_access", "context_proxy", "denied"
    pub reason_code: String,
    /// Human-readable explanation of why access was allowed or denied.
    pub reason_text: String,
    /// The chain of entities that led to the permission decision.
    pub context_chain: Vec<ContextChainItem>,
    /// IDs of contexts (articles) that reference the target asset.
    pub referenced_by: Vec<String>,
}

/// One link in the permission resolution chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChainItem {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub relation: String,
    /// How the relation was resolved: "direct", "group", "parent"
    pub via: String,
}

#[derive(Clone)]
pub struct PermissionService<R: PermissionRepository + AuditRepository> {
    pub repo: Arc<R>,
}

impl<R> PermissionService<R>
where
    R: PermissionRepository + UserRepository + AuditRepository + Send + Sync + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Primary Entry Point: Checks if User can perform Action on Node.
    /// Delegates to `check_permission_explained` and returns only the boolean.
    ///
    /// Mapped Actions:
    /// - "read" -> requires "viewer", "editor", "owner"
    /// - "write" -> requires "editor", "owner"
    /// - "delete" -> requires "owner"
    pub async fn check_permission(
        &self,
        user_id: Uuid,
        node_id: Uuid,
        action: &str,
    ) -> Result<bool, anyhow::Error> {
        let explanation = self
            .check_permission_explained(user_id, node_id, action)
            .await?;
        Ok(explanation.allowed)
    }

    /// Extended entry point that returns a full [`PermissionExplanation`]
    /// describing *why* the permission was granted or denied.
    pub async fn check_permission_explained(
        &self,
        user_id: Uuid,
        node_id: Uuid,
        action: &str,
    ) -> Result<PermissionExplanation, anyhow::Error> {
        // Map Action to Required Relations
        let required_relations: Vec<&str> = match action {
            "read" => vec!["viewer", "editor", "owner", "author", "parent"],
            "write" => vec!["editor", "owner", "author"],
            "delete" => vec!["owner", "author"],
            "manage_users" => vec!["user_manager", "owner"],
            _ => {
                return Ok(PermissionExplanation {
                    allowed: false,
                    reason_code: "denied".to_string(),
                    reason_text: format!("Unknown action '{}'", action),
                    context_chain: vec![],
                    referenced_by: vec![],
                });
            }
        };

        // Walk each required relation and collect explanation on the first match
        for relation in &required_relations {
            let mut chain: Vec<ContextChainItem> = Vec::new();
            if self
                .check_relation_explained(node_id, "node", relation, user_id, &mut chain)
                .await?
            {
                let (reason_code, reason_text) = Self::build_reason(&chain, relation);
                return Ok(PermissionExplanation {
                    allowed: true,
                    reason_code,
                    reason_text,
                    context_chain: chain,
                    referenced_by: vec![],
                });
            }
        }

        Ok(PermissionExplanation {
            allowed: false,
            reason_code: "denied".to_string(),
            reason_text: format!(
                "User has none of the required relations ({}) on this node",
                required_relations.join(", ")
            ),
            context_chain: vec![],
            referenced_by: vec![],
        })
    }

    /// Derives a human-readable reason code/text pair from the collected chain.
    fn build_reason(chain: &[ContextChainItem], matched_relation: &str) -> (String, String) {
        if chain.is_empty() {
            return (
                "direct_grant".to_string(),
                format!("Granted via direct '{}' relation", matched_relation),
            );
        }

        let last = &chain[chain.len() - 1];
        match last.via.as_str() {
            "direct" => (
                "direct_grant".to_string(),
                format!(
                    "Granted via direct '{}' relation on {}:{}",
                    last.relation, last.entity_type, last.entity_id
                ),
            ),
            "group" => (
                "group_membership".to_string(),
                format!(
                    "Granted via group membership ('{}') on {}:{}",
                    last.relation, last.entity_type, last.entity_id
                ),
            ),
            "parent" => (
                "parent_inheritance".to_string(),
                format!(
                    "Granted via parent inheritance ('{}') through {}:{}",
                    last.relation, last.entity_type, last.entity_id
                ),
            ),
            _ => (
                "direct_grant".to_string(),
                format!("Granted via '{}' relation", matched_relation),
            ),
        }
    }

    /// Like `check_relation` but appends to `chain` so callers can inspect the
    /// resolution path.
    #[async_recursion]
    async fn check_relation_explained(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        relation: &str,
        subject_id: Uuid,
        chain: &mut Vec<ContextChainItem>,
    ) -> Result<bool, anyhow::Error> {
        // A. Direct Tuple Check
        if self
            .repo
            .has_relation(entity_id, entity_type, relation, subject_id, "user")
            .await?
        {
            chain.push(ContextChainItem {
                entity_id,
                entity_type: entity_type.to_string(),
                relation: relation.to_string(),
                via: "direct".to_string(),
            });
            return Ok(true);
        }

        // B. Group Membership Check
        let mut user_groups = self.repo.get_subject_groups(subject_id).await?;
        user_groups.push(Uuid::nil()); // Public group

        for group_id in user_groups {
            if self
                .repo
                .has_relation(entity_id, entity_type, relation, group_id, "group")
                .await?
            {
                chain.push(ContextChainItem {
                    entity_id,
                    entity_type: entity_type.to_string(),
                    relation: relation.to_string(),
                    via: "group".to_string(),
                });
                return Ok(true);
            }
        }

        // C. Inheritance Check (parent walk)
        let parents = self.repo.get_parents(entity_id).await?;
        for parent_id in parents {
            let mut sub_chain = Vec::new();
            if self
                .check_relation_explained(parent_id, "node", relation, subject_id, &mut sub_chain)
                .await?
            {
                chain.push(ContextChainItem {
                    entity_id,
                    entity_type: entity_type.to_string(),
                    relation: relation.to_string(),
                    via: "parent".to_string(),
                });
                chain.extend(sub_chain);
                return Ok(true);
            }
        }

        Ok(false)
    }

    // Keep the original check_relation for backward compat in internal callers
    #[async_recursion]
    async fn check_relation(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        relation: &str,
        subject_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        // A. Direct Tuple Check: (Entity, Relation, User:SubjectID)
        if self
            .repo
            .has_relation(entity_id, entity_type, relation, subject_id, "user")
            .await?
        {
            return Ok(true);
        }

        // B. Group Membership Check
        let mut user_groups = self.repo.get_subject_groups(subject_id).await?;
        user_groups.push(Uuid::nil());

        for group_id in user_groups {
            if self
                .repo
                .has_relation(entity_id, entity_type, relation, group_id, "group")
                .await?
            {
                return Ok(true);
            }
        }

        // C. Inheritance Check
        let parents = self.repo.get_parents(entity_id).await?;
        for parent_id in parents {
            if self
                .check_relation(parent_id, "node", relation, subject_id)
                .await?
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Admin Feature: Break Glass
    /// Allows a Super Admin to force-acquire 'owner' or 'editor' permission on ANY entity.
    /// This action is AUDITED.
    pub async fn break_glass_access(
        &self,
        admin_id: Uuid,
        entity_id: Uuid,
        relation: &str,
    ) -> Result<(), anyhow::Error> {
        // 1. Verify Admin Status (The only time we check u64::MAX)
        let admin = self
            .repo
            .find_by_id(&crate::domain::models::UserId(admin_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Admin not found"))?;

        if admin.permissions != u64::MAX {
            return Err(anyhow::anyhow!(
                "Unauthorized: functionality restricted to Super Admins"
            ));
        }

        // 2. Log Critical Audit Event
        self.repo
            .log_event(
                "break_glass",
                admin_id,
                &entity_id.to_string(),
                serde_json::json!({ "granted_relation": relation }),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Audit failure: {}", e))?;

        // 3. Grant Permission (Tuple)
        // We add a direct tuple: (Entity, relation, AdminID)
        // We assume entity type is generic "node" for now, or we define it.
        // For safety, let's look up entity type or assume generic "break-glass" covers "node".
        // Current constraint: PermissionService check_relation uses "node".
        self.repo
            .add_relation(entity_id, "node", relation, admin_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to grant access: {}", e))?;

        Ok(())
    }

    // --- Management API ---

    /// Lists all explicit permissions for a user:
    /// 1. Direct Tuples (User specific grants)
    /// 2. Group Memberships (Roles)
    pub async fn get_user_explicit_permissions(
        &self,
        user_id: Uuid,
    ) -> Result<serde_json::Value, anyhow::Error> {
        // 1. Get Groups
        let groups = self.repo.get_subject_groups(user_id).await?;

        // 2. Get Direct Tuples
        let direct = self.repo.get_direct_relations(user_id).await?;

        Ok(serde_json::json!({
            "groups": groups,
            "direct_grants": direct // List of (EntityId, EntityType, Relation)
        }))
    }

    pub async fn grant_permission(
        &self,
        user_id: Uuid,
        entity_id: Uuid,
        relation: &str,
    ) -> Result<(), anyhow::Error> {
        // Enforce: only "owner", "editor", "viewer" are valid for now? Or allow flexible?
        // Let's allow flexible for extensibility.
        self.repo
            .add_relation(entity_id, "node", relation, user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Grant failed: {}", e))
    }

    pub async fn revoke_permission(
        &self,
        user_id: Uuid,
        entity_id: Uuid,
        relation: &str,
    ) -> Result<(), anyhow::Error> {
        self.repo
            .remove_relation(entity_id, "node", relation, user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Revoke failed: {}", e))
    }

    // --- Team / Group Management API ---

    pub async fn create_team(&self, name: String, owner_id: Uuid) -> Result<Uuid, anyhow::Error> {
        // 1. Create Group Entity
        let group_id = Uuid::new_v4();
        // Rely on repo specific method or generic logic?
        // Repo has `create_group` method.
        self.repo
            .create_group(group_id, name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create group: {}", e))?;

        // 2. Add Owner Relation (Group, owner, User)
        self.repo
            .add_relation(group_id, "group", "owner", owner_id, "user")
            .await?;

        // 3. Add Member Relation (Group, member, User) - Owner is implicitly member?
        // Let's make it explicit so queries are simpler.
        self.repo
            .add_relation(group_id, "group", "member", owner_id, "user")
            .await?;

        Ok(group_id)
    }

    pub async fn add_team_member(
        &self,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        self.repo
            .add_relation(group_id, "group", "member", user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add member: {}", e))
    }

    pub async fn remove_team_member(
        &self,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        self.repo
            .remove_relation(group_id, "group", "member", user_id, "user")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove member: {}", e))
    }
}
