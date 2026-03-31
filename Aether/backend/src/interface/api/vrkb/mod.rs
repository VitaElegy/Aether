use crate::interface::state::AppState;
use axum::Router;

pub mod assets;
pub mod docs;
pub mod findings;
pub mod members;
pub mod projects;
pub mod specs;
pub mod stats;

pub mod structure;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(projects::router())
        .merge(findings::router())
        .merge(assets::router())
        .merge(members::router())
        .merge(specs::router())
        .merge(docs::router())
        .merge(stats::router())
        .route(
            "/api/kb/:id/structure",
            axum::routing::get(structure::get_kb_structure),
        )
}
