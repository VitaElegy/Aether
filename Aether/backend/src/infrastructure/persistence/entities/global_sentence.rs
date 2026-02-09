use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "global_sentences")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub text: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub translation: Option<String>,
    pub origin_article_id: Option<Uuid>,
    pub origin_sentence_uuid: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::vocab_example::Entity")]
    VocabExamples,
}

impl Related<super::vocab_example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VocabExamples.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
