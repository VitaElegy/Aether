use sea_orm::*;
use uuid::Uuid;
use crate::infrastructure::persistence::entities::{blocks, node};
use crate::domain::blocks::models::Block;
use crate::domain::blocks::strategies::apply_searchable_trait;

pub struct BlockRepository {
    db: DatabaseConnection,
}

impl BlockRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    #[allow(dead_code)]
    pub async fn save_blocks(&self, document_id: Uuid, blocks: Vec<Block>) -> Result<(), DbErr> {
        let txn = self.db.begin().await?;

        // 1. Delete existing blocks for this document (Strategy: Replace All)
        blocks::Entity::delete_many()
            .filter(blocks::Column::DocumentId.eq(document_id))
            .exec(&txn)
            .await?;

        // 2. Insert new blocks
        if !blocks.is_empty() {
             let active_models: Vec<blocks::ActiveModel> = blocks.into_iter().map(|mut b| {
                // Apply Search Strategy to populate 'text_mirror'
                apply_searchable_trait(&mut b);

                blocks::ActiveModel {
                    id: Set(b.id),
                    document_id: Set(b.document_id),
                    r#type: Set(b.type_name),
                    ordinal: Set(b.ordinal),
                    revision: Set(b.revision),
                    payload: Set(b.payload),
                    created_at: Set(b.created_at.into()),
                    updated_at: Set(b.updated_at.into()),
                }
            }).collect();

            // Insert matching blocks
            // Note: If the list is huge, we might need batching. For now, assuming reasonable doc size.
            blocks::Entity::insert_many(active_models)
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn find_by_document_id(&self, document_id: Uuid) -> Result<Vec<Block>, DbErr> {
        let models = blocks::Entity::find()
            .filter(blocks::Column::DocumentId.eq(document_id))
            .order_by_asc(blocks::Column::Ordinal)
            .all(&self.db)
            .await?;
            
        Ok(models.into_iter().map(|m| Block {
            id: m.id,
            document_id: m.document_id,
            type_name: m.r#type,
            ordinal: m.ordinal,
            revision: m.revision,
            payload: m.payload,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }).collect())
    }

    pub async fn find_by_kb_id(&self, kb_id: Uuid) -> Result<Vec<Block>, DbErr> {
         let models = blocks::Entity::find()
            .join(JoinType::InnerJoin, blocks::Relation::Node.def())
            .filter(node::Column::KnowledgeBaseId.eq(kb_id))
            .all(&self.db)
            .await?;

         Ok(models.into_iter().map(|m| Block {
            id: m.id,
            document_id: m.document_id,
            type_name: m.r#type,
            ordinal: m.ordinal,
            revision: m.revision,
            payload: m.payload,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }).collect())
    }
    
    // Feature: Search Global via Mandatory Trait is implemented via the 'plain_text' column in PG
    // But since plain_text is a generated column in DB, we don't write to it in Rust.
    // However, SeaORM ActiveModel doesn't know it's generated unless marked.
    // We didn't include `plain_text` in `ActiveModel` fields above (Set), which is correct for generated columns.
}
