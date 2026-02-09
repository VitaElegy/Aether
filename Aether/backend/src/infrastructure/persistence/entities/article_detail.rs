use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "article_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid, // FK to nodes.id
    pub slug: String,
    pub status: String,
    pub category: Option<String>,
    pub body: Json,
    pub tags: String, // Kept as simple storage for now
    #[sea_orm(nullable)]
    pub derived_data: Option<Json>,
    #[sea_orm(nullable)]
    pub public_version_id: Option<Uuid>,
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
}

impl Related<super::node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
