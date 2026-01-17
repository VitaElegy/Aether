use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vocab_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub vocab_id: Uuid,
    pub sentence: String,
    pub translation: Option<String>,
    pub note: Option<String>,
    pub image_url: Option<String>,
    pub article_id: Option<Uuid>,
    pub sentence_uuid: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::vocab_detail::Entity",
        from = "Column::VocabId",
        to = "super::vocab_detail::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    VocabDetail,
}

impl Related<super::vocab_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VocabDetail.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
