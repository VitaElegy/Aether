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
    auth, content, comment, memo, export, upload, tags, vocabulary, dictionary, knowledge_base, permission, user, system, template
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
    // --- DROPPING TABLES FOR SCHEMA RESET (Phase 1 Refactor) ---
    // User approved "Fresh Start". We destroy old tables to rebuild the "Linux Kernel" architecture.
    // --- DROPPING TABLES FOR SCHEMA RESET (Phase 1 Refactor) ---
    // User approved "Fresh Start". We destroy old tables to rebuild the "Linux Kernel" architecture.
    let _ = db.execute_unprepared("
        DROP TABLE IF EXISTS comments;
        DROP TABLE IF EXISTS content_versions;
        DROP TABLE IF EXISTS knowledge_bases;
        DROP TABLE IF EXISTS contents;      -- Old Articles
        DROP TABLE IF EXISTS vocabularies;  -- Old Vocab
        DROP TABLE IF EXISTS memos;         -- Old Memos
        DROP TABLE IF EXISTS article_details;
        DROP TABLE IF EXISTS vocab_details;
        DROP TABLE IF EXISTS memo_details;
        DROP TABLE IF EXISTS nodes;
        DROP TABLE IF EXISTS users;
    ").await;

    // --- RECREATING SCHEMA ---
// ... (Lines 59-165 kept implicitly as we only replace the block around line 44-56 and the memos table)
// Wait, I cannot edit non-contiguous blocks with replace_file_content.
// I will use multi_replace_file_content.


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
            experience JSONB DEFAULT '[]', -- Added directly
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
        -- Knowledge Bases (Independent, but can contain Nodes)
        CREATE TABLE IF NOT EXISTS knowledge_bases (
            id UUID PRIMARY KEY,
            author_id UUID NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            tags JSONB NOT NULL DEFAULT '[]',
            cover_image TEXT,
            cover_offset_y INT NOT NULL DEFAULT 50, -- Added directly
            renderer_id TEXT, -- Added directly
            visibility TEXT NOT NULL DEFAULT 'Private',
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            FOREIGN KEY(author_id) REFERENCES users(id) ON DELETE CASCADE
        );

        -- The Kernel (Base Node)
        CREATE TABLE IF NOT EXISTS nodes (
            id UUID PRIMARY KEY,
            parent_id UUID,
            author_id UUID NOT NULL,
            knowledge_base_id UUID, -- Added directly
            type TEXT NOT NULL, -- 'article', 'vocabulary', 'memo', 'folder'
            title TEXT NOT NULL, -- Lifted title to generic node for consistent displaying
            permission_mode TEXT NOT NULL DEFAULT 'Public', -- Public/Private/Internal
            permission_data TEXT DEFAULT '{}', -- Advanced ACLs
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            FOREIGN KEY(author_id) REFERENCES users(id),
            FOREIGN KEY(parent_id) REFERENCES nodes(id) ON DELETE CASCADE,
            FOREIGN KEY(knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE SET NULL
        );

        -- File System Driver: Articles
        CREATE TABLE IF NOT EXISTS article_details (
            id UUID PRIMARY KEY, -- FK to nodes.id
            slug TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL,
            category TEXT,
            body JSONB NOT NULL,
            tags TEXT NOT NULL,
            derived_data JSONB, -- Added directly
            FOREIGN KEY(id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- File System Driver: Vocab_Roots
        CREATE TABLE IF NOT EXISTS vocab_roots (
            id UUID PRIMARY KEY,
            root TEXT UNIQUE NOT NULL,
            meaning TEXT
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
            root_id UUID REFERENCES vocab_roots(id) ON DELETE SET NULL, -- Added directly
            query_count INT NOT NULL DEFAULT 0, -- Added directly
            is_important BOOLEAN NOT NULL DEFAULT FALSE, -- Added directly
            FOREIGN KEY(id) REFERENCES nodes(id) ON DELETE CASCADE
        );
        
        -- File System Driver: Memos
        CREATE TABLE IF NOT EXISTS memo_details (
            id UUID PRIMARY KEY, -- FK to nodes.id
            project_id UUID,
            color TEXT NOT NULL,
            is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
            content JSONB NOT NULL,
            status TEXT NOT NULL,
            priority TEXT NOT NULL,
            due_at TIMESTAMPTZ,
            reminder_at TIMESTAMPTZ,
            tags JSONB NOT NULL DEFAULT '[]',
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

        -- User Drafts (Server-Side Single Slot Cache)
        CREATE TABLE IF NOT EXISTS user_drafts (
            user_id UUID PRIMARY KEY,
            target_article_id UUID,
            title TEXT,
            body TEXT,
            tags TEXT,
            category TEXT,
            knowledge_base_id UUID,
            parent_id UUID, -- Added directly
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS vocab_examples (
            id UUID PRIMARY KEY,
            vocab_id UUID NOT NULL, -- FK to vocab_details.id
            sentence TEXT NOT NULL,
            translation TEXT,
            note TEXT,
            image_url TEXT,
            article_id UUID, -- Added directly
            sentence_uuid UUID, -- Added directly
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vocab_id) REFERENCES vocab_details(id) ON DELETE CASCADE
        );
    ").await.expect("Failed to initialize Core Node schema");

    // --- DRAFTS SYSTEM (Shadow Drafts) ---
    let _ = db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS drafts (
            article_id UUID PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            body JSONB NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        );"
    )).await.map_err(|e| println!("Migration note (drafts table): {}", e));


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

    // --- VRKB Schema (Phase 5) ---
    // Project-Centric Vulnerability Research Knowledge Base
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS vrkb_projects (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            repository_url TEXT,
            quota_bytes BIGINT NOT NULL DEFAULT 5368709120, -- Default 5GB
            settings JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS vrkb_sections (
            id UUID PRIMARY KEY,
            project_id UUID NOT NULL,
            title TEXT NOT NULL,
            checklist JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS vrkb_findings (
            id UUID PRIMARY KEY,
            section_id UUID NOT NULL,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            severity TEXT NOT NULL,
            content JSONB,
            is_triage BOOLEAN NOT NULL DEFAULT FALSE,
            author_id UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (section_id) REFERENCES vrkb_sections(id) ON DELETE CASCADE,
            FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS vrkb_assets (
            id UUID PRIMARY KEY,
            hash TEXT UNIQUE NOT NULL,
            storage_path TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            size_bytes BIGINT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS vrkb_project_assets (
            project_id UUID NOT NULL,
            asset_id UUID NOT NULL,
            virtual_path TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (project_id, asset_id),
            FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
            FOREIGN KEY (asset_id) REFERENCES vrkb_assets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS vrkb_members (
            project_id UUID NOT NULL,
            user_id UUID NOT NULL,
            role TEXT NOT NULL, -- 'Owner', 'Admin', 'Member', 'Viewer'
            joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (project_id, user_id),
            FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS vrkb_specs (
            id UUID PRIMARY KEY,
            project_id UUID NOT NULL,
            title TEXT NOT NULL,
            content TEXT, -- Markdown content
            version INT NOT NULL DEFAULT 1,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS vrkb_docs (
            id UUID PRIMARY KEY,
            project_id UUID NOT NULL,
            title TEXT NOT NULL,
            content JSONB, -- Block-based content or simple text
            parent_id UUID, -- For hierarchy
            author_id UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES vrkb_docs(id) ON DELETE SET NULL,
            FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE SET NULL
        );

        -- Recycle Bin Migration
        -- SQLite Note: Removed IF NOT EXISTS. Ignore errors.
        ALTER TABLE vrkb_docs ADD COLUMN deleted_at TIMESTAMPTZ;
    ").await.map_err(|e| println!("Migration note (vrkb): {}", e));

    // --- RECYCLE BIN CLEANUP TASK ---
    let db_clone_cleanup = db.clone();
    tokio::spawn(async move {
        let threshold_days = 30;
        tracing::info!("Running Recycle Bin Cleanup (Threshold: {} days)...", threshold_days);
        
        use crate::infrastructure::persistence::entities::vrkb::doc;
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use chrono::Utc;

        let time_threshold = Utc::now() - chrono::Duration::days(threshold_days);
        
        // Using ORM for safe dialect handling
        let res = doc::Entity::delete_many()
            .filter(doc::Column::DeletedAt.lt(time_threshold))
            .exec(&db_clone_cleanup)
            .await;
            
        match res {
            Ok(r) => tracing::info!("Cleanup complete. {} items permanently removed.", r.rows_affected),
            Err(e) => tracing::error!("Cleanup failed: {}", e),
        }
    });

    // --- DYNAMIC LAYOUTS ---
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS layout_templates (
            id UUID PRIMARY KEY,
            renderer_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            thumbnail TEXT,
            tags JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
    ").await.expect("Failed to init layout templates");


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

    // --- SYSTEM KB SEEDING ---
    init_system_kbs(&db, &repo).await;

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
    let asset_storage = Arc::new(crate::infrastructure::storage::service::AssetStorageService::new(repo.clone() as Arc<dyn crate::domain::ports::VrkbRepository>, "uploads".to_string()));

    // --- Schema Registry Logic (Phase 11) ---
    let schema_registry = crate::domain::kb::SchemaRegistry::new();
    schema_registry.register("markdown", crate::domain::kb::schemas::markdown::MarkdownSchema);
    schema_registry.register("math_block", crate::domain::kb::schemas::math::MathSchema);
    tracing::info!("KB Schema Registry initialized (types: markdown, math_block)");

    let state = AppState {
        repo,
        auth_service,
        export_service,
        permission_service,
        dictionary,
        dictionary_cache,
        indexer_service,
        graph_service,
        asset_storage,
        schema_registry,
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
        // .merge(draft::router()) // Removed
        .merge(permission::router())
        .merge(user::router())
        .merge(system::router())
        .merge(template::router())
        .merge(crate::interface::api::graph::router())
        .merge(crate::interface::api::vrkb::router())
        .with_state(state);

    let app = Router::new()
        .route("/", get(health_check))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .merge(api_routes)
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB Limit
        .layer(TraceLayer::new_for_http()); // Magic happens here: Automatic logging for every request
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "migrate" {
        tracing::info!("Starting Bulk Migration: Articles -> Blocks...");
        run_bulk_migration(db).await;
        return;
    }

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Aether Core online at {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn run_bulk_migration(db: DatabaseConnection) {
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, TransactionTrait};
    use crate::infrastructure::persistence::entities::{article_detail, blocks, node};
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


async fn health_check() -> &'static str {
    "Aether Systems Operational"
}

// Ensure the "System" KBs exist (Hidden Admin Control Panel)
async fn init_system_kbs(db: &DatabaseConnection, repo: &Arc<PostgresRepository>) {
    use crate::infrastructure::persistence::entities::knowledge_base;
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};

    let system_renderer_id = "admin_system";
    
    // Check if it exists
    let existing = knowledge_base::Entity::find()
        .filter(knowledge_base::Column::RendererId.eq(system_renderer_id))
        .one(db)
        .await
        .unwrap_or_default();

    if existing.is_some() {
        tracing::info!("System KB verified.");
        return;
    }

    tracing::info!("Initializing Admin System Knowledge Base...");
    
    // We need an Author ID. Ideally the Admin.
    // We try to fetch the admin user again or just use a known UUID if we had one.
    // For now, we query the first user with admin permissions (which we just seeded/checked).
    let admin_user = match repo.find_by_username("admin").await { Ok(u) => u, Err(_) => None };
    
    if let Some(admin) = admin_user {
        let kb_id = Uuid::new_v4();
        let active_kb = knowledge_base::ActiveModel {
            id: Set(kb_id),
            author_id: Set(admin.id.0),
            title: Set("Aether System".to_string()),
            description: Set(Some("System Control Panel & Settings".to_string())),
            renderer_id: Set(Some(system_renderer_id.to_string())),
            visibility: Set("Private".to_string()), // Protected
            tags: Set(serde_json::json!(["System"])),
            cover_image: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        if let Err(e) = active_kb.insert(db).await {
             tracing::error!("Failed to create System KB: {}", e);
        } else {
             tracing::info!("Created System KB: {}", kb_id);
        }
    } else {
        tracing::error!("Cannot create System KB: Admin user not found!");
    }
}
