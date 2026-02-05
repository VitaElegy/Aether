use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_papers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub authors: Json,
    #[sea_orm(column_type = "Text")]
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub publication: Option<String>,
    pub publish_date: DateTimeUtc,
    pub source: String,
    pub saved_at: DateTimeUtc,
    pub is_read: bool,
    pub tags: Json,
    pub arxiv_id: Option<String>,
    pub venue_id: Option<Uuid>,
    pub state: String,
    pub pdf_local_path: Option<String>,
    pub metadata: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::prkb_venues::Entity",
        from = "Column::VenueId",
        to = "super::prkb_venues::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Venue,
    #[sea_orm(has_one = "super::prkb_signals::Entity")]
    Signals,
    #[sea_orm(has_many = "super::prkb_papers_authors::Entity")]
    PapersAuthors,
}

impl Related<super::prkb_venues::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Venue.def()
    }
}

impl Related<super::prkb_signals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Signals.def()
    }
}

impl Related<super::prkb_authors::Entity> for Entity {
    fn to() -> RelationDef {
        super::prkb_papers_authors::Relation::Authors.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::prkb_papers_authors::Relation::Papers.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
