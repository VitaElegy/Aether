use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{VrkbProject, VrkbSection, VrkbFinding, VrkbAsset, VrkbMember, VrkbSpec, VrkbDoc};
use crate::domain::ports::{VrkbRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::vrkb::{project, section, finding, asset, project_asset, member, spec, doc};

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

    async fn list_project_assets(&self, project_id: &Uuid) -> Result<Vec<VrkbAsset>, RepositoryError> {
        // We need to join project_asset and asset
        let assets = asset::Entity::find()
            .join(JoinType::InnerJoin, project_asset::Relation::Asset.def().rev())
            .filter(project_asset::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(assets.into_iter().map(|m| VrkbAsset {
            id: m.id,
            hash: m.hash,
            storage_path: m.storage_path,
            mime_type: m.mime_type,
            size_bytes: m.size_bytes,
            created_at: m.created_at.with_timezone(&Utc),
        }).collect())
    }

    async fn delete_asset(&self, id: &Uuid) -> Result<(), RepositoryError> {
        asset::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    // --- Members ---

    async fn add_member(&self, member_data: VrkbMember) -> Result<(), RepositoryError> {
        let active_model = member::ActiveModel {
            project_id: Set(member_data.project_id),
            user_id: Set(member_data.user_id),
            role: Set(member_data.role),
            joined_at: Set(member_data.joined_at.into()),
        };
        member::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([member::Column::ProjectId, member::Column::UserId])
                    .update_column(member::Column::Role) // Update role if exists
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn remove_member(&self, project_id: &Uuid, user_id: &Uuid) -> Result<(), RepositoryError> {
        member::Entity::delete_many()
            .filter(member::Column::ProjectId.eq(*project_id))
            .filter(member::Column::UserId.eq(*user_id))
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn list_members(&self, project_id: &Uuid) -> Result<Vec<VrkbMember>, RepositoryError> {
        let models = member::Entity::find()
            .filter(member::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        // TODO: Join with User table to fill user details
        
        Ok(models.into_iter().map(|m| VrkbMember {
            project_id: m.project_id,
            user_id: m.user_id,
            role: m.role,
            joined_at: m.joined_at.with_timezone(&Utc),
            user: None,
        }).collect())
    }

    async fn update_member_role(&self, project_id: &Uuid, user_id: &Uuid, role: String) -> Result<(), RepositoryError> {
         // Re-using add_member since we set upsert logic there
         let member = VrkbMember {
             project_id: *project_id,
             user_id: *user_id,
             role,
             joined_at: Utc::now(),
             user: None
         };
         self.add_member(member).await
    }

    // --- Specs ---

    async fn get_specs(&self, project_id: &Uuid) -> Result<Vec<VrkbSpec>, RepositoryError> {
        let models = spec::Entity::find()
            .filter(spec::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| VrkbSpec {
            id: m.id,
            project_id: m.project_id,
            title: m.title,
            content: m.content,
            version: m.version,
            updated_at: m.updated_at.with_timezone(&Utc),
        }).collect())
    }

    async fn save_spec(&self, spec_data: VrkbSpec) -> Result<Uuid, RepositoryError> {
        let active_model = spec::ActiveModel {
            id: Set(spec_data.id),
            project_id: Set(spec_data.project_id),
            title: Set(spec_data.title),
            content: Set(spec_data.content),
            version: Set(spec_data.version),
            updated_at: Set(spec_data.updated_at.into()),
        };
        
        spec::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(spec::Column::Id)
                    .update_columns([spec::Column::Title, spec::Column::Content, spec::Column::Version, spec::Column::UpdatedAt])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(spec_data.id)
    }

    // --- Docs ---

    async fn create_doc(&self, doc_data: VrkbDoc) -> Result<Uuid, RepositoryError> {
        let active_model = doc::ActiveModel {
            id: Set(doc_data.id),
            project_id: Set(doc_data.project_id),
            title: Set(doc_data.title),
            content: Set(doc_data.content),
            parent_id: Set(doc_data.parent_id),
            author_id: Set(doc_data.author_id),
            created_at: Set(doc_data.created_at.into()),
            updated_at: Set(doc_data.updated_at.into()),
            deleted_at: Set(doc_data.deleted_at.map(|d| d.into())),
        };
        doc::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(doc_data.id)
    }

    async fn get_doc(&self, id: &Uuid) -> Result<Option<VrkbDoc>, RepositoryError> {
        let model = doc::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(model.map(|m| VrkbDoc {
            id: m.id,
            project_id: m.project_id,
            title: m.title,
            content: m.content,
            parent_id: m.parent_id,
            author_id: m.author_id,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
            deleted_at: m.deleted_at.map(|d| d.with_timezone(&Utc)),
        }))
    }

    async fn update_doc(&self, doc_data: VrkbDoc) -> Result<(), RepositoryError> {
        let active_model = doc::ActiveModel {
            id: Set(doc_data.id),
            project_id: Set(doc_data.project_id),
            title: Set(doc_data.title),
            content: Set(doc_data.content),
            parent_id: Set(doc_data.parent_id),
            author_id: Set(doc_data.author_id),
            // created_at: Set(doc_data.created_at.into()), // Don't update created_at?
            updated_at: Set(doc_data.updated_at.into()),
            deleted_at: Set(doc_data.deleted_at.map(|d| d.into())),
            ..Default::default() // Important strictly for partial updates if we were doing find first, but here we replace all fields we set.
        };
        
        // Use update method which expects model to result from find
        // Or clearer: find -> update.
        // But for upsert-like behavior we can just do insert ... on conflict update
        
        // Let's stick to update logic:
         doc::Entity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(())
    }

    async fn delete_doc(&self, id: &Uuid) -> Result<(), RepositoryError> {
        // Soft Delete
        let doc_res = doc::Entity::find_by_id(*id).one(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        if let Some(d) = doc_res {
            let mut active: doc::ActiveModel = d.into();
            active.deleted_at = Set(Some(Utc::now().into()));
            active.update(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn list_docs(&self, project_id: &Uuid) -> Result<Vec<VrkbDoc>, RepositoryError> {
        let models = doc::Entity::find()
            .filter(doc::Column::ProjectId.eq(*project_id))
            .filter(doc::Column::DeletedAt.is_null()) // Filter out deleted
            .order_by_desc(doc::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| VrkbDoc {
            id: m.id,
            project_id: m.project_id,
            title: m.title,
            content: m.content,
            parent_id: m.parent_id,
            author_id: m.author_id,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
            deleted_at: m.deleted_at.map(|d| d.with_timezone(&Utc)),
        }).collect())
    }

    // --- Trash Management ---

    async fn list_trash(&self, project_id: &Uuid) -> Result<Vec<VrkbDoc>, RepositoryError> {
         let models = doc::Entity::find()
            .filter(doc::Column::ProjectId.eq(*project_id))
            .filter(doc::Column::DeletedAt.is_not_null()) // Only deleted
            .order_by_desc(doc::Column::DeletedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(models.into_iter().map(|m| VrkbDoc {
            id: m.id,
            project_id: m.project_id,
            title: m.title,
            content: m.content,
            parent_id: m.parent_id,
            author_id: m.author_id,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
            deleted_at: m.deleted_at.map(|d| d.with_timezone(&Utc)),
        }).collect())
    }

    async fn restore_doc(&self, id: &Uuid) -> Result<(), RepositoryError> {
        let doc_res = doc::Entity::find_by_id(*id).one(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        if let Some(d) = doc_res {
            let mut active: doc::ActiveModel = d.into();
            active.deleted_at = Set(None); // Clear deleted_at
            active.update(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn permanent_delete_doc(&self, id: &Uuid) -> Result<(), RepositoryError> {
        doc::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn cleanup_trash(&self, days: i64) -> Result<u64, RepositoryError> {
        // Should execute a raw SQL delete for efficiency or use a complex filter
        // "DELETE FROM vrkb_docs WHERE deleted_at < NOW() - INTERVAL 'days' DAYS"
        
        let time_threshold = Utc::now() - chrono::Duration::days(days);
        
        let res = doc::Entity::delete_many()
            .filter(doc::Column::DeletedAt.lt(time_threshold))
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        Ok(res.rows_affected)
    }

    async fn get_project_stats(&self, project_id: &Uuid) -> Result<crate::domain::models::VrkbStats, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::finding;
        use crate::domain::models::{VrkbMetrics, VrkbModuleStat, VrkbHeatmapItem};
        
        // 1. Fetch all findings (joined with sections to filter by project)
        let findings = finding::Entity::find()
             .join(JoinType::InnerJoin, finding::Relation::Section.def())
             .filter(section::Column::ProjectId.eq(*project_id))
             .all(&self.db)
             .await
             .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Calculate Metrics
        let total = findings.len() as i64;
        let mut critical = 0;
        let mut triage = 0;
        let mut fixed = 0;
        
        for f in &findings {
            match f.status.as_str() {
                "Verified" => fixed += 1,
                "Triage" => triage += 1,
                _ => {}
            }
            if f.severity == "Critical" {
                critical += 1;
            }
            if f.is_triage {
                 triage += 1; // Assuming overlap or distinct definition
            }
        }
        
        let metrics = VrkbMetrics { total, critical, triage, fixed };

        // 3. Module Stats (from sections)
        let sections = section::Entity::find()
             .filter(section::Column::ProjectId.eq(*project_id))
             .all(&self.db)
             .await
             .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut modules = Vec::new();
        // Naive module generation from sections
        for s in sections {
             // Find bugs for this section
             let section_bugs = findings.iter().filter(|f| f.section_id == s.id).count() as i64;
             modules.push(VrkbModuleStat {
                 name: s.title,
                 status: "Active".to_string(), // Placeholder
                 progress: 50, // Placeholder
                 bugs: section_bugs,
                 last_audit: "Today".to_string(), // Placeholder
             });
        }
        
        // 4. Heatmap (Placeholder for now, could be derived from findings path metadata if stored)
        let heatmap = vec![
             VrkbHeatmapItem { path: "src".to_string(), name: "src".to_string(), r#type: "folder".to_string(), level: 0, vulns: total / 2 },
             VrkbHeatmapItem { path: "src/main.rs".to_string(), name: "main.rs".to_string(), r#type: "file".to_string(), level: 1, vulns: total / 2 },
        ];
        
        Ok(crate::domain::models::VrkbStats { metrics, modules, heatmap })
    }
}
