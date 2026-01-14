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

use crate::domain::ports::{ArticleRepository, CommentRepository, MemoRepository, UserRepository, PermissionRepository};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::infrastructure::services::export_service::DataExportService;
use crate::domain::models::User;
use crate::interface::state::AppState;
use crate::interface::api::{
    auth, content, comment, memo, export, upload, tags, vocabulary, dictionary, knowledge_base, draft, permission, user, system
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

        CREATE TABLE IF NOT EXISTS groups (
            id UUID PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS relationships (
            id UUID PRIMARY KEY,
            entity_type TEXT NOT NULL,
            entity_id UUID NOT NULL,
            relation TEXT NOT NULL,
            subject_type TEXT NOT NULL,
            subject_id UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (entity_type, entity_id, relation, subject_type, subject_id)
        );

        CREATE INDEX IF NOT EXISTS idx_rels_entity ON relationships(entity_type, entity_id, relation);
        CREATE INDEX IF NOT EXISTS idx_rels_subject ON relationships(subject_type, subject_id, relation);
    ").await.expect("Failed to initialize users table");



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
            FOREIGN KEY(author_id) REFERENCES users(id),
            FOREIGN KEY(parent_id) REFERENCES nodes(id) ON DELETE CASCADE
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
            query_count INT NOT NULL DEFAULT 0,
            is_important BOOLEAN NOT NULL DEFAULT FALSE,
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
            cover_offset_y INT NOT NULL DEFAULT 50,
            visibility TEXT NOT NULL DEFAULT 'Private',
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            FOREIGN KEY(author_id) REFERENCES users(id) ON DELETE CASCADE
        );

        -- User Drafts (Server-Side Single Slot Cache)
        CREATE TABLE IF NOT EXISTS user_drafts (
            user_id UUID PRIMARY KEY,
            target_article_id UUID,
            title TEXT,
            body TEXT,
            tags TEXT,
            category TEXT,
            knowledge_base_id UUID,
            parent_id UUID,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
    ").await.expect("Failed to initialize Core Node schema");

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE nodes ADD COLUMN knowledge_base_id UUID REFERENCES knowledge_bases(id) ON DELETE SET NULL;"
    )).await.map_err(|e| println!("Migration note (node.kb_id): {}", e));

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE knowledge_bases ADD COLUMN cover_offset_y INT NOT NULL DEFAULT 50;"
    )).await.map_err(|e| println!("Migration note (kb.offset): {}", e));

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE knowledge_bases ADD COLUMN renderer_id TEXT;"
    )).await.map_err(|e| println!("Migration note (kb.renderer_id): {}", e));


    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE users ADD COLUMN experience JSONB DEFAULT '[]';"
    )).await.map_err(|e| println!("Migration note (user.experience): {}", e));

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE user_drafts ADD COLUMN parent_id UUID;"
    )).await.map_err(|e| println!("Migration note (draft.parent_id): {}", e));

    // Attempt to add Self-Referential FK for Cascade Delete (Postgres Only usually)
    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE nodes ADD CONSTRAINT fk_nodes_parent FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE;"
    )).await.map_err(|e| println!("Migration note (nodes.parent_fk): {}", e));

    // --- NEW VOCABULARY SCHEMA ---
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS vocab_roots (
            id UUID PRIMARY KEY,
            root TEXT UNIQUE NOT NULL,
            meaning TEXT
        );

        CREATE TABLE IF NOT EXISTS vocab_examples (
            id UUID PRIMARY KEY,
            vocab_id UUID NOT NULL, -- FK to vocab_details.id (which is same as node.id)
            sentence TEXT NOT NULL,
            translation TEXT,
            note TEXT,
            image_url TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vocab_id) REFERENCES vocab_details(id) ON DELETE CASCADE
        );
    ").await.map_err(|e| println!("Migration note (vocab_new_tables): {}", e));

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE vocab_details ADD COLUMN root_id UUID REFERENCES vocab_roots(id) ON DELETE SET NULL;"
    )).await.map_err(|e| println!("Migration note (vocab.root_id): {}", e));

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE vocab_details ADD COLUMN query_count INT NOT NULL DEFAULT 0;"
    )).await.map_err(|e| println!("Migration note (vocab.query_count): {}", e));

    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE vocab_details ADD COLUMN is_important BOOLEAN NOT NULL DEFAULT FALSE;"
    )).await.map_err(|e| println!("Migration note (vocab.is_important): {}", e));

    // --- SEMANTIC INDEX (Math KB V2) ---
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS semantic_nodes (
            id UUID PRIMARY KEY,
            article_id UUID NOT NULL,
            client_id TEXT NOT NULL, -- The 'id' in the markdown block
            type TEXT NOT NULL,      -- theorem, function, etc.
            title TEXT,
            content TEXT,
            metrics JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (article_id) REFERENCES nodes(id) ON DELETE CASCADE,
            UNIQUE (article_id, client_id)
        );

        CREATE TABLE IF NOT EXISTS semantic_edges (
            id UUID PRIMARY KEY,
            source_id UUID NOT NULL,
            target_id UUID NOT NULL,
            relation_type TEXT NOT NULL,
            metrics JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (source_id) REFERENCES semantic_nodes(id) ON DELETE CASCADE,
            FOREIGN KEY (target_id) REFERENCES semantic_nodes(id) ON DELETE CASCADE
        );
    ").await.map_err(|e| println!("Migration note (semantic_index): {}", e));

    // --- GRAPH NODES (Phase 7: Manual Graph Editing) ---
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS graph_nodes (
            id UUID PRIMARY KEY,
            knowledge_base_id UUID NOT NULL,
            parent_id UUID,
            label TEXT NOT NULL,
            data JSONB DEFAULT '{}',
            rank INT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES graph_nodes(id) ON DELETE SET NULL
        );
    ").await.map_err(|e| println!("Migration note (graph_nodes): {}", e));


    // Initialize Auth Service
    let repo = Arc::new(PostgresRepository::new(db.clone()));
    let auth_service = Arc::new(Arg2JwtAuthService::new(
        repo.clone() as Arc<dyn UserRepository>,
        env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string())
    ));

    tracing::info!("Checking for admin user...");
    // Use timeout to prevent startup hangs on DB locks
    let admin_name = "admin";
    let existing_admin = match tokio::time::timeout(std::time::Duration::from_secs(5), repo.find_by_username(admin_name)).await {
        Ok(res) => res.unwrap_or_else(|e| {
            tracing::error!("Failed to fetch admin user: {}", e);
            None
        }),
        Err(_) => {
            tracing::error!("Timeout fetching admin user - Database might be locked or slow.");
            None
        }
    };

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
            experience: None,
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

    let permission_service = crate::domain::permission_service::PermissionService::new(repo.clone());

    // Initialize Public Group
    let public_group_id = uuid::Uuid::nil(); 
    match repo.create_group(public_group_id, "public".to_string()).await {
        Ok(_) => tracing::info!("Public group initialized"),
        Err(e) => tracing::warn!("Public group init: {}", e),
    }
    
    let dictionary_cache = moka::future::Cache::builder()
        .max_capacity(1000)
        .time_to_live(std::time::Duration::from_secs(3600))
        .build();

    let indexer_service = Arc::new(crate::domain::indexer_service::IndexerService::new(db.clone()));
    let graph_service = Arc::new(crate::domain::graph_service::GraphService::new(repo.clone() as Arc<dyn crate::domain::ports::GraphRepository>));

    let state = AppState {
        repo,
        auth_service,
        export_service,
        permission_service,
        dictionary,
        dictionary_cache,
        indexer_service,
        graph_service,
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
        .merge(draft::router())
        .merge(permission::router())
        .merge(user::router())
        .merge(system::router())
        .merge(crate::interface::api::graph::router())
        .with_state(state);

    let app = Router::new()
        .route("/", get(health_check))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .merge(api_routes)
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB Limit
        .layer(TraceLayer::new_for_http()); // Magic happens here: Automatic logging for every request
    
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Aether Core online at {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Aether Systems Operational"
}
