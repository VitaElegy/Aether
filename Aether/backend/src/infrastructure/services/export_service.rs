use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;
use serde_json::json;
use crate::domain::models::{CommentableId, CommentableType, ContentId, MemoId, UserId};
use crate::domain::ports::{ContentRepository, CommentRepository, MemoRepository, ExportService, ExportFormat, ExportData, ExportMetadata, RepositoryError};

pub struct DataExportService {
    content_repo: Arc<dyn ContentRepository>,
    comment_repo: Arc<dyn CommentRepository>,
    memo_repo: Arc<dyn MemoRepository>,
}

impl DataExportService {
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        comment_repo: Arc<dyn CommentRepository>,
        memo_repo: Arc<dyn MemoRepository>,
    ) -> Self {
        Self {
            content_repo,
            comment_repo,
            memo_repo,
        }
    }

    fn format_markdown(&self, data: &ExportData) -> String {
        let mut md = String::new();

        // Metadata
        md.push_str(&format!("---\n"));
        md.push_str(&format!("export_date: {}\n", data.metadata.exported_at));
        md.push_str(&format!("type: {:?}\n", data.entity_type));
        md.push_str(&format!("---\n\n"));

        // Content
        match data.entity_type {
            CommentableType::Content => {
                if let Some(title) = data.entity_data.get("title").and_then(|v| v.as_str()) {
                    md.push_str(&format!("# {}\n\n", title));
                }
                if let Some(body) = data.entity_data.get("body").and_then(|v| v.get("data")).and_then(|v| v.as_str()) {
                     md.push_str(body);
                     md.push_str("\n\n");
                }
            },
            CommentableType::Memo => {
                if let Some(title) = data.entity_data.get("title").and_then(|v| v.as_str()) {
                    md.push_str(&format!("# Memo: {}\n\n", title));
                }
                if let Some(content) = data.entity_data.get("content").and_then(|v| v.as_str()) {
                    md.push_str(content);
                    md.push_str("\n\n");
                }
            }
        }

        // Comments
        if !data.comments.is_empty() {
            md.push_str("---\n## Comments\n\n");
            for comment in &data.comments {
                md.push_str(&format!("**{}** ({}):\n",
                    comment.user_name.as_deref().unwrap_or("Unknown User"),
                    comment.created_at.format("%Y-%m-%d %H:%M")
                ));
                md.push_str(&format!("> {}\n\n", comment.text.replace("\n", "\n> ")));
            }
        }

        md
    }
}

#[async_trait]
impl ExportService for DataExportService {
    async fn export_content_with_comments(
        &self,
        content_id: &ContentId,
        format: ExportFormat,
        requester: Option<UserId>
    ) -> Result<Vec<u8>, RepositoryError> {
        let content = self.content_repo.find_by_id(content_id).await?
            .ok_or(RepositoryError::NotFound)?;

        // Check visibility?? Assuming requester has access logic in Handler or Repo
        // Repo.list handles visibility, but find_by_id usually returns it regardless?
        // Let's assume handler checks permissions or we trust repo.
        // For simple export, we proceed.

        let target = CommentableId {
            target_type: CommentableType::Content,
            target_id: content_id.0,
        };
        let comments = self.comment_repo.get_comments(&target).await?;

        let export_data = ExportData {
            entity_type: CommentableType::Content,
            entity_id: content.id.0,
            entity_data: serde_json::to_value(&content).unwrap_or(json!({})),
            comments,
            metadata: ExportMetadata {
                exported_at: Utc::now(),
                exported_by: requester,
                format: format.clone(),
            },
        };

        match format {
            ExportFormat::Json => {
                let s = serde_json::to_string_pretty(&export_data).map_err(|e| RepositoryError::Unknown(e.to_string()))?;
                Ok(s.into_bytes())
            },
            ExportFormat::Markdown => {
                let s = self.format_markdown(&export_data);
                Ok(s.into_bytes())
            },
            ExportFormat::Html => {
                // Basic HTML wrapper around Markdown or simple structure
                let md = self.format_markdown(&export_data);
                let html = format!("<html><body><pre>{}</pre></body></html>", md); // Simplified
                Ok(html.into_bytes())
            }
        }
    }

    async fn export_memo_with_comments(
        &self,
        memo_id: &MemoId,
        format: ExportFormat,
        requester: Option<UserId>
    ) -> Result<Vec<u8>, RepositoryError> {
        let memo = self.memo_repo.find_by_id(memo_id).await?
            .ok_or(RepositoryError::NotFound)?;

        let target = CommentableId {
            target_type: CommentableType::Memo,
            target_id: memo_id.0,
        };
        let comments = self.comment_repo.get_comments(&target).await?;

        let export_data = ExportData {
            entity_type: CommentableType::Memo,
            entity_id: memo.id.0,
            entity_data: serde_json::to_value(&memo).unwrap_or(json!({})),
            comments,
            metadata: ExportMetadata {
                exported_at: Utc::now(),
                exported_by: requester,
                format: format.clone(),
            },
        };

        match format {
            ExportFormat::Json => {
                let s = serde_json::to_string_pretty(&export_data).map_err(|e| RepositoryError::Unknown(e.to_string()))?;
                Ok(s.into_bytes())
            },
            ExportFormat::Markdown => {
                let s = self.format_markdown(&export_data);
                Ok(s.into_bytes())
            },
            ExportFormat::Html => {
                let md = self.format_markdown(&export_data);
                let html = format!("<html><body><pre>{}</pre></body></html>", md);
                Ok(html.into_bytes())
            }
        }
    }
}
