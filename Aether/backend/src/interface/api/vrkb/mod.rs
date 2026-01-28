use axum::Router;
use crate::interface::state::AppState;

pub mod projects;
pub mod findings;
pub mod assets;
pub mod members;
pub mod specs;
pub mod docs;
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
        .route("/api/kb/:id/structure", axum::routing::get(structure::get_kb_structure))
}
