use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_authors")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub canonical_name: Option<String>,
    pub profile_url: Option<String>,
    pub aliases: Json,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::prkb_papers_authors::Entity")]
    PapersAuthors,
}

impl Related<super::prkb_papers::Entity> for Entity {
    fn to() -> RelationDef {
        super::prkb_papers_authors::Relation::Papers.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::prkb_papers_authors::Relation::Authors.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
