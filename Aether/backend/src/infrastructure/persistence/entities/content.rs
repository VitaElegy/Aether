use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub status: String,
    pub visibility: String, 
    pub category: Option<String>, 
    pub knowledge_base_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub content_type: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "JsonBinary")]
    pub body: serde_json::Value,
    #[sea_orm(column_type = "JsonBinary")]
    pub tags: serde_json::Value,
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
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Parent,
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
