use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contents")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub author_id: String,
    pub title: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub status: String,
    pub visibility: String, // Added
    pub category: Option<String>, // Added
    #[sea_orm(column_type = "Text", nullable)]
    pub knowledge_base_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    pub tags: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::knowledge_base::Entity",
        from = "Column::KnowledgeBaseId",
        to = "super::knowledge_base::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    KnowledgeBase,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::knowledge_base::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::KnowledgeBase.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
