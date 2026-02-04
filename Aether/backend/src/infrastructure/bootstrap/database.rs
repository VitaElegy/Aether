use std::env;
use sea_orm::{Database, DatabaseConnection, ConnectionTrait};

pub async fn init_pool() -> DatabaseConnection {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&db_url).await.expect("Failed to connect to DB");
    tracing::info!("Database Connection Established");
    db
}

pub async fn run_migrations(db: &DatabaseConnection) {
    tracing::info!("Running Schema Migrations from ./migrations directory...");

    let migrations_dir = std::path::Path::new("migrations");
    if !migrations_dir.exists() {
        tracing::warn!("migrations directory not found at ./migrations");
        return;
    }

    let mut paths: Vec<_> = std::fs::read_dir(migrations_dir)
        .expect("Failed to read migrations directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "sql"))
        .collect();

    // Sort to ensure order (e.g., 000_..., 001_...)
    paths.sort();

    for path in paths {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        tracing::info!("Applying migration: {}", filename);
        
        match std::fs::read_to_string(&path) {
            Ok(sql) => {
                // Execute the raw SQL. 
                // Note: execute_unprepared handles multiple statements for SQLite/Postgres in SQLx usually.
                if let Err(e) = db.execute_unprepared(&sql).await {
                    tracing::error!("Migration failed for {}: {}", filename, e);
                    // We continue, as some statements might be "IF NOT EXISTS" or fail harmlessly 
                    // (like specific ALTERs in dev). In production, this should likely panic or be transactional.
                }
            },
            Err(e) => tracing::error!("Failed to read file {}: {}", filename, e),
        }
    }

    tracing::info!("Schema Migrations Complete.");
}

// --- BULK MIGRATION ---
pub async fn run_bulk_migration(db: DatabaseConnection) {
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, TransactionTrait};
    use crate::infrastructure::persistence::entities::{article_detail, blocks};
    use crate::domain::blocks::parser::parse_markdown_to_blocks;

    // Fetch all articles
    let articles = article_detail::Entity::find().all(&db).await.expect("Failed to fetch articles");
    tracing::info!("Found {} articles to migrate.", articles.len());

    for article in articles {
        // Only migrate Markdown content
        let body_json = article.body;
        // Check if body acts like Markdown (string or object with 'markdown' key)
        let md_text = if let Some(t) = body_json.as_str() {
             t.to_string()
        } else if let Some(t) = body_json.get("markdown").and_then(|v| v.as_str()) {
             t.to_string()
        } else if let Some(content) = body_json.get("content").and_then(|v| v.as_str()) {
             // Tiptap or other formats might differ, but assuming simple here
             content.to_string()
        } else {
             // Fallback: try to serialize generic JSON to string
             serde_json::to_string_pretty(&body_json).unwrap_or_default()
        };

        if md_text.is_empty() {
            tracing::warn!("Skipping empty article: {}", article.id);
            continue;
        }

        let blocks_vec = parse_markdown_to_blocks(article.id, &md_text);
        
        if blocks_vec.is_empty() {
             tracing::info!("No blocks parsed for article: {}", article.id);
             continue;
        }

        // Transactional Replace
        let txn = db.begin().await.expect("Txn begin failed");
        
        // 1. Delete existing
        let del_res = blocks::Entity::delete_many()
            .filter(blocks::Column::DocumentId.eq(article.id))
            .exec(&txn)
            .await;
            
        if let Err(e) = del_res {
             tracing::error!("Failed to clean blocks for {}: {}", article.id, e);
             continue; 
        }

        // 2. Insert new
        let active_blocks: Vec<blocks::ActiveModel> = blocks_vec.into_iter().map(|mut b| {
            // Ensure search trait application 
            crate::domain::blocks::strategies::apply_searchable_trait(&mut b);

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

        if let Err(e) = blocks::Entity::insert_many(active_blocks).exec(&txn).await {
            tracing::error!("Failed to insert blocks for {}: {}", article.id, e);
        } else {
            if let Err(e) = txn.commit().await {
                 tracing::error!("Failed to commit migration for {}: {}", article.id, e);
            } else {
                 tracing::info!("Migrated article: {} ({} blocks)", article.id, article.slug);
            }
        }
    }
    tracing::info!("Migration Complete.");
}
