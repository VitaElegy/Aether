use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Vocabulary, Node, NodeType, PermissionMode};
use crate::domain::models::UserId;
use crate::domain::ports::{VocabularyRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{node, vocab_detail, vocab_example, global_sentence, vocab_root};

#[async_trait]
impl VocabularyRepository for PostgresRepository {
    async fn save(&self, vocab: Vocabulary) -> Result<Uuid, RepositoryError> {
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 1. Handle Root (if present)
        let mut root_id = None;
        if let Some(r_str) = &vocab.root {
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

        // 4. Save Examples (Shared/Global Logic)
        
        // Get current examples for this vocab to see what is being removed
        let current_examples = vocab_example::Entity::find()
            .filter(vocab_example::Column::VocabId.eq(vocab.node.id))
            .all(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        let current_ids: Vec<Uuid> = current_examples.iter().map(|e| e.id).collect();
        let input_ids: Vec<Uuid> = vocab.examples.iter().map(|e| e.id).collect();
        
        // Delete removed examples
        let to_delete: Vec<Uuid> = current_ids.into_iter().filter(|id| !input_ids.contains(id)).collect();
        if !to_delete.is_empty() {
             vocab_example::Entity::delete_many()
                .filter(vocab_example::Column::Id.is_in(to_delete))
                .exec(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        for ex in vocab.examples {
            // Logic:
            // 1. Check if ex.sentence matches an existing global_sentence (by ID or text)
            //    - The frontend *should* send global_sentence_id if it knows it.
            //    - But our domain model `VocabularyExample` does not have `global_sentence_id` field exposed yet?
            //    - It has `sentence_uuid`... but that was for something else (origin).
            //    - Let's assume we rely on text matching or existing link.
            
            // Try to find if this example row already exists
            let existing_link = vocab_example::Entity::find_by_id(ex.id)
                .one(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
            let global_id = if let Some(link) = existing_link {
                // Existing link.
                if let Some(gid) = link.global_sentence_id {
                    // It has a global sentence.
                    // "Global Update" scope: Update the global sentence text.
                    let global_s = global_sentence::ActiveModel {
                        id: Set(gid),
                        text: Set(ex.sentence.clone()),
                        translation: Set(ex.translation.clone()), // Also update translation? Yes, likely desired.
                        ..Default::default()
                    };
                    global_sentence::Entity::update(global_s)
                        .exec(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
                    gid
                } else {
                    // Legacy record without global_sentence_id? Upgrade it.
                    // Or it's a new global sentence.
                    create_or_find_global_sentence(&txn, &ex).await?
                }
            } else {
                // New Example
                create_or_find_global_sentence(&txn, &ex).await?
            };

            let example_model = vocab_example::ActiveModel {
                id: Set(ex.id),
                vocab_id: Set(vocab.node.id),
                sentence: Set(None), // Deprecated/Nullable
                translation: Set(None), // Deprecated/Nullable (stored in global)
                note: Set(ex.note), // Note is specific to the usage (e.g. grammar focus), so keep on link
                image_url: Set(ex.image_url), // Image might be specific? Or shared? Let's keep specific for now.
                article_id: Set(ex.article_id),
                sentence_uuid: Set(ex.sentence_uuid),
                created_at: Set(ex.created_at.into()),
                global_sentence_id: Set(Some(global_id)),
            };
            
            vocab_example::Entity::insert(example_model)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(vocab_example::Column::Id)
                        .update_columns([vocab_example::Column::Note, vocab_example::Column::ImageUrl, vocab_example::Column::GlobalSentenceId])
                        .to_owned()
                )
                .exec(&txn).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        // Cleanup Orphans
        let stmt = Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"DELETE FROM global_sentences WHERE id NOT IN (SELECT DISTINCT global_sentence_id FROM vocab_examples WHERE global_sentence_id IS NOT NULL)"#,
            vec![]
        );
        txn.execute(stmt).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(vocab.node.id)
    }

    async fn find_by_word(&self, user_id: &UserId, word: &str) -> Result<Option<Vocabulary>, RepositoryError> {
        let details = vocab_detail::Entity::find()
            .filter(vocab_detail::Column::Word.eq(word))
            .all(&self.db).await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        for d in details {
             let n_opt = node::Entity::find_by_id(d.id).one(&self.db).await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
             
             if let Some(n) = n_opt {
                 if n.author_id == user_id.0 {
                     let root = if let Some(rid) = d.root_id {
                         vocab_root::Entity::find_by_id(rid).one(&self.db).await
                            .unwrap_or(None).map(|r| r.root)
                     } else { None };

                     let examples = fetch_examples_for_vocab(&self.db, d.id).await?;
                     
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
                     vocab_root::Entity::find_by_id(rid).one(&self.db).await
                        .unwrap_or(None).map(|r| r.root)
                 } else { None };

                 let examples = fetch_examples_for_vocab(&self.db, d.id).await?;
                
                 Ok(Some(map_vocab(n, d, root, examples)))
            },
            _ => Ok(None)
        }
    }

    async fn list(&self, user_id: &UserId, limit: u64, offset: u64, query: Option<String>, sort_by: Option<String>, order: Option<String>, knowledge_base_id: Option<Uuid>) -> Result<Vec<Vocabulary>, RepositoryError> {
        let mut select = node::Entity::find()
            .filter(node::Column::Type.eq("Vocabulary"))
            .filter(node::Column::AuthorId.eq(user_id.0)) 
            .find_also_related(vocab_detail::Entity);

        if let Some(kbid) = knowledge_base_id {
            select = select.filter(node::Column::KnowledgeBaseId.eq(kbid));
        }

        if let Some(q) = query {
            select = select.filter(node::Column::Title.contains(&q));
        }

        let sort_col = sort_by.as_deref().unwrap_or("created_at");
        let is_desc = order.as_deref().unwrap_or("desc") == "desc";
        let order_enum = if is_desc { sea_orm::Order::Desc } else { sea_orm::Order::Asc };
        
        match sort_col {
             "query_count" => { select = select.order_by(vocab_detail::Column::QueryCount, order_enum); },
             "is_important" => { select = select.order_by(vocab_detail::Column::IsImportant, order_enum); },
             "word" | "title" => { select = select.order_by(node::Column::Title, order_enum); },
             _ => { select = select.order_by(node::Column::CreatedAt, order_enum); }
        }
        
        if sort_col != "created_at" { select = select.order_by_desc(node::Column::CreatedAt); }

        let results = select
            .limit(limit)
            .offset(offset)
            .all(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
         let mut vocabs = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                 let root = if let Some(rid) = detail.root_id {
                     vocab_root::Entity::find_by_id(rid).one(&self.db).await
                        .unwrap_or(None).map(|r| r.root)
                 } else { None };
                 
                 let examples = fetch_examples_for_vocab(&self.db, detail.id).await?;
                
                vocabs.push(map_vocab(n, detail, root, examples));
            }
        }
        Ok(vocabs)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        node::Entity::delete_by_id(*id).exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        // Orphan Cleanup
         let stmt = Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"DELETE FROM global_sentences WHERE id NOT IN (SELECT DISTINCT global_sentence_id FROM vocab_examples WHERE global_sentence_id IS NOT NULL)"#,
            vec![]
        );
        txn.execute(stmt).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn delete_many(&self, ids: &[Uuid]) -> Result<(), RepositoryError> {
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        node::Entity::delete_many()
            .filter(node::Column::Id.is_in(ids.to_vec()))
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        // Orphan Cleanup
         let stmt = Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"DELETE FROM global_sentences WHERE id NOT IN (SELECT DISTINCT global_sentence_id FROM vocab_examples WHERE global_sentence_id IS NOT NULL)"#,
            vec![]
        );
        txn.execute(stmt).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn increment_query_count(&self, id: &Uuid) -> Result<(), RepositoryError> {
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

    async fn count(&self, user_id: &UserId, knowledge_base_id: Option<Uuid>) -> Result<u64, RepositoryError> {
        let mut query = node::Entity::find()
            .filter(node::Column::Type.eq("Vocabulary"))
            .filter(node::Column::AuthorId.eq(user_id.0));

        if let Some(kbid) = knowledge_base_id {
            query = query.filter(node::Column::KnowledgeBaseId.eq(kbid));
        }

        let count = query.count(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(count)
    }

    async fn search_global_sentences(&self, query: &str) -> Result<Vec<(Uuid, String, Option<String>)>, RepositoryError> {
        let results = global_sentence::Entity::find()
            .filter(global_sentence::Column::Text.contains(query))
            .limit(20)
            .all(&self.db).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(results.into_iter().map(|g| (g.id, g.text, g.translation)).collect())
    }
}

// Helpers

async fn create_or_find_global_sentence<C>(db: &C, ex: &crate::domain::models::VocabularyExample) -> Result<Uuid, RepositoryError> 
where C: ConnectionTrait 
{
    // 1. Try to find match by text
    let existing = global_sentence::Entity::find()
        .filter(global_sentence::Column::Text.eq(&ex.sentence))
        .one(db).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
    if let Some(g) = existing {
        return Ok(g.id);
    }
    
    // 2. Create new
    let new_id = Uuid::new_v4();
    let model = global_sentence::ActiveModel {
        id: Set(new_id),
        text: Set(ex.sentence.clone()),
        translation: Set(ex.translation.clone()),
        origin_article_id: Set(ex.article_id),
        origin_sentence_uuid: Set(ex.sentence_uuid),
        created_at: Set(Utc::now().into()),
    };
    global_sentence::Entity::insert(model)
        .exec(db).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
    
    Ok(new_id)
}

async fn fetch_examples_for_vocab(db: &DatabaseConnection, vocab_id: Uuid) -> Result<Vec<crate::domain::models::VocabularyExample>, RepositoryError> {
    let examples = vocab_example::Entity::find()
        .filter(vocab_example::Column::VocabId.eq(vocab_id))
        .find_also_related(global_sentence::Entity)
        .all(db).await.map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
    Ok(examples.into_iter().map(|(ex, global)| {
        let (sentence, translation, global_sentence_id) = if let Some(g) = global {
            (g.text, g.translation, Some(g.id))
        } else {
            (ex.sentence.unwrap_or_default(), ex.translation, None)
        };
        
        crate::domain::models::VocabularyExample {
            id: ex.id,
            sentence,
            translation,
            note: ex.note,
            image_url: ex.image_url,
            article_id: ex.article_id,
            sentence_uuid: ex.sentence_uuid,
            created_at: ex.created_at.with_timezone(&Utc),
            global_sentence_id,
        }
    }).collect())
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
