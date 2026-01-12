use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vocab_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid, // FK to nodes.id
    pub word: String,
    pub definition: String,
    pub translation: Option<String>,
    pub phonetic: Option<String>,
    pub language: String,
    pub status: String,
    pub root_id: Option<Uuid>,
    #[sea_orm(default_value = 0)]
    pub query_count: i32,
    #[sea_orm(default_value = false)]
    pub is_important: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::node::Entity",
        from = "Column::Id",
        to = "super::node::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Node,

    #[sea_orm(
        belongs_to = "super::vocab_root::Entity",
        from = "Column::RootId",
        to = "super::vocab_root::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Root,
    
    #[sea_orm(has_many = "super::vocab_example::Entity")]
    Examples,
}

impl Related<super::node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl Related<super::vocab_root::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Root.def()
    }
}

impl Related<super::vocab_example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Examples.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
