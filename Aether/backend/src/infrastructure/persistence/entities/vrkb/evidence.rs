use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vrkb_evidence")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: Uuid,
    /// Type: screenshot, request_response, log_extract, poc_file, external_reference
    pub evidence_type: String,
    pub title: String,
    #[sea_orm(column_type = "Json")]
    pub content: Option<serde_json::Value>,
    pub asset_id: Option<Uuid>,
    #[sea_orm(column_type = "Text")]
    pub url: Option<String>,
    /// Polymorphic link type: "finding", "doc", "asset"
    pub linked_entity_type: Option<String>,
    pub linked_entity_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Project,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
