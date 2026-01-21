use axum::Router;
use crate::interface::state::AppState;

pub mod projects;
pub mod findings;
pub mod assets;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(projects::router())
        .merge(findings::router())
        .merge(assets::router())
}
