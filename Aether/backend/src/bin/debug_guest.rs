use sea_orm::{Database, ConnectionTrait, ConnectOptions, DbBackend};
use aether_backend::infrastructure::persistence::postgres::PostgresRepository;
use aether_backend::domain::ports::ContentRepository;
use aether_backend::domain::models::{ContentAggregate, ContentId, ContentStatus, Visibility, ContentBody};
use chrono::Utc;
use uuid::Uuid;
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 1. Setup In-Memory SQLite with Logging
    let mut opt = ConnectOptions::new("sqlite::memory:".to_owned());
    opt.sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt).await.unwrap();

    // 2. Schema
    let _ = db.execute_unprepared("
        CREATE TABLE users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            permissions BIGINT NOT NULL,
            display_name TEXT,
            bio TEXT,
            avatar_url TEXT
        );
        CREATE TABLE contents (
            id TEXT PRIMARY KEY,
            author_id TEXT NOT NULL,
            title TEXT NOT NULL,
            slug TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL,
            visibility TEXT NOT NULL DEFAULT 'Public',
            category TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            body TEXT NOT NULL,
            tags TEXT NOT NULL
        );
        CREATE TABLE content_versions (
            id TEXT PRIMARY KEY,
            content_id TEXT NOT NULL,
            version INT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            change_reason TEXT,
            content_hash TEXT,
            editor_id TEXT
        );
         CREATE TABLE comments (
            id TEXT PRIMARY KEY,
            content_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            parent_id TEXT,
            text TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
    ").await.unwrap();

    let repo = PostgresRepository::new(db.clone());

    // 3. Data Setup
    let author_id = Uuid::new_v4();
    let _ = db.execute_unprepared(&format!("
        INSERT INTO users (id, username, email, password_hash, permissions, display_name)
        VALUES ('{}', 'author', 'a@a.com', 'hash', 1, 'Author Name')
    ", author_id)).await.unwrap();

    // Insert Mixed Case Data
    let content_id = Uuid::new_v4();
    let _ = db.execute_unprepared(&format!("
        INSERT INTO contents (id, author_id, title, slug, status, visibility, created_at, updated_at, body, tags)
        VALUES ('{}', '{}', 'Test Post', 'test-post', 'Published', 'Public', '{}', '{}', '{{ \"type\": \"Markdown\", \"data\": \"Body\" }}', '[]')
    ", content_id, author_id, Utc::now().to_rfc3339(), Utc::now().to_rfc3339())).await.unwrap();

    println!("--- Testing Guest Access ---");
    match repo.list(None, None, 10, 0).await {
        Ok(results) => {
            println!("Results: {}", results.len());
            for c in results {
                println!("Found: {}", c.title);
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }
}
