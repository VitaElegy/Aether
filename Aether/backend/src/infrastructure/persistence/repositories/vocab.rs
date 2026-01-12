use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Vocabulary, Node, NodeType, PermissionMode};
use crate::domain::models::UserId;
use crate::domain::ports::{VocabularyRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{node, vocab_detail};

#[async_trait]
impl VocabularyRepository for PostgresRepository {
    async fn save(&self, vocab: Vocabulary) -> Result<Uuid, RepositoryError> {
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 1. Handle Root (if present)
        let mut root_id = None;
        if let Some(r_str) = &vocab.root {
            use crate::infrastructure::persistence::entities::vocab_root;
            
            // Check if root exists
            let existing = vocab_root::Entity::find()
                .filter(vocab_root::Column::Root.eq(r_str))
                .one(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
            if let Some(e) = existing {
                root_id = Some(e.id);
            } else {
                let new_id = Uuid::new_v4();
                let model = vocab_root::ActiveModel {
                    id: Set(new_id),
                    root: Set(r_str.clone()),
                    meaning: Set(None), 
                };
                vocab_root::Entity::insert(model)
                    .exec(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
                root_id = Some(new_id);
            }
        }

        // 2. Save Node
        let node_model = node::ActiveModel {
            id: Set(vocab.node.id),
            parent_id: Set(vocab.node.parent_id),
            author_id: Set(vocab.node.author_id),
            knowledge_base_id: Set(vocab.node.knowledge_base_id),
            r#type: Set("Vocabulary".to_string()),
            title: Set(vocab.node.title.clone()), 
            permission_mode: Set(match vocab.node.permission_mode {
                PermissionMode::Public => "Public".to_string(),
                PermissionMode::Private => "Private".to_string(),
                PermissionMode::Internal => "Internal".to_string(),
            }),
            permission_data: Set(None),
            created_at: Set(vocab.node.created_at.into()),
            updated_at: Set(vocab.node.updated_at.into()),
        };
        node::Entity::insert(node_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(node::Column::Id)
                    .update_columns([node::Column::Title, node::Column::UpdatedAt])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 3. Save Detail (with Root ID)
        let detail_model = vocab_detail::ActiveModel {
            id: Set(vocab.node.id),
            word: Set(vocab.word),
            definition: Set(vocab.definition),
            translation: Set(vocab.translation),
            phonetic: Set(vocab.phonetic),
            language: Set(vocab.language),
            status: Set(vocab.status),
            root_id: Set(root_id),
            query_count: Set(vocab.query_count),
            is_important: Set(vocab.is_important),
        };
        vocab_detail::Entity::insert(detail_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(vocab_detail::Column::Id)
                    .update_columns([vocab_detail::Column::Definition, vocab_detail::Column::Translation, vocab_detail::Column::Status, vocab_detail::Column::RootId, vocab_detail::Column::QueryCount, vocab_detail::Column::IsImportant])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 4. Save Examples
        // First, delete existing examples (simple full overwrite for editing)
        use crate::infrastructure::persistence::entities::vocab_example;
        vocab_example::Entity::delete_many()
            .filter(vocab_example::Column::VocabId.eq(vocab.node.id))
            .exec(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if !vocab.examples.is_empty() {
            let example_models: Vec<vocab_example::ActiveModel> = vocab.examples.into_iter().map(|ex| {
                vocab_example::ActiveModel {
                    id: Set(ex.id), // Use provided ID or generate? Provided from handler.
                    vocab_id: Set(vocab.node.id),
                    sentence: Set(ex.sentence),
                    translation: Set(ex.translation),
                    note: Set(ex.note),
                    image_url: Set(ex.image_url),
                    created_at: Set(ex.created_at.into()),
                }
            }).collect();
            
             vocab_example::Entity::insert_many(example_models)
                .exec(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(vocab.node.id)
    }

    async fn find_by_word(&self, user_id: &UserId, word: &str) -> Result<Option<Vocabulary>, RepositoryError> {
        let result = vocab_detail::Entity::find()
            .filter(vocab_detail::Column::Word.eq(word))
            .find_also_related(node::Entity)
            .all(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        for (d, n_opt) in result {
             if let Some(n) = n_opt {
                 if n.author_id == user_id.0 {
                     // Lazy Load Root and Examples
                     let root = if let Some(rid) = d.root_id {
                         use crate::infrastructure::persistence::entities::vocab_root;
                         vocab_root::Entity::find_by_id(rid).one(&self.db).await
                            .unwrap_or(None).map(|r| r.root)
                     } else { None };

                     use crate::infrastructure::persistence::entities::vocab_example;
                     let examples = d.find_related(vocab_example::Entity).all(&self.db).await
                         .unwrap_or_default().into_iter().map(map_example).collect();
                     
                     return Ok(Some(map_vocab(n, d, root, examples)));
                 }
             }
        }
        Ok(None)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Vocabulary>, RepositoryError> {
        let result = node::Entity::find_by_id(*id)
            .find_also_related(vocab_detail::Entity)
            .one(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        match result {
            Some((n, Some(d))) => {
                 let root = if let Some(rid) = d.root_id {
                     use crate::infrastructure::persistence::entities::vocab_root;
                     vocab_root::Entity::find_by_id(rid).one(&self.db).await
                        .unwrap_or(None).map(|r| r.root)
                 } else { None };

                 use crate::infrastructure::persistence::entities::vocab_example;
                 let examples = d.find_related(vocab_example::Entity).all(&self.db).await
                     .unwrap_or_default().into_iter().map(map_example).collect();
                
                 Ok(Some(map_vocab(n, d, root, examples)))
            },
            _ => Ok(None)
        }
    }

    async fn list(&self, user_id: &UserId, limit: u64, offset: u64, query: Option<String>, sort_by: Option<String>, order: Option<String>) -> Result<Vec<Vocabulary>, RepositoryError> {
        // Eager Loading using find_with_related is messy for 3 levels, so doing it iteratively or with find_also
        // Let's do a basic list then fetch details.
        
        let mut select = node::Entity::find()
            .filter(node::Column::Type.eq("Vocabulary"))
            .filter(node::Column::AuthorId.eq(user_id.0)) 
            .find_also_related(vocab_detail::Entity);
            
        // Query Search
        if let Some(q) = query {
            // Filter by word (Node Title)
            select = select.filter(node::Column::Title.contains(&q));
        }

        // Sorting
        let sort_col = sort_by.as_deref().unwrap_or("created_at");
        let is_desc = order.as_deref().unwrap_or("desc") == "desc";
        let order_enum = if is_desc { sea_orm::Order::Desc } else { sea_orm::Order::Asc };
        
        match sort_col {
             "query_count" => {
                 select = select.order_by(vocab_detail::Column::QueryCount, order_enum);
             },
             "is_important" => {
                 select = select.order_by(vocab_detail::Column::IsImportant, order_enum);
             },
             "word" | "title" => {
                 select = select.order_by(node::Column::Title, order_enum);
             },
             _ => {
                 select = select.order_by(node::Column::CreatedAt, order_enum);
             }
        }
        
        // Stabilize Sort
        if sort_col != "created_at" {
             select = select.order_by_desc(node::Column::CreatedAt);
        }

        let results = select
            .limit(limit)
            .offset(offset)
            .all(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
         let mut vocabs = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                 let root = if let Some(rid) = detail.root_id {
                     use crate::infrastructure::persistence::entities::vocab_root;
                     vocab_root::Entity::find_by_id(rid).one(&self.db).await
                        .unwrap_or(None).map(|r| r.root)
                 } else { None };
                 
                 // Optimization: Could batch load examples, but for now N+1 limited by paging is acceptable for MVP
                 use crate::infrastructure::persistence::entities::vocab_example;
                 let examples = detail.find_related(vocab_example::Entity).all(&self.db).await
                     .unwrap_or_default().into_iter().map(map_example).collect();
                
                vocabs.push(map_vocab(n, detail, root, examples));
            }
        }
        Ok(vocabs)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id).exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn delete_many(&self, ids: &[Uuid]) -> Result<(), RepositoryError> {
        node::Entity::delete_many()
            .filter(node::Column::Id.is_in(ids.to_vec()))
            .exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn increment_query_count(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use sea_orm::sea_query::{Query, Expr};
        // Atomic increment using SeaQuery
        let stmt = sea_orm::Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"UPDATE vocab_details SET query_count = query_count + 1 WHERE id = $1"#,
            vec![(*id).into()]
        );
        self.db.execute(stmt).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn set_importance(&self, id: &Uuid, is_important: bool) -> Result<(), RepositoryError> {
        let stmt = sea_orm::Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"UPDATE vocab_details SET is_important = $1 WHERE id = $2"#,
            vec![is_important.into(), (*id).into()]
        );
        self.db.execute(stmt).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

fn map_vocab(n: node::Model, d: vocab_detail::Model, root: Option<String>, examples: Vec<crate::domain::models::VocabularyExample>) -> Vocabulary {
    Vocabulary {
        node: Node {
             id: n.id,
            parent_id: n.parent_id,
            author_id: n.author_id,
            knowledge_base_id: n.knowledge_base_id,
            r#type: NodeType::Vocabulary,
            title: n.title,
            permission_mode: match n.permission_mode.as_str() {
                "Private" => PermissionMode::Private,
                "Internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            },
            created_at: n.created_at.with_timezone(&Utc),
            updated_at: n.updated_at.with_timezone(&Utc),
        },
        word: d.word,
        definition: d.definition,
        translation: d.translation,
        phonetic: d.phonetic,
        context_sentence: None,
        image_url: None,
        language: d.language,
        status: d.status,
        root,
        examples,
        query_count: d.query_count,
        is_important: d.is_important,
    }
}

fn map_example(e: crate::infrastructure::persistence::entities::vocab_example::Model) -> crate::domain::models::VocabularyExample {
    crate::domain::models::VocabularyExample {
        id: e.id,
        sentence: e.sentence,
        translation: e.translation,
        note: e.note,
        image_url: e.image_url,
        created_at: e.created_at.with_timezone(&Utc),
    }
}
