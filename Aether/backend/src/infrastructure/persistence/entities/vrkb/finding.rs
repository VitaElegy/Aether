use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vrkb_findings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    pub status: String,
    pub severity: String,
    #[sea_orm(column_type = "Json")]
    pub content: Option<serde_json::Value>,
    pub is_triage: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub author_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::section::Entity",
        from = "Column::SectionId",
        to = "super::section::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Section,
}

impl Related<super::section::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Section.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
