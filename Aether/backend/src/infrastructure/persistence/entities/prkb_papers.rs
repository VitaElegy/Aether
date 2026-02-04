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
    pub publish_date: DateTimeUtc,
    pub source: String,
    pub saved_at: DateTimeUtc,
    pub is_read: bool,
    pub tags: Json,
    pub arxiv_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
