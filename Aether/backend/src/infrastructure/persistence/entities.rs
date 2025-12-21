use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Table Name: contents
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub status: String, // Stored as string, mapped to Enum in Domain
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    #[sea_orm(column_type = "JsonBinary")]
    pub body: Json,     // The genius part: we store the complex Body enum as JSONB
    pub tags: Vec<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

