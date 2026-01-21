use axum::{
    routing::get,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use std::process::Command;
use serde::{Serialize};

#[derive(Serialize, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

pub async fn get_git_log_handler() -> impl IntoResponse {
    let output = Command::new("git")
        .args(&["log", "--pretty=format:%H|%h|%an|%ad|%s", "-n", "100", "--date=iso"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let commits: Vec<GitCommit> = stdout.lines().filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 5 {
                        Some(GitCommit {
                            hash: parts[0].to_string(),
                            short_hash: parts[1].to_string(),
                            author: parts[2].to_string(),
                            date: parts[3].to_string(),
                            message: parts[4..].join("|"), // Rejoin if message contained |
                        })
                    } else {
                        None
                    }
                }).collect();
                (StatusCode::OK, Json(commits)).into_response()
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::error!("Git log failed: {}", stderr);
                 (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Git command failed" }))).into_response()
            }
        },
        Err(e) => {
             tracing::error!("Failed to execute git: {}", e);
             (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub fn router() -> axum::Router<crate::interface::state::AppState> {
    axum::Router::new()
        .route("/api/system/git-log", get(get_git_log_handler))
}
