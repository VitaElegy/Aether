use crate::infrastructure::persistence::repositories::system_settings_repository::SystemSettingsRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::extract::{Json as AxumJson, State};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::sync::Arc;

#[derive(Serialize, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UpdateSettingDto {
    pub value: Value,
}

pub async fn get_settings_handler(
    State(repo): State<Arc<SystemSettingsRepository>>,
    _auth: AuthenticatedUser,
) -> impl IntoResponse {
    let max_upload = repo.get_int("max_upload_size_mb", 5).await;
    let settings = serde_json::json!({
        "max_upload_size_mb": max_upload
    });
    (StatusCode::OK, AxumJson(settings)).into_response()
}

pub async fn update_setting_handler(
    State(repo): State<Arc<SystemSettingsRepository>>,
    _auth: AuthenticatedUser,
    AxumJson(payload): AxumJson<serde_json::Map<String, Value>>,
) -> impl IntoResponse {
    for (key, value) in payload {
        if let Err(e) = repo.set(&key, value).await {
            tracing::error!("Failed to update setting {}: {}", key, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(serde_json::json!({ "error": "Failed to update setting" })),
            )
                .into_response();
        }
    }
    (
        StatusCode::OK,
        AxumJson(serde_json::json!({ "status": "updated" })),
    )
        .into_response()
}

pub async fn get_git_log_handler(_auth: AuthenticatedUser) -> impl IntoResponse {
    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%H|%h|%an|%ad|%s",
            "-n",
            "100",
            "--date=iso",
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let commits: Vec<GitCommit> = stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() >= 5 {
                            Some(GitCommit {
                                hash: parts[0].to_string(),
                                short_hash: parts[1].to_string(),
                                author: parts[2].to_string(),
                                date: parts[3].to_string(),
                                message: parts[4..].join("|"),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                (StatusCode::OK, Json(commits)).into_response()
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::error!("Git log failed: {}", stderr);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Git command failed" })),
                )
                    .into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to execute git: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/api/system/git-log", get(get_git_log_handler))
        .route(
            "/api/system/settings",
            get(get_settings_handler).put(update_setting_handler),
        )
}
