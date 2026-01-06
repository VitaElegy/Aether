use axum::{
    routing::get,
    Router,
};
use tower_http::{
    trace::TraceLayer,
    services::ServeDir,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use sea_orm::{Database, DatabaseConnection, ConnectionTrait};
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;
mod domain;
mod infrastructure;
mod interface;

use crate::domain::ports::{UserRepository, ContentRepository, CommentRepository, MemoRepository};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::infrastructure::services::export_service::DataExportService;
use crate::domain::models::User;
use crate::interface::state::AppState;
use crate::interface::api::{
    auth, content, comment, memo, export, upload, knowledge_base, tags, vocabulary
};



#[tokio::main]
async fn main() {
    // Simple logging setup
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: DatabaseConnection = Database::connect(db_url).await.expect("Failed to connect to DB");

    // ... DB Init & Seeding (Same as before) ...
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            permissions BIGINT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS contents (
            id UUID PRIMARY KEY,
            author_id UUID NOT NULL,
            title TEXT NOT NULL,
            slug TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL,
            visibility TEXT NOT NULL DEFAULT 'Public',
            category TEXT,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            body JSONB NOT NULL,
            body JSONB NOT NULL,
            tags TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS content_versions (
            id UUID PRIMARY KEY,
            content_id UUID NOT NULL,
            version INT NOT NULL,
            title TEXT NOT NULL,
            body JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            change_reason TEXT,
            content_hash TEXT NOT NULL,
            editor_id UUID NOT NULL
        );
        CREATE TABLE IF NOT EXISTS comments (
            id UUID PRIMARY KEY,
            target_type TEXT NOT NULL,
            target_id TEXT NOT NULL,
            user_id UUID NOT NULL,
            parent_id UUID,
            text TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL
        );
        CREATE TABLE IF NOT EXISTS memos (
            id UUID PRIMARY KEY,
            author_id UUID NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            visibility TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS knowledge_bases (
            id UUID PRIMARY KEY,
            author_id UUID NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );
    ").await.expect("Failed to initialize DB schema");

    // Migration: Add columns (Resilient logic for SQLite/Postgres)
    // We attempt to add columns one by one. If they exist or syntax fails (e.g. IF NOT EXISTS on old SQLite), we catch it.
    let migrations = vec![
        "ALTER TABLE users ADD COLUMN display_name TEXT",
        "ALTER TABLE users ADD COLUMN bio TEXT",
        "ALTER TABLE users ADD COLUMN avatar_url TEXT",
        "ALTER TABLE content_versions ADD COLUMN change_reason TEXT",
        "ALTER TABLE content_versions ADD COLUMN content_hash TEXT DEFAULT ''",
        "ALTER TABLE content_versions ADD COLUMN editor_id UUID",
        // Comment Migration
        "ALTER TABLE comments ADD COLUMN target_type TEXT DEFAULT 'Content'",
        "ALTER TABLE comments ADD COLUMN target_id TEXT",
        "UPDATE comments SET target_id = content_id WHERE target_id IS NULL AND content_id IS NOT NULL",
        // Knowledge Base Migration
        "ALTER TABLE contents ADD COLUMN knowledge_base_id UUID",
        "ALTER TABLE knowledge_bases ADD COLUMN tags TEXT DEFAULT '[]'",
        "ALTER TABLE knowledge_bases ADD COLUMN cover_image TEXT",
        "ALTER TABLE knowledge_bases ADD COLUMN visibility TEXT DEFAULT 'Private'",
        "ALTER TABLE contents ADD COLUMN parent_id UUID",
        "ALTER TABLE contents ADD COLUMN content_type TEXT DEFAULT 'Article'",
        // Fix Schema Mismatches
        "ALTER TABLE comments ALTER COLUMN target_id TYPE TEXT",
        "ALTER TABLE contents ALTER COLUMN tags TYPE TEXT",
        // Note: We leave content_id for now as dropping columns in SQLite can be tricky depending on version,
        // and we want to be safe. It becomes zombie column.
    ];

    for sql in migrations {
        if let Err(e) = db.execute_unprepared(sql).await {
            tracing::info!("Migration note (safe to ignore if column exists): {} | Error: {}", sql, e);
        }
    }

    // Editor ID Backfill (Safe to run multiple times)
    let _ = db.execute_unprepared("UPDATE content_versions SET editor_id = '00000000-0000-0000-0000-000000000000' WHERE editor_id IS NULL;").await;

    // Attempt constraint update (Will fail on SQLite, succeed on Postgres)
    let _ = db.execute_unprepared("ALTER TABLE content_versions ALTER COLUMN editor_id SET NOT NULL;").await;

    // Vocabulary Table
    db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS vocabularies (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            word TEXT NOT NULL,
            definition TEXT NOT NULL,
            context_sentence TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
    ").await.expect("Failed to create vocabularies table");

    let repo = Arc::new(PostgresRepository::new(db.clone()));
    let auth_service = Arc::new(Arg2JwtAuthService::new(
        repo.clone() as Arc<dyn UserRepository>,
        env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string())
    ));

    let admin_name = "admin";
    let existing_admin = repo.find_by_username(admin_name).await.unwrap();

    // Always update admin if it exists to ensure new fields are populated, or create if missing
    if let Some(mut admin) = existing_admin {
        tracing::info!("Updating admin user...");
        if admin.display_name.is_none() { admin.display_name = Some("Administrator".to_string()); }
        if admin.bio.is_none() { admin.bio = Some("The system architect.".to_string()); }
        UserRepository::save(&*repo, admin).await.expect("Failed to update admin");
    } else {
        tracing::info!("Seeding admin user...");
        let hash = crate::infrastructure::auth::jwt_service::hash_password("admin");
        let admin = User {
            id: crate::domain::models::UserId(Uuid::new_v4()),
            username: admin_name.to_string(),
            email: "admin@aether.io".to_string(),
            display_name: Some("Administrator".to_string()),
            bio: Some("The system architect.".to_string()),
            avatar_url: None,
            password_hash: hash,
            permissions: u64::MAX,
        };
        UserRepository::save(&*repo, admin).await.expect("Failed to seed admin");
    }

    let export_service = Arc::new(DataExportService::new(
        repo.clone() as Arc<dyn ContentRepository>,
        repo.clone() as Arc<dyn CommentRepository>,
        repo.clone() as Arc<dyn MemoRepository>,
    ));

    let state = AppState {
        repo,
        auth_service,
        export_service,
    };

    // --- 4. Build Router with Trace Middleware ---
    let api_routes = auth::router()
        .merge(content::router())
        .merge(comment::router())
        .merge(memo::router())
        .merge(knowledge_base::router())
        .merge(export::router())
        .merge(upload::router())
        .merge(tags::router())
        .merge(vocabulary::router())
        .with_state(state);

    let app = Router::new()
        .route("/", get(health_check))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .merge(api_routes)
        .layer(TraceLayer::new_for_http()); // Magic happens here: Automatic logging for every request

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Aether Core online at {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Aether Systems Operational"
}
