use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum MemoPriority {
    #[sea_orm(string_value = "P0")]
    P0, // Urgent
    #[sea_orm(string_value = "P1")]
    P1, // High
    #[sea_orm(string_value = "P2")]
    P2, // Normal
    #[sea_orm(string_value = "P3")]
    P3, // Low
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum MemoStatus {
    #[sea_orm(string_value = "Todo")]
    Todo,
    #[sea_orm(string_value = "Doing")]
    Doing,
    #[sea_orm(string_value = "Done")]
    Done,
    #[sea_orm(string_value = "Archived")]
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum MemoColor {
    #[sea_orm(string_value = "Yellow")]
    Yellow,
    #[sea_orm(string_value = "Red")]
    Red,
    #[sea_orm(string_value = "Green")]
    Green,
    #[sea_orm(string_value = "Blue")]
    Blue,
    #[sea_orm(string_value = "Purple")]
    Purple,
    #[sea_orm(string_value = "Gray")]
    Gray,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "memo_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid, // FK to nodes.id
    
    // Project/Board Context (Can be null if in Global Inbox)
    pub project_id: Option<Uuid>, 

    // Visualization
    pub color: MemoColor,
    pub is_pinned: bool,

    // Content
    // Content
    #[sea_orm(column_type = "JsonBinary")]
    pub content: Json,
    
    // GTD Fields
    pub status: MemoStatus,
    pub priority: MemoPriority,
    pub due_at: Option<DateTimeWithTimeZone>,
    pub reminder_at: Option<DateTimeWithTimeZone>,

    // Tags
    #[sea_orm(column_type = "JsonBinary")]
    pub tags: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::node::Entity",
        from = "Column::Id",
        to = "super::node::Column::Id",
        on_update = "Cascade",
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
