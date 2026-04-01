use crate::interface::api::{
    assets, auth, backup, comment, content, dictionary, export, graph, group, knowledge_base, math,
    memo, openapi::ApiDoc, permission, portability, prkb, system, tags, template, upload, user,
    user_settings, vocabulary, vrkb,
};
use crate::interface::state::AppState;
use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
        .merge(math::router())
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
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "tauri://localhost".parse::<HeaderValue>().unwrap(),
                    "https://tauri.localhost".parse::<HeaderValue>().unwrap(),
                    "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                    "http://localhost:3000".parse::<HeaderValue>().unwrap(),
                ])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
                .allow_credentials(true),
        )
}

async fn health_check() -> &'static str {
    "Aether Systems Operational (Bootstrap Mode)"
}
