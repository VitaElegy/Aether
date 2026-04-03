use crate::interface::state::AppState;
use axum::Router;

pub mod assets;
pub mod audit;
pub mod checklist;
pub mod docs;
pub mod evidence;
pub mod findings;
pub mod members;
pub mod projects;
pub mod specs;
pub mod stats;
pub mod triage;

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
        .merge(audit::router())
        .merge(checklist::router())
        .merge(evidence::router())
        .merge(triage::router())
        .route(
            "/api/kb/:id/structure",
            axum::routing::get(structure::get_kb_structure),
        )
}
