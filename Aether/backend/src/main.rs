use axum::{
    routing::{get, post},
    Router, extract::FromRef,
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

use crate::domain::ports::{UserRepository, AuthService, ContentRepository, CommentRepository, MemoRepository, ExportService};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::infrastructure::services::export_service::DataExportService;
use crate::domain::models::User;
use crate::interface::api::auth::{login_handler, register_handler, get_user_handler, update_user_handler};
use crate::interface::api::content::{create_content_handler, list_content_handler, get_content_handler, update_content_handler, delete_content_handler, get_content_diff_handler, search_content_handler, get_history_handler, get_version_handler};
use crate::interface::api::comment::{create_comment_handler, get_comments_handler};
use crate::interface::api::memo::{create_memo_handler, get_memo_handler, list_memos_handler, delete_memo_handler};
use crate::interface::api::export::{export_content_handler, export_memo_handler};
use crate::interface::api::upload::upload_handler;

// Define the Global State
#[derive(Clone)]
struct AppState {
    repo: Arc<PostgresRepository>,
    auth_service: Arc<dyn AuthService>,
    export_service: Arc<dyn ExportService>,
}

impl FromRef<AppState> for Arc<dyn AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn UserRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn UserRepository>
    }
}

impl FromRef<AppState> for Arc<dyn ContentRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn ContentRepository>
    }
}

impl FromRef<AppState> for Arc<dyn CommentRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn CommentRepository>
    }
}

impl FromRef<AppState> for Arc<dyn MemoRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn MemoRepository>
    }
}

impl FromRef<AppState> for Arc<dyn ExportService> {
    fn from_ref(state: &AppState) -> Self {
        state.export_service.clone()
    }
}

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
            tags TEXT[] NOT NULL
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
            target_id UUID NOT NULL,
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
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/register", post(register_handler))
        .route("/api/users/:id", get(get_user_handler).put(update_user_handler))
        .route("/api/content", post(create_content_handler).get(list_content_handler))
        .route("/api/content/:id", get(get_content_handler).put(update_content_handler).delete(delete_content_handler))
        .route("/api/content/:id/diff/:v1/:v2", get(get_content_diff_handler))
        .route("/api/content/:id/history", get(get_history_handler))
        .route("/api/content/:id/version/:version", get(get_version_handler))
        .route("/api/search", get(search_content_handler))
        // Polymorphic Comments
        .route("/api/comments/:type/:id", post(create_comment_handler).get(get_comments_handler))
        // Memos
        .route("/api/memos", post(create_memo_handler).get(list_memos_handler))
        .route("/api/memos/:id", get(get_memo_handler).delete(delete_memo_handler))
        // Export
        .route("/api/export/content/:id", get(export_content_handler))
        .route("/api/export/memo/:id", get(export_memo_handler))
        .route("/api/upload", post(upload_handler))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .with_state(state)
        .layer(TraceLayer::new_for_http()); // Magic happens here: Automatic logging for every request

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Aether Core online at {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Aether Systems Operational"
}
