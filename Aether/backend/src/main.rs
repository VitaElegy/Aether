use axum::{
    routing::{get, post},
    Router, Extension, extract::State, Json,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use sea_orm::{Database, DatabaseConnection};
use dotenvy::dotenv;
use std::env;

// Import our modules
mod domain;
mod infrastructure;
mod application; // We will populate this later
mod interface;   // We will populate this later

use crate::domain::ports::ContentRepository;
use crate::infrastructure::persistence::postgres::PostgresContentRepository;

// 1. Define the Global State
// This is where dependency injection happens.
#[derive(Clone)]
struct AppState {
    // We use Arc<dyn Trait> for dynamic dispatch.
    // This allows us to swap the implementation at runtime or for testing.
    repo: Arc<dyn ContentRepository>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    dotenv().ok();

    // 2. Setup Infrastructure (The "Dirty" Details)
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: DatabaseConnection = Database::connect(db_url).await.expect("Failed to connect to DB");

    // 3. Dependency Injection
    // Wrap the concrete struct in Arc and cast to the Trait object.
    let repo = Arc::new(PostgresContentRepository::new(db));
    let state = AppState { repo };

    // 4. Build Router
    let app = Router::new()
        .route("/", get(health_check))
        // We inject the state into the router
        .with_state(state);

    // 5. Run Server
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Aether Core online at {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Aether Systems Operational"
}

