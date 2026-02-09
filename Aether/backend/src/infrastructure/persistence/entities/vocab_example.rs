use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vocab_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub vocab_id: Uuid,
    // Legacy/Cache field, now nullable. 
    // In new logic, this might be empty if linked to global_sentence, 
    // OR we keep it synced for easier querying without joins (denormalization).
    // User asked for "Shared" behavior. If we denormalize, we must update all copies.
    // Given the "Global Update" requirement, it is better to rely on JOINs or strictly keep them in sync.
    // For now, let's make it Option.
    pub sentence: Option<String>, 
    pub translation: Option<String>,
    pub note: Option<String>,
    pub image_url: Option<String>,
    pub article_id: Option<Uuid>,
    pub sentence_uuid: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub global_sentence_id: Option<Uuid>,
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

    #[sea_orm(
        belongs_to = "super::global_sentence::Entity",
        from = "Column::GlobalSentenceId",
        to = "super::global_sentence::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    GlobalSentence,
}

impl Related<super::vocab_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VocabDetail.def()
    }
}

impl Related<super::global_sentence::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GlobalSentence.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
