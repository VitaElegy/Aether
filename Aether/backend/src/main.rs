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

use crate::domain::ports::{ArticleRepository, CommentRepository, MemoRepository, UserRepository};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::infrastructure::services::export_service::DataExportService;
use crate::domain::models::User;
use crate::interface::state::AppState;
use crate::interface::api::{
    auth, content, comment, memo, export, upload, tags, vocabulary, dictionary, knowledge_base
};



#[tokio::main]
async fn main() {
    // Simple logging setup
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: DatabaseConnection = Database::connect(db_url).await.expect("Failed to connect to DB");

    // ... DB Init & Seeding (Same as before) ...
    // --- DROPPING TABLES FOR SCHEMA RESET (Phase 1 Refactor) ---
    // User approved "Fresh Start". We destroy old tables to rebuild the "Linux Kernel" architecture.
    // let _ = db.execute_unprepared("
    //     DROP TABLE IF EXISTS comments;
    //     DROP TABLE IF EXISTS content_versions;
    //     DROP TABLE IF EXISTS knowledge_bases;
    //     DROP TABLE IF EXISTS contents;      -- Old Articles
    //     DROP TABLE IF EXISTS vocabularies;  -- Old Vocab
    //     DROP TABLE IF EXISTS memos;         -- Old Memos
    //     DROP TABLE IF EXISTS article_details;
    //     DROP TABLE IF EXISTS vocab_details;
    //     DROP TABLE IF EXISTS memo_details;
    //     DROP TABLE IF EXISTS nodes;
    //     DROP TABLE IF EXISTS users;
    // ").await;

    // --- RECREATING SCHEMA ---
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            display_name TEXT,
            bio TEXT,
            avatar_url TEXT,
            permissions BIGINT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
    ").await.expect("Failed to initialize users table");

    // Update schema with migration-like logic for Knowledge Base Link
    // Safe to run repeatedly (ignore error if column exists)
    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE nodes ADD COLUMN knowledge_base_id UUID REFERENCES knowledge_bases(id) ON DELETE SET NULL;"
    )).await.map_err(|e| println!("Migration warning (likely exists): {}", e));

    let _ = db.execute_unprepared("
        -- The Kernel (Base Node)
        CREATE TABLE IF NOT EXISTS nodes (
            id UUID PRIMARY KEY,
            parent_id UUID,
            author_id UUID NOT NULL,
            knowledge_base_id UUID,
            type TEXT NOT NULL, -- 'article', 'vocabulary', 'memo', 'folder'
            title TEXT NOT NULL, -- Lifted title to generic node for consistent displaying
            permission_mode TEXT NOT NULL DEFAULT 'Public', -- Public/Private/Internal
            permission_data TEXT DEFAULT '{}', -- Advanced ACLs
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            FOREIGN KEY(author_id) REFERENCES users(id)
        );

        -- File System Driver: Articles
        CREATE TABLE IF NOT EXISTS article_details (
            id UUID PRIMARY KEY, -- FK to nodes.id
            slug TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL,
            category TEXT,
            body JSONB NOT NULL,
            tags TEXT NOT NULL,
            FOREIGN KEY(id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- File System Driver: Vocabularies
        CREATE TABLE IF NOT EXISTS vocab_details (
            id UUID PRIMARY KEY, -- FK to nodes.id
            word TEXT NOT NULL,
            definition TEXT NOT NULL,
            translation TEXT,
            phonetic TEXT,
            language TEXT NOT NULL DEFAULT 'en',
            status TEXT NOT NULL,
            FOREIGN KEY(id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- File System Driver: Memos
        CREATE TABLE IF NOT EXISTS memo_details (
            id UUID PRIMARY KEY, -- FK to nodes.id
            content TEXT NOT NULL,
            priority TEXT DEFAULT 'Medium', -- High/Medium/Low
            tags TEXT NOT NULL,
            FOREIGN KEY(id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- Versioning (Linked to Nodes)
        CREATE TABLE IF NOT EXISTS content_versions (
            id UUID PRIMARY KEY,
            node_id UUID NOT NULL,
            version INT NOT NULL,
            title TEXT NOT NULL,
            body JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            change_reason TEXT,
            content_hash TEXT NOT NULL,
            editor_id UUID NOT NULL,
            FOREIGN KEY(node_id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- Comments (Linked to Nodes)
        CREATE TABLE IF NOT EXISTS comments (
            id UUID PRIMARY KEY,
            target_id UUID NOT NULL, -- Links to nodes.id
            user_id UUID NOT NULL,
            parent_id UUID,
            text TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            FOREIGN KEY(target_id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- Knowledge Bases (Independent, but can contain Nodes)
        CREATE TABLE IF NOT EXISTS knowledge_bases (
            id UUID PRIMARY KEY,
            author_id UUID NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            tags JSONB NOT NULL DEFAULT '[]',
            cover_image TEXT,
            visibility TEXT NOT NULL DEFAULT 'Private',
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            FOREIGN KEY(author_id) REFERENCES users(id) ON DELETE CASCADE
        );
    ").await.expect("Failed to initialize Core Node schema");



    // Initialize Auth Service
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
        repo.clone() as Arc<dyn ArticleRepository>,
        repo.clone() as Arc<dyn MemoRepository>,
        repo.clone() as Arc<dyn CommentRepository>,
    ));

    use crate::infrastructure::dictionary::loader::DictionaryLoader;
    let dictionary = DictionaryLoader::new("data/dictionary");

    let state = AppState {
        repo,
        auth_service,
        export_service,
        dictionary,
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
        .merge(dictionary::router())
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
