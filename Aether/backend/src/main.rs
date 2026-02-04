use std::env;
use tokio::net::TcpListener;
use dotenvy::dotenv;

mod domain;
mod infrastructure;
mod interface;

use infrastructure::bootstrap::{database, seeding, services, router};

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    // 1. Logger
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 2. Database
    let db = database::init_pool().await;
    
    // 3. Migrations (CLI or Auto)
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "migrate" {
        tracing::info!("Starting Bulk Migration: Articles -> Blocks...");
        database::run_bulk_migration(db).await;
        return;
    }
    
    // Auto Schema Migrations
    database::run_migrations(&db).await;

    // 4. Services Initialization
    let state = services::init_app_state(db.clone()).await;

    // 5. Seeding
    seeding::seed_all(&db, &state.repo).await;

    // 6. Router & Server
    let app = router::build_router(state);
    
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind port 3000");
    tracing::info!("Aether Core online at {} (Refactored)", addr);
    axum::serve(listener, app).await.unwrap();
}
