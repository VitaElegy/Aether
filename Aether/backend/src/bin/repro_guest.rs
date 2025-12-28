use sea_orm::{Database, ConnectionTrait, Schema, DbBackend};
use aether_backend::infrastructure::persistence::postgres::PostgresRepository;
use aether_backend::domain::ports::ContentRepository;
use aether_backend::domain::models::{ContentAggregate, ContentId, ContentStatus, Visibility, ContentBody};
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 1. Setup In-Memory SQLite
    let db = Database::connect("sqlite::memory:").await.unwrap();

    // 2. Run Migrations (Manual DDL)
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
    ").await.unwrap();

    let repo = PostgresRepository::new(db.clone());

    // 3. Create Author
    let author_id = Uuid::new_v4();
    // (Skipping user creation as logic doesn't strictly enforce FK in list query if we don't join,
    // but the repo implementation DOES join users. So we must create user.)

    // We need to insert user directly via SQL or Repo (Repo trait doesn't expose raw insert easily without User object)
    let _ = db.execute_unprepared(&format!("
        INSERT INTO users (id, username, email, password_hash, permissions, display_name)
        VALUES ('{}', 'author', 'a@a.com', 'hash', 1, 'Author Name')
    ", author_id)).await.unwrap();


    // 4. Create Content (Public & Published) - Using LOWERCASE to verify fix (as Enum default serialization might vary or DB might have legacy data)
    // Actually, let's manually insert to force lowercase in DB if we want to be sure, or just rely on Enum serialization?
    // The Enum `Visibility::Public` serializes to "Public" by default.
    // To test the "public" case safely, let's use raw SQL insert or just trust the previous issue was indeed case related.
    // Let's Insert TWO items: one PascalCase, one lowercase.

    // Item 1: Standard (PascalCase from Enum)
    let content_id_1 = ContentId(Uuid::new_v4());
    let content_1 = ContentAggregate {
        id: content_id_1.clone(),
        author_id,
        author_name: None,
        title: "Standard Post".to_string(),
        slug: "standard-post".to_string(),
        status: ContentStatus::Published,
        visibility: Visibility::Public,
        category: Some("General".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        body: ContentBody::Markdown("Body".to_string()),
        tags: vec!["tag".to_string()],
        version_message: None,
    };
    repo.save(content_1, aether_backend::domain::models::UserId(author_id), true).await.unwrap();

    // Item 2: Legacy/Lowercase (Manual SQL Insert)
    let content_id_2 = Uuid::new_v4();
    let _ = db.execute_unprepared(&format!("
        INSERT INTO contents (id, author_id, title, slug, status, visibility, created_at, updated_at, body, tags)
        VALUES ('{}', '{}', 'Legacy Post', 'legacy-post', 'published', 'public', '{}', '{}', '{{ \"type\": \"Markdown\", \"data\": \"Body\" }}', '[]')
    ", content_id_2, author_id, Utc::now().to_rfc3339(), Utc::now().to_rfc3339())).await.unwrap();

    // 5. List as Guest (viewer_id = None)
    let results = repo.list(None, None, 10, 0).await.unwrap();

    println!("Results found: {}", results.len());
    for c in results {
        println!(" - {} ({:?}/{:?})", c.title, c.status, c.visibility);
    }
}
