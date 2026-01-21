use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vrkb_projects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub repository_url: Option<String>,
    pub quota_bytes: i64,
    #[sea_orm(column_type = "Json")]
    pub settings: Option<serde_json::Value>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::section::Entity")]
    Section,
    #[sea_orm(has_many = "super::project_asset::Entity")]
    ProjectAsset,
}

impl Related<super::section::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Section.def()
    }
}

impl Related<super::project_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectAsset.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
