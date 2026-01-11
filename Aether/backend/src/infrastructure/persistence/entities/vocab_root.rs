use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vocab_roots")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub root: String,
    pub meaning: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::vocab_detail::Entity")]
    Vocabularies,
}

impl Related<super::vocab_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Vocabularies.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
