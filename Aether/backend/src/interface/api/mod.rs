use crate::interface::state::AppState;
use axum::Router;

pub mod auth;
pub mod user;
pub mod content;
pub mod vocabulary;
pub mod memo;
pub mod comment;
pub mod tags;
pub mod upload;
pub mod dictionary;
pub mod knowledge_base;
pub mod permission;
pub mod group;
pub mod vrkb;
pub mod export;
pub mod graph;
pub mod template;
pub mod system;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(user::router())
        .merge(content::router()) // Handles articles, nodes
        .merge(vocabulary::router())
        .merge(memo::router())
        .merge(comment::router())
        .merge(tags::router())
        .merge(upload::router())
        .merge(dictionary::router())
        .merge(knowledge_base::router())
        .merge(permission::router())
        .merge(group::router())
        .merge(vrkb::router())
        .merge(export::router())
        .merge(graph::router())
        .merge(template::router())
        .merge(system::router())
}