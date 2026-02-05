use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_signals")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub paper_id: Uuid,
    pub citation_count: i32,
    pub github_stars: i32,
    pub sota_rank: Option<String>,
    pub last_updated: DateTimeUtc,
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
    Paper,
}

impl Related<super::prkb_papers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Paper.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
