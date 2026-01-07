use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vocabularies")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub user_id: String,
    #[sea_orm(column_type = "Text")]
    pub word: String,
    #[sea_orm(column_type = "Text")]
    pub definition: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub translation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub phonetic: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub context_sentence: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub image_url: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub language: String,
    #[sea_orm(column_type = "Text")]
    pub status: String,
    pub created_at: String, // Stored as RFC3339 string
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
