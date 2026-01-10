use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use std::sync::Arc;
use crate::domain::ports::{PermissionRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{relationship, group};
use crate::domain::models::UserId;

#[async_trait]
impl PermissionRepository for PostgresRepository {
    async fn add_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid, subject_type: &str) -> Result<(), RepositoryError> {
        let rel = relationship::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set(entity_type.to_string()),
            entity_id: Set(entity_id),
            relation: Set(relation.to_string()),
            subject_type: Set(subject_type.to_string()),
            subject_id: Set(subject_id),
            created_at: Set(chrono::Utc::now().into()),
        };
        // Use Insert with OnConflict Do Nothing (to handle idempotency)
        match rel.insert(&self.db).await {
            Ok(_) => Ok(()),
            Err(DbErr::Query(err)) if err.to_string().contains("unique constraint") => Ok(()), // Already exists
            Err(e) => Err(RepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn remove_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid, subject_type: &str) -> Result<(), RepositoryError> {
        let _res = relationship::Entity::delete_many()
            .filter(relationship::Column::EntityType.eq(entity_type))
            .filter(relationship::Column::EntityId.eq(entity_id))
            .filter(relationship::Column::Relation.eq(relation))
            .filter(relationship::Column::SubjectType.eq(subject_type))
            .filter(relationship::Column::SubjectId.eq(subject_id))
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn has_relation(&self, entity_id: Uuid, entity_type: &str, relation: &str, subject_id: Uuid, subject_type: &str) -> Result<bool, RepositoryError> {
        let count = relationship::Entity::find()
            .filter(relationship::Column::EntityType.eq(entity_type))
            .filter(relationship::Column::EntityId.eq(entity_id))
            .filter(relationship::Column::Relation.eq(relation))
            .filter(relationship::Column::SubjectType.eq(subject_type))
            .filter(relationship::Column::SubjectId.eq(subject_id))
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(count > 0)
    }

    async fn get_subject_groups(&self, subject_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        // Find all groups where subject is a 'member'
        let rels = relationship::Entity::find()
            .filter(relationship::Column::EntityType.eq("group"))
            .filter(relationship::Column::Relation.eq("member")) // Hardcoded 'member' relation for groups
            .filter(relationship::Column::SubjectId.eq(subject_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(rels.into_iter().map(|r| r.entity_id).collect())
    }

    async fn get_parents(&self, entity_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        // Find (Entity, parent, Subject) tuples where Entity is current node
        // The "Subject" of a "parent" relation is the Parent itself (Container).
        let rels = relationship::Entity::find()
            .filter(relationship::Column::EntityType.eq("node"))
            .filter(relationship::Column::EntityId.eq(entity_id))
            .filter(relationship::Column::Relation.eq("parent"))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(rels.into_iter().map(|r| r.subject_id).collect())
    }

    async fn create_group(&self, id: Uuid, name: String) -> Result<Uuid, RepositoryError> {
        let grp = group::ActiveModel {
            id: Set(id),
            name: Set(name),
            description: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };
        match grp.insert(&self.db).await {
            Ok(m) => Ok(m.id),
            Err(e) => Err(RepositoryError::DatabaseError(e.to_string())),
        }
    }
}
