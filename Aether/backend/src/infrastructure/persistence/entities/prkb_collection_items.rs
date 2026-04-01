use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_collection_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub collection_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub paper_id: Uuid,
    pub added_at: DateTimeUtc,
    pub sort_order: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::prkb_collections::Entity",
        from = "Column::CollectionId",
        to = "super::prkb_collections::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Collection,
    #[sea_orm(
        belongs_to = "super::prkb_papers::Entity",
        from = "Column::PaperId",
        to = "super::prkb_papers::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Paper,
}

impl Related<super::prkb_collections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collection.def()
    }
}

impl Related<super::prkb_papers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Paper.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
