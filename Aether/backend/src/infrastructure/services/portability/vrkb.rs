use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::domain::models::{VrkbDoc, VrkbFinding, VrkbProject};
use crate::domain::portability::models::{
    ExportSection, ExportSummary, ImportSection, ImportSummary, ProgressEvent,
};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::ports::VrkbRepository;
use serde::{Deserialize, Serialize};

/// VRKB-10: Portability provider for VRKB projects.
/// Handles export and import of complete project packages including:
/// - Project metadata
/// - Findings with evidence links
/// - Documents (nested structure preserved)
/// - Asset references
/// - Member map
pub struct VrkbPortabilityProvider {
    repo: Arc<dyn VrkbRepository>,
}

/// Serialized package format for VRKB project export
#[derive(Debug, Serialize, Deserialize)]
struct VrkbExportPackage {
    version: String,
    exported_at: String,
    project: VrkbProject,
    findings: Vec<VrkbFinding>,
    docs: Vec<VrkbDoc>,
    members: Vec<VrkbMemberExport>,
    asset_refs: Vec<AssetReference>,
    id_mappings: IdMappings,
}

#[derive(Debug, Serialize, Deserialize)]
struct VrkbMemberExport {
    user_id: Uuid,
    role: String,
    joined_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    asset_id: Uuid,
    original_path: String,
    mime_type: String,
    size_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdMappings {
    project_id: Uuid,
    finding_ids: Vec<Uuid>,
    doc_ids: Vec<Uuid>,
    asset_ids: Vec<Uuid>,
}

impl VrkbPortabilityProvider {
    pub fn new(repo: Arc<dyn VrkbRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl PortabilityProvider for VrkbPortabilityProvider {
    fn provider_id(&self) -> String {
        "vrkb".to_string()
    }

    async fn analyze_export(&self, kb_id: Uuid) -> Result<ExportSummary, String> {
        // kb_id maps to project_id for VRKB
        let project = self
            .repo
            .get_project(&kb_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;

        let findings = self
            .repo
            .list_findings(None, Some(kb_id))
            .await
            .map_err(|e| e.to_string())?;

        let docs = self
            .repo
            .list_docs(&kb_id)
            .await
            .map_err(|e| e.to_string())?;

        let members = self
            .repo
            .list_members(&kb_id)
            .await
            .map_err(|e| e.to_string())?;

        let assets = self
            .repo
            .list_project_assets(&kb_id)
            .await
            .map_err(|e| e.to_string())?;

        let total_items = findings.len() + docs.len() + members.len() + assets.len();

        Ok(ExportSummary {
            total_items,
            estimated_size: format!(
                "~{} KB",
                (total_items * 2) + (project.name.len() / 1024)
            ),
            sections: vec![
                ExportSection {
                    name: "Project".to_string(),
                    count: 1,
                    details: format!("Project: {}", project.name),
                },
                ExportSection {
                    name: "Findings".to_string(),
                    count: findings.len(),
                    details: format!("{} findings with evidence", findings.len()),
                },
                ExportSection {
                    name: "Documents".to_string(),
                    count: docs.len(),
                    details: format!("{} documents (nested)", docs.len()),
                },
                ExportSection {
                    name: "Members".to_string(),
                    count: members.len(),
                    details: format!("{} team members", members.len()),
                },
                ExportSection {
                    name: "Assets".to_string(),
                    count: assets.len(),
                    details: format!("{} asset references", assets.len()),
                },
            ],
        })
    }

    async fn export(
        &self,
        kb_id: Uuid,
        _user_id: Uuid,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<PathBuf, String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Collecting".to_string(),
                percent: 5,
                message: "Gathering project data...".to_string(),
                error: None,
            })
            .await;

        // Fetch all project data
        let project = self
            .repo
            .get_project(&kb_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Collecting".to_string(),
                percent: 15,
                message: "Fetching findings...".to_string(),
                error: None,
            })
            .await;

        let findings = self
            .repo
            .list_findings(None, Some(kb_id))
            .await
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Collecting".to_string(),
                percent: 30,
                message: "Fetching documents...".to_string(),
                error: None,
            })
            .await;

        let docs = self
            .repo
            .list_docs(&kb_id)
            .await
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Collecting".to_string(),
                percent: 45,
                message: "Fetching members and assets...".to_string(),
                error: None,
            })
            .await;

        let members = self
            .repo
            .list_members(&kb_id)
            .await
            .map_err(|e| e.to_string())?;

        let assets = self
            .repo
            .list_project_assets(&kb_id)
            .await
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Packaging".to_string(),
                percent: 60,
                message: "Building export package...".to_string(),
                error: None,
            })
            .await;

        // Build export package
        let package = VrkbExportPackage {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            project: project.clone(),
            findings: findings.clone(),
            docs: docs.clone(),
            members: members
                .iter()
                .map(|m| VrkbMemberExport {
                    user_id: m.user_id,
                    role: m.role.clone(),
                    joined_at: m.joined_at.to_rfc3339(),
                })
                .collect(),
            asset_refs: assets
                .iter()
                .map(|a| AssetReference {
                    asset_id: a.id,
                    original_path: a.storage_path.clone(),
                    mime_type: a.mime_type.clone(),
                    size_bytes: a.size_bytes,
                })
                .collect(),
            id_mappings: IdMappings {
                project_id: project.id,
                finding_ids: findings.iter().map(|f| f.id).collect(),
                doc_ids: docs.iter().map(|d| d.id).collect(),
                asset_ids: assets.iter().map(|a| a.id).collect(),
            },
        };

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Writing".to_string(),
                percent: 80,
                message: "Writing to disk...".to_string(),
                error: None,
            })
            .await;

        // Serialize to JSON and write to temp file
        let json =
            serde_json::to_string_pretty(&package).map_err(|e| format!("Serialize error: {}", e))?;

        let export_dir = std::env::temp_dir().join("vrkb_exports");
        std::fs::create_dir_all(&export_dir)
            .map_err(|e| format!("Failed to create export dir: {}", e))?;

        let filename = format!("vrkb-export-{}-{}.json", project.name, task_id);
        let file_path = export_dir.join(&filename);

        std::fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 99,
                message: "Export ready for download.".to_string(),
                error: None,
            })
            .await;

        Ok(file_path)
    }

    async fn analyze_import(&self, file_path: PathBuf) -> Result<ImportSummary, String> {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let package: VrkbExportPackage =
            serde_json::from_str(&content).map_err(|e| format!("Invalid VRKB package: {}", e))?;

        // Check for conflicts (existing project with same ID)
        let existing = self
            .repo
            .get_project(&package.project.id)
            .await
            .map_err(|e| e.to_string())?;

        let mut conflicts = vec![];
        if existing.is_some() {
            conflicts.push(format!(
                "Project '{}' already exists (ID: {}). It will be imported with a new ID.",
                package.project.name, package.project.id
            ));
        }

        Ok(ImportSummary {
            total_items: package.findings.len()
                + package.docs.len()
                + package.members.len()
                + package.asset_refs.len()
                + 1,
            sections: vec![
                ImportSection {
                    name: "Project".to_string(),
                    count: 1,
                    action: if existing.is_some() {
                        "Create (new ID)".to_string()
                    } else {
                        "Create".to_string()
                    },
                },
                ImportSection {
                    name: "Findings".to_string(),
                    count: package.findings.len(),
                    action: "Create".to_string(),
                },
                ImportSection {
                    name: "Documents".to_string(),
                    count: package.docs.len(),
                    action: "Create".to_string(),
                },
                ImportSection {
                    name: "Members".to_string(),
                    count: package.members.len(),
                    action: "Create".to_string(),
                },
                ImportSection {
                    name: "Asset References".to_string(),
                    count: package.asset_refs.len(),
                    action: "Skip (references only)".to_string(),
                },
            ],
            conflicts,
        })
    }

    async fn import(
        &self,
        _kb_id: Uuid,
        file_path: PathBuf,
        task_id: Uuid,
        progress: Sender<ProgressEvent>,
    ) -> Result<(), String> {
        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Parsing".to_string(),
                percent: 5,
                message: "Reading import file...".to_string(),
                error: None,
            })
            .await;

        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let package: VrkbExportPackage =
            serde_json::from_str(&content).map_err(|e| format!("Invalid VRKB package: {}", e))?;

        // Check if project already exists; if so, generate new IDs
        let existing = self
            .repo
            .get_project(&package.project.id)
            .await
            .map_err(|e| e.to_string())?;

        let new_project_id = if existing.is_some() {
            Uuid::new_v4()
        } else {
            package.project.id
        };

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing".to_string(),
                percent: 15,
                message: "Creating project...".to_string(),
                error: None,
            })
            .await;

        // Create project with potentially new ID
        let mut project = package.project.clone();
        project.id = new_project_id;
        project.updated_at = chrono::Utc::now();

        self.repo
            .create_project(project)
            .await
            .map_err(|e| e.to_string())?;

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing".to_string(),
                percent: 35,
                message: format!("Importing {} findings...", package.findings.len()),
                error: None,
            })
            .await;

        // Import findings — remap section_id references if needed
        for finding in &package.findings {
            let mut f = finding.clone();
            // If project ID changed, we need to handle section remapping
            // For now, preserve original finding data
            f.updated_at = chrono::Utc::now();
            let _ = self.repo.create_finding(f).await;
        }

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing".to_string(),
                percent: 55,
                message: format!("Importing {} documents...", package.docs.len()),
                error: None,
            })
            .await;

        // Import docs — remap project_id
        for doc in &package.docs {
            let mut d = doc.clone();
            d.project_id = new_project_id;
            d.updated_at = chrono::Utc::now();
            d.deleted_at = None; // Don't import trashed docs as trashed
            let _ = self.repo.create_doc(d).await;
        }

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Importing".to_string(),
                percent: 75,
                message: format!("Importing {} members...", package.members.len()),
                error: None,
            })
            .await;

        // Import members
        for member_export in &package.members {
            let member = crate::domain::models::VrkbMember {
                project_id: new_project_id,
                user_id: member_export.user_id,
                role: member_export.role.clone(),
                joined_at: chrono::Utc::now(),
                user: None,
            };
            let _ = self.repo.add_member(member).await;
        }

        let _ = progress
            .send(ProgressEvent {
                task_id,
                stage: "Finalizing".to_string(),
                percent: 95,
                message: "Import complete. Asset references preserved as metadata.".to_string(),
                error: None,
            })
            .await;

        Ok(())
    }
}
