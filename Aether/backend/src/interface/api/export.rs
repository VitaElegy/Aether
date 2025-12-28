use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap, header},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::models::{ContentId, MemoId, UserId};
use crate::domain::ports::{ExportService, ExportFormat};
use crate::interface::api::auth::AuthenticatedUser;

#[derive(serde::Deserialize, Debug)]
pub struct ExportRequest {
    pub format: Option<String>,
}

pub async fn export_content_handler(
    State(service): State<Arc<dyn ExportService>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Query(params): Query<ExportRequest>,
) -> impl IntoResponse {
    let format = match params.format.as_deref().unwrap_or("markdown").to_lowercase().as_str() {
        "json" => ExportFormat::Json,
        "html" => ExportFormat::Html,
        _ => ExportFormat::Markdown,
    };

    match service.export_content_with_comments(&ContentId(id), format.clone(), Some(UserId(user.id))).await {
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

pub async fn export_memo_handler(
    State(service): State<Arc<dyn ExportService>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Query(params): Query<ExportRequest>,
) -> impl IntoResponse {
    let format = match params.format.as_deref().unwrap_or("markdown").to_lowercase().as_str() {
        "json" => ExportFormat::Json,
        "html" => ExportFormat::Html,
        _ => ExportFormat::Markdown,
    };

    match service.export_memo_with_comments(&MemoId(id), format.clone(), Some(UserId(user.id))).await {
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
