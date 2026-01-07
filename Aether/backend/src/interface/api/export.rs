use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap, header},
    response::IntoResponse,
};
use uuid::Uuid;
use crate::domain::models::UserId;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use crate::domain::ports::ExportFormat; // Need Trait

#[derive(serde::Deserialize, Debug)]
pub struct ExportRequest {
    pub format: Option<String>,
}

pub async fn export_node_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Query(params): Query<ExportRequest>,
) -> impl IntoResponse {
    let format = match params.format.as_deref().unwrap_or("markdown").to_lowercase().as_str() {
        "json" => ExportFormat::Json,
        "html" => ExportFormat::Html,
        _ => ExportFormat::Markdown,
    };

    match state.export_service.export_node_with_comments(&id, format.clone(), Some(UserId(user.id))).await {
        Ok(bytes) => {
             let mut headers = HeaderMap::new();
             let content_type = match format {
                 ExportFormat::Json => "application/json",
                 ExportFormat::Html => "text/html",
                 ExportFormat::Markdown => "text/markdown",
             };
             headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
             headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"export.{}\"",
                match format {
                    ExportFormat::Json => "json",
                    ExportFormat::Html => "html",
                    ExportFormat::Markdown => "md",
                }
             ).parse().unwrap());

             (StatusCode::OK, headers, bytes).into_response()
        },
        Err(e) => {
            tracing::error!("Export failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Export failed").into_response()
        }
    }
}

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::get;
    axum::Router::new()
        .route("/api/export/:id", get(export_node_handler))
}
