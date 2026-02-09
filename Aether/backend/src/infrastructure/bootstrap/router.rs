use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::interface::state::AppState;
use crate::interface::api::{
    auth, content, comment, memo, knowledge_base, export, upload, 
    tags, vocabulary, dictionary, permission, user, system, template, group, prkb, graph, vrkb, assets, backup, portability, user_settings,
    openapi::ApiDoc
};

pub fn build_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .merge(auth::router())
        .merge(content::router())
        .merge(comment::router())
        .merge(memo::router())
        .merge(knowledge_base::router())
        .merge(export::router())
        .merge(upload::router())
        .merge(tags::router())
        .merge(vocabulary::router())
        .merge(dictionary::router())
        .merge(permission::router())
        .merge(user::router())
        .merge(group::router())
        .merge(user_settings::router())
        .merge(system::router())
        .merge(prkb::router())
        .merge(template::router())
        .merge(graph::router())
        .merge(vrkb::router())
        .nest("/api/assets", assets::router())
        .nest("/api/backups", backup::router())
        .nest("/api/portability", portability::router())
        .with_state(state);

    Router::new()
        .route("/", axum::routing::get(health_check))
        .nest_service("/uploads", tower_http::services::ServeDir::new("uploads"))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(api_routes)
        .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024)) // 500MB Dynamic Ceiling
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> &'static str {
    "Aether Systems Operational (Bootstrap Mode)"
}
