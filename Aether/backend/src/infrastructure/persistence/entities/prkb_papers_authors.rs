use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_papers_authors")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub paper_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub author_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::prkb_papers::Entity",
        from = "Column::PaperId",
        to = "super::prkb_papers::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Papers,
    #[sea_orm(
        belongs_to = "super::prkb_authors::Entity",
        from = "Column::AuthorId",
        to = "super::prkb_authors::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Authors,
}

impl Related<super::prkb_papers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Papers.def()
    }
}

impl Related<super::prkb_authors::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Authors.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
