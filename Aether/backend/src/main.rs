use axum::{
    routing::{get, post},
    Router, extract::FromRef,
};
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use tokio::net::TcpListener;
use sea_orm::{Database, DatabaseConnection, ConnectionTrait};
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

mod domain;
mod infrastructure;
mod interface;

use crate::domain::ports::{UserRepository, AuthService, ContentRepository};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::domain::models::User;
use crate::interface::api::auth::{login_handler, register_handler};
use crate::interface::api::content::{create_content_handler, list_content_handler, get_content_handler};

// Define the Global State
#[derive(Clone)]
struct AppState {
    repo: Arc<PostgresRepository>,
    auth_service: Arc<dyn AuthService>,
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

#[allow(dead_code)]
fn setup_logging() {
    // 1. File Appender (Non-blocking)
    let file_appender = tracing_appender::rolling::daily("logs", "aether-core.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 2. Layers
    // Stdout: Human readable
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    // File: JSON Structured
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_filter(tracing_subscriber::EnvFilter::new("info")); // Always log info+ to file

    // 3. Registry
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();
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
    ").await.expect("Failed to initialize DB schema");

    let repo = Arc::new(PostgresRepository::new(db.clone()));
    let auth_service = Arc::new(Arg2JwtAuthService::new(
        repo.clone() as Arc<dyn UserRepository>,
        env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string())
    ));

    let admin_name = "admin";
    if repo.find_by_username(admin_name).await.unwrap().is_none() {
        tracing::info!("Seeding admin user...");
        let hash = crate::infrastructure::auth::jwt_service::hash_password("admin");
        let admin = User {
            id: crate::domain::models::UserId(Uuid::new_v4()),
            username: admin_name.to_string(),
            email: "admin@aether.io".to_string(),
            password_hash: hash,
            permissions: u64::MAX,
        };
        UserRepository::save(&*repo, admin).await.expect("Failed to seed admin");
    }

    let state = AppState {
        repo,
        auth_service,
    };

    // --- 4. Build Router with Trace Middleware ---
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/register", post(register_handler))
        .route("/api/content", post(create_content_handler).get(list_content_handler))
        .route("/api/content/:id", get(get_content_handler))
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
