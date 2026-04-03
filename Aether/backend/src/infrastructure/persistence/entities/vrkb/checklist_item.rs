use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vrkb_checklist_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: Option<String>,
    pub is_completed: bool,
    pub sort_order: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
