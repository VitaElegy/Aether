use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use crate::domain::ports::{AuditRepository, AuditLog, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::audit_log;

#[async_trait]
impl AuditRepository for PostgresRepository {
    async fn log_event(&self, action: &str, actor_id: Uuid, target: &str, details: serde_json::Value) -> Result<(), RepositoryError> {
        let log = audit_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            action: Set(action.to_string()),
            actor_id: Set(actor_id),
            target_resource: Set(target.to_string()),
            details: Set(details),
            created_at: Set(chrono::Utc::now().into()),
        };
        
        log.insert(&self.db).await
            .map(|_| ())
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn get_logs_by_target(&self, target: &str) -> Result<Vec<AuditLog>, RepositoryError> {
        let logs = audit_log::Entity::find()
            .filter(audit_log::Column::TargetResource.contains(target)) // Loose matching? Or exact. 
            // "target resource" might be "kb:uuid".
            // Let's do exact match or starts_with for resource families
            .order_by_desc(audit_log::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(logs.into_iter().map(|l| AuditLog {
            id: l.id,
            action: l.action,
            actor_id: l.actor_id,
            target_resource: l.target_resource,
            details: l.details,
            created_at: l.created_at.into(),
        }).collect())
    }

    async fn get_logs_by_actor(&self, actor_id: Uuid) -> Result<Vec<AuditLog>, RepositoryError> {
        let logs = audit_log::Entity::find()
            .filter(audit_log::Column::ActorId.eq(actor_id))
            .order_by_desc(audit_log::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(logs.into_iter().map(|l| AuditLog {
            id: l.id,
            action: l.action,
            actor_id: l.actor_id,
            target_resource: l.target_resource,
            details: l.details,
            created_at: l.created_at.into(),
        }).collect())
    }
}
