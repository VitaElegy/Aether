use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub knowledge_base_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub label: String,
    pub data: Json,
    pub rank: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Parent,
}

impl ActiveModelBehavior for ActiveModel {}
