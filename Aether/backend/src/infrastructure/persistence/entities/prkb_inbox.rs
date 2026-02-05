use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_inbox")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub feed_id: Uuid,
    pub external_id: String,
    pub title: String,
    pub authors: Json,
    #[sea_orm(column_type = "Text")]
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub publication: Option<String>,
    pub publish_date: DateTimeUtc,
    pub is_read: bool,
    pub is_saved: bool,
    pub fetched_at: DateTimeUtc,
    pub state: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::prkb_feeds::Entity",
        from = "Column::FeedId",
        to = "super::prkb_feeds::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Feed,
}

impl Related<super::prkb_feeds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feed.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
