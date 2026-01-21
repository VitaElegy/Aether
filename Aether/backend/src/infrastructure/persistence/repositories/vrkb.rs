use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{VrkbProject, VrkbSection, VrkbFinding, VrkbAsset};
use crate::domain::ports::{VrkbRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::vrkb::{project, section, finding, asset, project_asset};

#[async_trait]
impl VrkbRepository for PostgresRepository {
    // --- Project ---

    async fn create_project(&self, project_data: VrkbProject) -> Result<Uuid, RepositoryError> {
        let active_model = project::ActiveModel {
            id: Set(project_data.id),
            name: Set(project_data.name),
            repository_url: Set(project_data.repository_url),
            quota_bytes: Set(project_data.quota_bytes),
            settings: Set(project_data.settings),
            created_at: Set(project_data.created_at.into()),
            updated_at: Set(project_data.updated_at.into()),
        };
        project::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(project_data.id)
    }

    async fn get_project(&self, id: &Uuid) -> Result<Option<VrkbProject>, RepositoryError> {
        let model = project::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(model.map(|m| VrkbProject {
            id: m.id,
            name: m.name,
            repository_url: m.repository_url,
            quota_bytes: m.quota_bytes,
            settings: m.settings,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }))
    }

    async fn list_projects(&self) -> Result<Vec<VrkbProject>, RepositoryError> {
        let models = project::Entity::find()
            .order_by_desc(project::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| VrkbProject {
            id: m.id,
            name: m.name,
            repository_url: m.repository_url,
            quota_bytes: m.quota_bytes,
            settings: m.settings,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }).collect())
    }

    // --- Section ---

    async fn create_section(&self, section_data: VrkbSection) -> Result<Uuid, RepositoryError> {
        let active_model = section::ActiveModel {
            id: Set(section_data.id),
            project_id: Set(section_data.project_id),
            title: Set(section_data.title),
            checklist: Set(section_data.checklist),
            created_at: Set(section_data.created_at.into()),
            updated_at: Set(section_data.updated_at.into()),
        };
        section::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(section_data.id)
    }

    async fn list_sections(&self, project_id: &Uuid) -> Result<Vec<VrkbSection>, RepositoryError> {
        let models = section::Entity::find()
            .filter(section::Column::ProjectId.eq(*project_id))
            .order_by_asc(section::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| VrkbSection {
            id: m.id,
            project_id: m.project_id,
            title: m.title,
            checklist: m.checklist,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }).collect())
    }

    // --- Finding ---

    async fn create_finding(&self, finding_data: VrkbFinding) -> Result<Uuid, RepositoryError> {
        let active_model = finding::ActiveModel {
            id: Set(finding_data.id),
            section_id: Set(finding_data.section_id),
            title: Set(finding_data.title),
            status: Set(finding_data.status),
            severity: Set(finding_data.severity),
            content: Set(finding_data.content),
            is_triage: Set(finding_data.is_triage),
            author_id: Set(finding_data.author_id),
            created_at: Set(finding_data.created_at.into()),
            updated_at: Set(finding_data.updated_at.into()),
        };
        finding::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(finding_data.id)
    }

    async fn get_finding(&self, id: &Uuid) -> Result<Option<VrkbFinding>, RepositoryError> {
        let model = finding::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(model.map(|m| VrkbFinding {
            id: m.id,
            section_id: m.section_id,
            title: m.title,
            status: m.status,
            severity: m.severity,
            content: m.content,
            is_triage: m.is_triage,
            author_id: m.author_id,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }))
    }

    async fn list_findings(&self, section_id: Option<Uuid>, project_id: Option<Uuid>) -> Result<Vec<VrkbFinding>, RepositoryError> {
        let mut query = finding::Entity::find();

        if let Some(sid) = section_id {
            query = query.filter(finding::Column::SectionId.eq(sid));
        }
        
        // If filtering by Project ID, we need to join with Section
        if let Some(pid) = project_id {
            query = query
                .join(JoinType::InnerJoin, finding::Relation::Section.def())
                .filter(section::Column::ProjectId.eq(pid));
        }

        let models = query
            .order_by_desc(finding::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| VrkbFinding {
            id: m.id,
            section_id: m.section_id,
            title: m.title,
            status: m.status,
            severity: m.severity,
            content: m.content,
            is_triage: m.is_triage,
            author_id: m.author_id,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }).collect())
    }
    
    async fn update_finding_status(&self, id: &Uuid, status: String) -> Result<(), RepositoryError> {
        let finding_res = finding::Entity::find_by_id(*id).one(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        if let Some(f) = finding_res {
            let mut active: finding::ActiveModel = f.into();
            active.status = Set(status);
            if active.is_triage.as_ref() == &true {
                 // If updating status, assume triage is passed? 
                 // Or require explicit toggle? For now, let's keep it manual.
                 // But typically status changes imply workflow progress.
            }
            active.update(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    // --- Assets ---

    async fn create_asset(&self, asset_data: VrkbAsset) -> Result<Uuid, RepositoryError> {
        let active_model = asset::ActiveModel {
            id: Set(asset_data.id),
            hash: Set(asset_data.hash),
            storage_path: Set(asset_data.storage_path),
            mime_type: Set(asset_data.mime_type),
            size_bytes: Set(asset_data.size_bytes),
            created_at: Set(asset_data.created_at.into()),
        };
        asset::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(asset_data.id)
    }

    async fn get_asset_by_hash(&self, hash: &str) -> Result<Option<VrkbAsset>, RepositoryError> {
        let model = asset::Entity::find()
            .filter(asset::Column::Hash.eq(hash))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(model.map(|m| VrkbAsset {
            id: m.id,
            hash: m.hash,
            storage_path: m.storage_path,
            mime_type: m.mime_type,
            size_bytes: m.size_bytes,
            created_at: m.created_at.with_timezone(&Utc),
        }))
    }

    async fn link_asset_to_project(&self, project_id: Uuid, asset_id: Uuid, virtual_path: String) -> Result<(), RepositoryError> {
        let active_model = project_asset::ActiveModel {
            project_id: Set(project_id),
            asset_id: Set(asset_id),
            virtual_path: Set(virtual_path),
            created_at: Set(Utc::now().into()),
        };
        project_asset::Entity::insert(active_model)
             .on_conflict(
                sea_orm::sea_query::OnConflict::columns([project_asset::Column::ProjectId, project_asset::Column::AssetId])
                    .do_nothing() // Already linked? do nothing. Or update path?
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}
