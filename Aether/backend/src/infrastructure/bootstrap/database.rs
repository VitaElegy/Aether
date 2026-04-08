use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use std::env;
use std::time::Duration;

pub async fn init_pool() -> DatabaseConnection {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let min_connections: u32 = env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    let connect_timeout_secs: u64 = env::var("DB_CONNECT_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);
    let idle_timeout_secs: u64 = env::var("DB_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let mut opts = ConnectOptions::new(&db_url);
    opts.max_connections(max_connections)
        .min_connections(min_connections)
        .connect_timeout(Duration::from_secs(connect_timeout_secs))
        .idle_timeout(Duration::from_secs(idle_timeout_secs))
        .sqlx_logging(false);

    let db = Database::connect(opts)
        .await
        .expect("Failed to connect to DB");
    tracing::info!(
        "Database Connection Pool Established (max={}, min={}, connect_timeout={}s, idle_timeout={}s)",
        max_connections,
        min_connections,
        connect_timeout_secs,
        idle_timeout_secs,
    );
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
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .collect();

    paths.sort();

    for path in paths {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        tracing::info!("Applying migration: {}", filename);

        match std::fs::read_to_string(&path) {
            Ok(sql) => {
                // Execute each statement independently for resilience.
                let statements: Vec<&str> = sql
                    .split(';')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty() && !s.starts_with("--"))
                    .collect();

                let mut errors = 0u32;
                for stmt in &statements {
                    let exec_sql = format!("{};", stmt);
                    if let Err(e) = db.execute_unprepared(&exec_sql).await {
                        tracing::warn!("  [skip] {}: {}", filename, e);
                        errors += 1;
                    }
                }
                if errors > 0 {
                    tracing::warn!("  {} had {} non-fatal statement error(s)", filename, errors);
                }
            }
            Err(e) => tracing::error!("Failed to read file {}: {}", filename, e),
        }
    }

    tracing::info!("Schema Migrations Complete.");
}

// --- BULK MIGRATION ---
pub async fn run_bulk_migration(db: DatabaseConnection) {
    use crate::domain::blocks::parser::parse_markdown_to_blocks;
    use crate::infrastructure::persistence::entities::{article_detail, blocks};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};

    // Fetch all articles
    let articles = article_detail::Entity::find()
        .all(&db)
        .await
        .expect("Failed to fetch articles");
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
        let active_blocks: Vec<blocks::ActiveModel> = blocks_vec
            .into_iter()
            .map(|mut b| {
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
            })
            .collect();

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
