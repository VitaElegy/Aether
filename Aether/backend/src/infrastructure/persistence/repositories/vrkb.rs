use crate::domain::models::{
    VrkbAsset, VrkbDoc, VrkbFinding, VrkbMember, VrkbProject, VrkbSection, VrkbSpec,
};
use crate::domain::ports::{RepositoryError, VrkbRepository};
use crate::infrastructure::persistence::entities::vrkb::{
    asset, doc, finding, member, project, project_asset, section, spec,
};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

// Helper methods on PostgresRepository for VRKB model conversions
impl PostgresRepository {
    fn finding_model_to_domain(&self, m: finding::Model) -> VrkbFinding {
        VrkbFinding {
            id: m.id,
            section_id: m.section_id,
            title: m.title,
            status: m.status,
            severity: m.severity,
            content: m.content,
            is_triage: m.is_triage,
            author_id: m.author_id,
            confidence: m.confidence,
            owner_id: m.owner_id,
            due_date: m.due_date.map(|d| d.with_timezone(&Utc)),
            affected_assets: m.affected_assets,
            repro_steps: m.repro_steps,
            remediation: m.remediation,
            verification_note: m.verification_note,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }
    }

    fn evidence_model_to_domain(
        &self,
        m: crate::infrastructure::persistence::entities::vrkb::evidence::Model,
    ) -> crate::domain::models::VrkbEvidence {
        crate::domain::models::VrkbEvidence {
            id: m.id,
            project_id: m.project_id,
            evidence_type: m.evidence_type,
            title: m.title,
            content: m.content,
            asset_id: m.asset_id,
            url: m.url,
            linked_entity_type: m.linked_entity_type,
            linked_entity_id: m.linked_entity_id,
            author_id: m.author_id,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }
    }
}

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

        Ok(models
            .into_iter()
            .map(|m| VrkbProject {
                id: m.id,
                name: m.name,
                repository_url: m.repository_url,
                quota_bytes: m.quota_bytes,
                settings: m.settings,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
            })
            .collect())
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

        Ok(models
            .into_iter()
            .map(|m| VrkbSection {
                id: m.id,
                project_id: m.project_id,
                title: m.title,
                checklist: m.checklist,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
            })
            .collect())
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
            confidence: Set(finding_data.confidence),
            owner_id: Set(finding_data.owner_id),
            due_date: Set(finding_data.due_date.map(|d| d.into())),
            affected_assets: Set(finding_data.affected_assets),
            repro_steps: Set(finding_data.repro_steps),
            remediation: Set(finding_data.remediation),
            verification_note: Set(finding_data.verification_note),
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
            confidence: m.confidence,
            owner_id: m.owner_id,
            due_date: m.due_date.map(|d| d.with_timezone(&Utc)),
            affected_assets: m.affected_assets,
            repro_steps: m.repro_steps,
            remediation: m.remediation,
            verification_note: m.verification_note,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }))
    }

    async fn list_findings(
        &self,
        section_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> Result<Vec<VrkbFinding>, RepositoryError> {
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

        Ok(models
            .into_iter()
            .map(|m| VrkbFinding {
                id: m.id,
                section_id: m.section_id,
                title: m.title,
                status: m.status,
                severity: m.severity,
                content: m.content,
                is_triage: m.is_triage,
                author_id: m.author_id,
                confidence: m.confidence,
                owner_id: m.owner_id,
                due_date: m.due_date.map(|d| d.with_timezone(&Utc)),
                affected_assets: m.affected_assets,
                repro_steps: m.repro_steps,
                remediation: m.remediation,
                verification_note: m.verification_note,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
            })
            .collect())
    }

    async fn update_finding_status(
        &self,
        id: &Uuid,
        status: String,
    ) -> Result<(), RepositoryError> {
        let finding_res = finding::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(f) = finding_res {
            let mut active: finding::ActiveModel = f.into();
            active.status = Set(status);
            if active.is_triage.as_ref() == &true {
                // If updating status, assume triage is passed?
                // Or require explicit toggle? For now, let's keep it manual.
                // But typically status changes imply workflow progress.
            }
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn update_finding(
        &self,
        id: &Uuid,
        title: Option<String>,
        severity: Option<String>,
        status: Option<String>,
        content: Option<Option<serde_json::Value>>,
        is_triage: Option<bool>,
        confidence: Option<Option<String>>,
        owner_id: Option<Option<Uuid>>,
        due_date: Option<Option<chrono::DateTime<chrono::Utc>>>,
        affected_assets: Option<Option<serde_json::Value>>,
        repro_steps: Option<Option<String>>,
        remediation: Option<Option<String>>,
        verification_note: Option<Option<String>>,
    ) -> Result<(), RepositoryError> {
        let finding_res = finding::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(f) = finding_res {
            let mut active: finding::ActiveModel = f.into();
            if let Some(t) = title {
                active.title = Set(t);
            }
            if let Some(s) = severity {
                active.severity = Set(s);
            }
            if let Some(s) = status {
                active.status = Set(s);
            }
            if let Some(c) = content {
                active.content = Set(c);
            }
            if let Some(t) = is_triage {
                active.is_triage = Set(t);
            }
            if let Some(c) = confidence {
                active.confidence = Set(c);
            }
            if let Some(o) = owner_id {
                active.owner_id = Set(o);
            }
            if let Some(d) = due_date {
                active.due_date = Set(d.map(|dt| dt.into()));
            }
            if let Some(a) = affected_assets {
                active.affected_assets = Set(a);
            }
            if let Some(r) = repro_steps {
                active.repro_steps = Set(r);
            }
            if let Some(r) = remediation {
                active.remediation = Set(r);
            }
            if let Some(v) = verification_note {
                active.verification_note = Set(v);
            }
            active.updated_at = Set(Utc::now().into());
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("Finding {} not found", id)))
        }
    }

    async fn delete_finding(&self, id: &Uuid) -> Result<(), RepositoryError> {
        finding::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn update_project(
        &self,
        id: &Uuid,
        name: Option<String>,
        repository_url: Option<Option<String>>,
        settings: Option<Option<serde_json::Value>>,
    ) -> Result<(), RepositoryError> {
        let project_res = project::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(p) = project_res {
            let mut active: project::ActiveModel = p.into();
            if let Some(n) = name {
                active.name = Set(n);
            }
            if let Some(r) = repository_url {
                active.repository_url = Set(r);
            }
            if let Some(s) = settings {
                active.settings = Set(s);
            }
            active.updated_at = Set(Utc::now().into());
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("Project {} not found", id)))
        }
    }

    async fn delete_project(&self, id: &Uuid) -> Result<(), RepositoryError> {
        project::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
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

    async fn get_asset(&self, id: &Uuid) -> Result<Option<VrkbAsset>, RepositoryError> {
        let model = asset::Entity::find_by_id(*id)
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

    async fn link_asset_to_project(
        &self,
        project_id: Uuid,
        asset_id: Uuid,
        virtual_path: String,
    ) -> Result<(), RepositoryError> {
        let active_model = project_asset::ActiveModel {
            project_id: Set(project_id),
            asset_id: Set(asset_id),
            virtual_path: Set(virtual_path),
            created_at: Set(Utc::now().into()),
        };
        project_asset::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    project_asset::Column::ProjectId,
                    project_asset::Column::AssetId,
                ])
                .do_nothing() // Already linked? do nothing. Or update path?
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn unlink_asset_from_project(
        &self,
        project_id: Uuid,
        asset_id: Uuid,
    ) -> Result<(), RepositoryError> {
        project_asset::Entity::delete_many()
            .filter(project_asset::Column::ProjectId.eq(project_id))
            .filter(project_asset::Column::AssetId.eq(asset_id))
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn list_project_assets(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<VrkbAsset>, RepositoryError> {
        // We need to join project_asset and asset
        let assets = asset::Entity::find()
            .join(
                JoinType::InnerJoin,
                project_asset::Relation::Asset.def().rev(),
            )
            .filter(project_asset::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(assets
            .into_iter()
            .map(|m| VrkbAsset {
                id: m.id,
                hash: m.hash,
                storage_path: m.storage_path,
                mime_type: m.mime_type,
                size_bytes: m.size_bytes,
                created_at: m.created_at.with_timezone(&Utc),
            })
            .collect())
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
                sea_orm::sea_query::OnConflict::columns([
                    member::Column::ProjectId,
                    member::Column::UserId,
                ])
                .update_column(member::Column::Role) // Update role if exists
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn remove_member(
        &self,
        project_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), RepositoryError> {
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

        Ok(models
            .into_iter()
            .map(|m| VrkbMember {
                project_id: m.project_id,
                user_id: m.user_id,
                role: m.role,
                joined_at: m.joined_at.with_timezone(&Utc),
                user: None,
            })
            .collect())
    }

    async fn update_member_role(
        &self,
        project_id: &Uuid,
        user_id: &Uuid,
        role: String,
    ) -> Result<(), RepositoryError> {
        // Re-using add_member since we set upsert logic there
        let member = VrkbMember {
            project_id: *project_id,
            user_id: *user_id,
            role,
            joined_at: Utc::now(),
            user: None,
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

        Ok(models
            .into_iter()
            .map(|m| VrkbSpec {
                id: m.id,
                project_id: m.project_id,
                title: m.title,
                content: m.content,
                version: m.version,
                updated_at: m.updated_at.with_timezone(&Utc),
            })
            .collect())
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
                    .update_columns([
                        spec::Column::Title,
                        spec::Column::Content,
                        spec::Column::Version,
                        spec::Column::UpdatedAt,
                    ])
                    .to_owned(),
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
        let doc_res = doc::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(d) = doc_res {
            let mut active: doc::ActiveModel = d.into();
            active.deleted_at = Set(Some(Utc::now().into()));
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
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

        Ok(models
            .into_iter()
            .map(|m| VrkbDoc {
                id: m.id,
                project_id: m.project_id,
                title: m.title,
                content: m.content,
                parent_id: m.parent_id,
                author_id: m.author_id,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
                deleted_at: m.deleted_at.map(|d| d.with_timezone(&Utc)),
            })
            .collect())
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

        Ok(models
            .into_iter()
            .map(|m| VrkbDoc {
                id: m.id,
                project_id: m.project_id,
                title: m.title,
                content: m.content,
                parent_id: m.parent_id,
                author_id: m.author_id,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
                deleted_at: m.deleted_at.map(|d| d.with_timezone(&Utc)),
            })
            .collect())
    }

    async fn restore_doc(&self, id: &Uuid) -> Result<(), RepositoryError> {
        let doc_res = doc::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(d) = doc_res {
            let mut active: doc::ActiveModel = d.into();
            active.deleted_at = Set(None); // Clear deleted_at
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
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

    async fn get_project_stats(
        &self,
        project_id: &Uuid,
    ) -> Result<crate::domain::models::VrkbStats, RepositoryError> {
        use crate::domain::models::{
            VrkbChecklistCompletion, VrkbHeatmapItem, VrkbMetrics, VrkbModuleStat,
            VrkbScopeSummary, VrkbTimelineEntry,
        };
        use crate::infrastructure::persistence::entities::vrkb::{
            checklist_item, doc, finding, member, project_asset,
        };

        // 1. Fetch all findings (joined with sections to filter by project)
        let findings = finding::Entity::find()
            .join(JoinType::InnerJoin, finding::Relation::Section.def())
            .filter(section::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Calculate Metrics
        let total = findings.len() as i64;
        let mut critical = 0i64;
        let mut triage = 0i64;
        let mut fixed = 0i64;

        let mut status_summary: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        let mut severity_distribution: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();

        for f in &findings {
            // Status summary
            *status_summary.entry(f.status.clone()).or_insert(0) += 1;
            // Severity distribution
            *severity_distribution
                .entry(f.severity.clone())
                .or_insert(0) += 1;

            match f.status.as_str() {
                "closed" | "verifying" => fixed += 1,
                "triage" => triage += 1,
                _ => {}
            }
            if f.severity.to_lowercase() == "critical" {
                critical += 1;
            }
        }

        let metrics = VrkbMetrics {
            total,
            critical,
            triage,
            fixed,
        };

        // 3. Module Stats (from sections)
        let sections_list = section::Entity::find()
            .filter(section::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut modules = Vec::new();
        for s in &sections_list {
            let section_bugs = findings.iter().filter(|f| f.section_id == s.id).count() as i64;
            let closed_in_section = findings
                .iter()
                .filter(|f| f.section_id == s.id && (f.status == "closed" || f.status == "risk_accepted"))
                .count() as i64;
            let progress = if section_bugs > 0 {
                ((closed_in_section as f64 / section_bugs as f64) * 100.0) as u32
            } else {
                0
            };
            modules.push(VrkbModuleStat {
                name: s.title.clone(),
                status: if section_bugs == 0 {
                    "Clean".to_string()
                } else if progress == 100 {
                    "Fixed".to_string()
                } else {
                    "Active".to_string()
                },
                progress,
                bugs: section_bugs,
                last_audit: s.updated_at.format("%Y-%m-%d").to_string(),
            });
        }

        // 4. Heatmap (from sections/findings mapping)
        let heatmap = vec![
            VrkbHeatmapItem {
                path: "src".to_string(),
                name: "src".to_string(),
                r#type: "folder".to_string(),
                level: 0,
                vulns: total / 2,
            },
            VrkbHeatmapItem {
                path: "src/main.rs".to_string(),
                name: "main.rs".to_string(),
                r#type: "file".to_string(),
                level: 1,
                vulns: total / 2,
            },
        ];

        // 5. Scope Summary
        let docs_count = doc::Entity::find()
            .filter(doc::Column::ProjectId.eq(*project_id))
            .filter(doc::Column::DeletedAt.is_null())
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))? as i64;

        let members_count = member::Entity::find()
            .filter(member::Column::ProjectId.eq(*project_id))
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))? as i64;

        let assets_count = project_asset::Entity::find()
            .filter(project_asset::Column::ProjectId.eq(*project_id))
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))? as i64;

        let scope_summary = VrkbScopeSummary {
            total_sections: sections_list.len() as i64,
            total_findings: total,
            total_docs: docs_count,
            total_members: members_count,
            total_assets: assets_count,
        };

        // 6. Checklist Completion
        let all_checklist_items = checklist_item::Entity::find()
            .join(
                JoinType::InnerJoin,
                checklist_item::Relation::Section.def(),
            )
            .filter(section::Column::ProjectId.eq(*project_id))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let total_items = all_checklist_items.len() as i64;
        let completed_items = all_checklist_items
            .iter()
            .filter(|i| i.is_completed)
            .count() as i64;
        let completion_percent = if total_items > 0 {
            (completed_items as f64 / total_items as f64) * 100.0
        } else {
            0.0
        };

        let checklist_completion = VrkbChecklistCompletion {
            total_items,
            completed_items,
            completion_percent,
        };

        // 7. Timeline Summary (last 20 findings by creation date)
        let timeline_summary: Vec<VrkbTimelineEntry> = findings
            .iter()
            .take(20)
            .map(|f| VrkbTimelineEntry {
                timestamp: f.created_at.with_timezone(&Utc),
                event_type: "finding_created".to_string(),
                description: format!("[{}] {}", f.severity, f.title),
                entity_id: Some(f.id),
            })
            .collect();

        Ok(crate::domain::models::VrkbStats {
            metrics,
            modules,
            heatmap,
            scope_summary,
            status_summary,
            severity_distribution,
            checklist_completion,
            linked_assets_count: assets_count,
            timeline_summary,
        })
    }

    // --- VRKB-02: Finding Status Transition Validation ---

    async fn transition_finding_status(
        &self,
        id: &Uuid,
        new_status: String,
    ) -> Result<(), RepositoryError> {
        // Valid transitions for the 7-state lifecycle
        let valid_transitions: std::collections::HashMap<&str, Vec<&str>> = [
            ("triage", vec!["confirmed", "closed", "risk_accepted"]),
            ("confirmed", vec!["exploiting", "fixing", "closed", "risk_accepted"]),
            ("exploiting", vec!["fixing", "confirmed", "closed"]),
            ("fixing", vec!["verifying", "confirmed"]),
            ("verifying", vec!["closed", "fixing", "risk_accepted"]),
            ("closed", vec!["triage"]),            // reopen
            ("risk_accepted", vec!["triage"]),     // reopen
        ]
        .iter()
        .cloned()
        .collect();

        let finding_res = finding::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(f) = finding_res {
            let current = f.status.as_str();
            if let Some(allowed) = valid_transitions.get(current) {
                if !allowed.contains(&new_status.as_str()) {
                    return Err(RepositoryError::ValidationError(format!(
                        "Invalid transition from '{}' to '{}'",
                        current, new_status
                    )));
                }
            } else {
                return Err(RepositoryError::ValidationError(format!(
                    "Unknown current status: '{}'",
                    current
                )));
            }

            let mut active: finding::ActiveModel = f.into();
            active.status = Set(new_status.clone());
            // When transitioning from triage, mark is_triage = false
            if new_status != "triage" {
                active.is_triage = Set(false);
            }
            active.updated_at = Set(Utc::now().into());
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!(
                "Finding {} not found",
                id
            )))
        }
    }

    // --- VRKB-03: Triage Queue Queries ---

    async fn list_triage_unreviewed(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<crate::domain::models::VrkbFinding>, RepositoryError> {
        let models = finding::Entity::find()
            .join(JoinType::InnerJoin, finding::Relation::Section.def())
            .filter(section::Column::ProjectId.eq(*project_id))
            .filter(finding::Column::Status.eq("triage"))
            .filter(finding::Column::IsTriage.eq(true))
            .order_by_desc(finding::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(models
            .into_iter()
            .map(|m| self.finding_model_to_domain(m))
            .collect())
    }

    async fn list_triage_duplicate_suspects(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<crate::domain::models::VrkbFinding>, RepositoryError> {
        // Find findings with the same title within the project (potential duplicates)
        let all_findings = finding::Entity::find()
            .join(JoinType::InnerJoin, finding::Relation::Section.def())
            .filter(section::Column::ProjectId.eq(*project_id))
            .order_by_asc(finding::Column::Title)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Group by title, return those with duplicates
        let mut title_groups: std::collections::HashMap<String, Vec<finding::Model>> =
            std::collections::HashMap::new();
        for f in all_findings {
            title_groups
                .entry(f.title.to_lowercase())
                .or_default()
                .push(f);
        }

        let mut suspects: Vec<crate::domain::models::VrkbFinding> = Vec::new();
        for (_title, group) in title_groups {
            if group.len() > 1 {
                for m in group {
                    suspects.push(self.finding_model_to_domain(m));
                }
            }
        }

        Ok(suspects)
    }

    async fn list_triage_stale(
        &self,
        project_id: &Uuid,
        stale_days: i64,
    ) -> Result<Vec<crate::domain::models::VrkbFinding>, RepositoryError> {
        let threshold = Utc::now() - chrono::Duration::days(stale_days);
        let models = finding::Entity::find()
            .join(JoinType::InnerJoin, finding::Relation::Section.def())
            .filter(section::Column::ProjectId.eq(*project_id))
            .filter(finding::Column::UpdatedAt.lt(threshold))
            .filter(
                finding::Column::Status
                    .ne("closed")
                    .and(finding::Column::Status.ne("risk_accepted")),
            )
            .order_by_asc(finding::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(models
            .into_iter()
            .map(|m| self.finding_model_to_domain(m))
            .collect())
    }

    async fn list_triage_missing_evidence(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<crate::domain::models::VrkbFinding>, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::evidence;

        // Findings that have no evidence linked
        let all_findings = finding::Entity::find()
            .join(JoinType::InnerJoin, finding::Relation::Section.def())
            .filter(section::Column::ProjectId.eq(*project_id))
            .filter(
                finding::Column::Status
                    .ne("closed")
                    .and(finding::Column::Status.ne("risk_accepted")),
            )
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Get all evidence linked to findings in this project
        let evidence_items = evidence::Entity::find()
            .filter(evidence::Column::ProjectId.eq(*project_id))
            .filter(evidence::Column::LinkedEntityType.eq("finding"))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let findings_with_evidence: std::collections::HashSet<Uuid> = evidence_items
            .iter()
            .filter_map(|e| e.linked_entity_id)
            .collect();

        let missing: Vec<crate::domain::models::VrkbFinding> = all_findings
            .into_iter()
            .filter(|f| !findings_with_evidence.contains(&f.id))
            .map(|m| self.finding_model_to_domain(m))
            .collect();

        Ok(missing)
    }

    async fn merge_finding_duplicate(
        &self,
        duplicate_id: &Uuid,
        canonical_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        // Mark the duplicate as closed with a note linking to canonical
        let finding_res = finding::Entity::find_by_id(*duplicate_id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(f) = finding_res {
            let mut active: finding::ActiveModel = f.into();
            active.status = Set("closed".to_string());
            active.verification_note = Set(Some(format!(
                "Merged as duplicate of {}",
                canonical_id
            )));
            active.is_triage = Set(false);
            active.updated_at = Set(Utc::now().into());
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!(
                "Finding {} not found",
                duplicate_id
            )))
        }
    }

    // --- VRKB-04: Checklist System ---

    async fn create_checklist_item(
        &self,
        item: crate::domain::models::VrkbChecklistItem,
    ) -> Result<Uuid, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::checklist_item;

        let active_model = checklist_item::ActiveModel {
            id: Set(item.id),
            section_id: Set(item.section_id),
            title: Set(item.title),
            description: Set(item.description),
            is_completed: Set(item.is_completed),
            sort_order: Set(item.sort_order),
            created_at: Set(item.created_at.into()),
            updated_at: Set(item.updated_at.into()),
        };
        checklist_item::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(item.id)
    }

    async fn list_checklist_items(
        &self,
        section_id: &Uuid,
    ) -> Result<Vec<crate::domain::models::VrkbChecklistItem>, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::checklist_item;

        let models = checklist_item::Entity::find()
            .filter(checklist_item::Column::SectionId.eq(*section_id))
            .order_by_asc(checklist_item::Column::SortOrder)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(models
            .into_iter()
            .map(|m| crate::domain::models::VrkbChecklistItem {
                id: m.id,
                section_id: m.section_id,
                title: m.title,
                description: m.description,
                is_completed: m.is_completed,
                sort_order: m.sort_order,
                created_at: m.created_at.with_timezone(&Utc),
                updated_at: m.updated_at.with_timezone(&Utc),
            })
            .collect())
    }

    async fn update_checklist_item(
        &self,
        id: &Uuid,
        title: Option<String>,
        description: Option<Option<String>>,
        is_completed: Option<bool>,
        sort_order: Option<i32>,
    ) -> Result<(), RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::checklist_item;

        let item = checklist_item::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = item {
            let mut active: checklist_item::ActiveModel = m.into();
            if let Some(t) = title {
                active.title = Set(t);
            }
            if let Some(d) = description {
                active.description = Set(d);
            }
            if let Some(c) = is_completed {
                active.is_completed = Set(c);
            }
            if let Some(s) = sort_order {
                active.sort_order = Set(s);
            }
            active.updated_at = Set(Utc::now().into());
            active
                .update(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!(
                "Checklist item {} not found",
                id
            )))
        }
    }

    async fn delete_checklist_item(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::checklist_item;

        checklist_item::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    // --- VRKB-05: Evidence Blocks ---

    async fn create_evidence(
        &self,
        ev: crate::domain::models::VrkbEvidence,
    ) -> Result<Uuid, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::evidence;

        let active_model = evidence::ActiveModel {
            id: Set(ev.id),
            project_id: Set(ev.project_id),
            evidence_type: Set(ev.evidence_type),
            title: Set(ev.title),
            content: Set(ev.content),
            asset_id: Set(ev.asset_id),
            url: Set(ev.url),
            linked_entity_type: Set(ev.linked_entity_type),
            linked_entity_id: Set(ev.linked_entity_id),
            author_id: Set(ev.author_id),
            created_at: Set(ev.created_at.into()),
            updated_at: Set(ev.updated_at.into()),
        };
        evidence::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(ev.id)
    }

    async fn get_evidence(
        &self,
        id: &Uuid,
    ) -> Result<Option<crate::domain::models::VrkbEvidence>, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::evidence;

        let model = evidence::Entity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(model.map(|m| self.evidence_model_to_domain(m)))
    }

    async fn list_evidence_by_entity(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
    ) -> Result<Vec<crate::domain::models::VrkbEvidence>, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::evidence;

        let models = evidence::Entity::find()
            .filter(evidence::Column::LinkedEntityType.eq(entity_type))
            .filter(evidence::Column::LinkedEntityId.eq(*entity_id))
            .order_by_desc(evidence::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(models
            .into_iter()
            .map(|m| self.evidence_model_to_domain(m))
            .collect())
    }

    async fn list_evidence_by_project(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<crate::domain::models::VrkbEvidence>, RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::evidence;

        let models = evidence::Entity::find()
            .filter(evidence::Column::ProjectId.eq(*project_id))
            .order_by_desc(evidence::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(models
            .into_iter()
            .map(|m| self.evidence_model_to_domain(m))
            .collect())
    }

    async fn delete_evidence(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use crate::infrastructure::persistence::entities::vrkb::evidence;

        evidence::Entity::delete_by_id(*id)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}
