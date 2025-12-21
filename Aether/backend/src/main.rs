use axum::{
    routing::{get, post},
    Router, extract::FromRef,
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

use crate::domain::ports::{UserRepository, AuthService};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::domain::models::User;
use crate::interface::api::auth::login_handler;

// Define the Global State
#[derive(Clone)]
struct AppState {
    #[allow(dead_code)] // Keep it for future use
    repo: Arc<PostgresRepository>,
    auth_service: Arc<dyn AuthService>,
}

// Support extracting specific services from AppState
impl FromRef<AppState> for Arc<dyn AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://aether.db?mode=rwc".to_string());
    let db: DatabaseConnection = Database::connect(db_url).await.expect("Failed to connect to DB");

    // --- 1. Auto-Migration (Hack for Demo) ---
    let _ = db.execute_unprepared("
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            permissions INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS contents (
            id TEXT PRIMARY KEY,
            author_id TEXT NOT NULL,
            title TEXT NOT NULL,
            slug TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            body TEXT NOT NULL,
            tags TEXT NOT NULL
        );
    ").await.expect("Failed to initialize DB schema");

    // --- 2. Setup Dependencies ---
    let repo = Arc::new(PostgresRepository::new(db.clone()));
    let auth_service = Arc::new(Arg2JwtAuthService::new(
        repo.clone() as Arc<dyn UserRepository>, // Cast to trait
        env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string())
    ));

    // --- 3. Seed Admin User ---
    let admin_name = "admin";
    if repo.find_by_username(admin_name).await.unwrap().is_none() {
        println!("Seeding admin user...");
        let hash = crate::infrastructure::auth::jwt_service::hash_password("admin");
        let admin = User {
            id: crate::domain::models::UserId(Uuid::new_v4()),
            username: admin_name.to_string(),
            email: "admin@aether.io".to_string(),
            password_hash: hash,
            permissions: u64::MAX, // God mode
        };
        // Explicit Trait Call to resolve ambiguity
        UserRepository::save(&*repo, admin).await.expect("Failed to seed admin");
    }

    let state = AppState {
        repo,
        auth_service,
    };

    // --- 4. Build Router ---
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/auth/login", post(login_handler))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Aether Core online at {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Aether Systems Operational"
}
