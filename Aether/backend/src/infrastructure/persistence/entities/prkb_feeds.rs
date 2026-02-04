use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "prkb_feeds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub feed_type: String,
    pub last_fetched_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::prkb_inbox::Entity")]
    InboxItems,
}

impl Related<super::prkb_inbox::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InboxItems.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
