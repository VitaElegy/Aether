use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vrkb_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub hash: String,
    pub storage_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project_asset::Entity")]
    ProjectAsset,
}

impl Related<super::project_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectAsset.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
