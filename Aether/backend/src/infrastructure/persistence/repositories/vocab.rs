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

        // 1. Save Node
        let node_model = node::ActiveModel {
            id: Set(vocab.node.id),
            parent_id: Set(vocab.node.parent_id),
            author_id: Set(vocab.node.author_id),
            r#type: Set("Vocabulary".to_string()),
            title: Set(vocab.node.title.clone()), // Usually "Word"
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

        // 2. Save Detail
        let detail_model = vocab_detail::ActiveModel {
            id: Set(vocab.node.id),
            word: Set(vocab.word),
            definition: Set(vocab.definition),
            translation: Set(vocab.translation),
            phonetic: Set(vocab.phonetic),
            language: Set(vocab.language),
            status: Set(vocab.status),
        };
        vocab_detail::Entity::insert(detail_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(vocab_detail::Column::Id)
                    .update_columns([vocab_detail::Column::Definition, vocab_detail::Column::Translation, vocab_detail::Column::Status])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(vocab.node.id)
    }

    async fn find_by_word(&self, user_id: &UserId, word: &str) -> Result<Option<Vocabulary>, RepositoryError> {
        // Find by word (requires Join with Node to check author/permission)
        // For simplicity, just finding matches and filtering in app or simple query
        // "Find my vocab word"
        let result = vocab_detail::Entity::find()
            .filter(vocab_detail::Column::Word.eq(word))
            .find_also_related(node::Entity)
            .all(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Filter for author match
        for (d, n_opt) in result {
             if let Some(n) = n_opt {
                 if n.author_id == user_id.0 {
                     return Ok(Some(map_vocab(n, d)));
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
            Some((n, Some(d))) => Ok(Some(map_vocab(n, d))),
            _ => Ok(None)
        }
    }

    async fn list(&self, user_id: &UserId, limit: u64, offset: u64, _query: Option<String>) -> Result<Vec<Vocabulary>, RepositoryError> {
        let results = node::Entity::find()
            .filter(node::Column::Type.eq("Vocabulary"))
            .filter(node::Column::AuthorId.eq(user_id.0)) // Only my vocab for now
            .find_also_related(vocab_detail::Entity)
            .limit(limit)
            .offset(offset)
            .order_by_desc(node::Column::CreatedAt)
            .all(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        // Map
         let mut vocabs = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                vocabs.push(map_vocab(n, detail));
            }
        }
        Ok(vocabs)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id).exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}

fn map_vocab(n: node::Model, d: vocab_detail::Model) -> Vocabulary {
    Vocabulary {
        node: Node {
             id: n.id,
            parent_id: n.parent_id,
            author_id: n.author_id,
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
    }
}
