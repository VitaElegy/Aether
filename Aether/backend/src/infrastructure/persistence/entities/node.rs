use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Uuid,
    pub knowledge_base_id: Option<Uuid>,
    pub r#type: String, // 'article', 'vocabulary', 'memo', 'folder'
    pub title: String,
    pub permission_mode: String, // 'Public', 'Private', 'Internal'
    pub permission_data: Option<String>, // JSON string
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
    #[sea_orm(has_one = "super::article_detail::Entity")]
    ArticleDetail,
    #[sea_orm(has_one = "super::vocab_detail::Entity")]
    VocabDetail,
    #[sea_orm(has_one = "super::memo_detail::Entity")]
    MemoDetail,
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

impl Related<super::article_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ArticleDetail.def()
    }
}

impl Related<super::vocab_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VocabDetail.def()
    }
}

impl Related<super::memo_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemoDetail.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
