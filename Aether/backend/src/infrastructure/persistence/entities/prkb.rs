

// PRKB Entitites
// We organize them in submodules to ensure generic names like 'Model' don't clash.

pub mod papers {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "prkb_papers")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub title: String,
        pub authors: Json,
        #[sea_orm(column_type = "Text")]
        pub abstract_text: String,
        pub url: String,
        pub pdf_url: Option<String>,
        pub publish_date: DateTimeWithTimeZone,
        pub source: String,
        pub saved_at: DateTimeWithTimeZone,
        pub is_read: bool,
        pub tags: Json,
        pub arxiv_id: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod feeds {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "prkb_feeds")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub name: String,
        pub url: String,
        pub feed_type: String, // 'arxiv', 'rss'
        pub last_fetched_at: Option<DateTimeWithTimeZone>,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::inbox::Entity")]
        Inbox,
    }
    
    impl Related<super::inbox::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Inbox.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod inbox {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "prkb_inbox")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub feed_id: Uuid,
        pub external_id: String,
        pub title: String,
        pub authors: Json,
        #[sea_orm(column_type = "Text")]
        pub abstract_text: String,
        pub url: String,
        pub pdf_url: Option<String>,
        pub publish_date: DateTimeWithTimeZone,
        pub is_read: bool,
        pub is_saved: bool,
        pub fetched_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::feeds::Entity",
            from = "Column::FeedId",
            to = "super::feeds::Column::Id",
            on_update = "NoAction",
            on_delete = "Cascade"
        )]
        Feed,
    }

    impl Related<super::feeds::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Feed.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
