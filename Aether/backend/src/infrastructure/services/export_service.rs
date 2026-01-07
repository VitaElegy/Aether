use crate::domain::ports::{ExportService, RepositoryError, ExportFormat,
    ArticleRepository, MemoRepository, CommentRepository};
use crate::domain::models::UserId;
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;

pub struct DataExportService {
    article_repo: Arc<dyn ArticleRepository>,
    memo_repo: Arc<dyn MemoRepository>,
    comment_repo: Arc<dyn CommentRepository>,
}

impl DataExportService {
    pub fn new(
        article_repo: Arc<dyn ArticleRepository>,
        memo_repo: Arc<dyn MemoRepository>,
        comment_repo: Arc<dyn CommentRepository>,
    ) -> Self {
        Self { article_repo, memo_repo, comment_repo }
    }

    fn format_markdown(&self, title: &str, body: &str, comments: &[String]) -> String {
        let mut md = format!("# {}\n\n{}\n\n", title, body);
        if !comments.is_empty() {
            md.push_str("## Comments\n\n");
            for c in comments {
                md.push_str(&format!("- {}\n", c));
            }
        }
        md
    }
}

#[async_trait]
impl ExportService for DataExportService {
    async fn export_node_with_comments(
        &self,
        node_id: &Uuid,
        format: ExportFormat,
        _requester: Option<UserId>
    ) -> Result<Vec<u8>, RepositoryError> {
        // 1. Try to find in Article Repo
        if let Ok(Some(article)) = self.article_repo.find_by_id(node_id).await {
            let comments = self.comment_repo.get_comments(node_id).await.unwrap_or_default();
            let comment_texts: Vec<String> = comments.into_iter().map(|c| format!("{}: {}", c.user_name.unwrap_or("Anon".into()), c.text)).collect();
            
            let content_str = match article.body {
                crate::domain::models::ContentBody::Markdown(s) => s,
                _ => "Non-text content".to_string(),
            };

            let md = self.format_markdown(&article.node.title, &content_str, &comment_texts);
            
            // Format handling
            match format {
                ExportFormat::Markdown => Ok(md.into_bytes()),
                ExportFormat::Json => Ok(serde_json::to_vec(&serde_json::json!({
                    "title": article.node.title,
                    "body": content_str,
                    "comments": comment_texts
                })).unwrap()),
                ExportFormat::Html => Ok(format!("<html><h1>{}</h1><p>{}</p></html>", article.node.title, content_str).into_bytes()),
            }
        } else if let Ok(Some(memo)) = self.memo_repo.find_by_id(node_id).await {
             let comments = self.comment_repo.get_comments(node_id).await.unwrap_or_default();
             let comment_texts: Vec<String> = comments.into_iter().map(|c| format!("{}: {}", c.user_name.unwrap_or("Anon".into()), c.text)).collect();
             
             let md = self.format_markdown(&memo.node.title, &memo.content, &comment_texts);
             match format {
                ExportFormat::Markdown => Ok(md.into_bytes()),
                ExportFormat::Json => Ok(serde_json::to_vec(&serde_json::json!({
                    "title": memo.node.title,
                    "body": memo.content,
                    "comments": comment_texts
                })).unwrap()),
                ExportFormat::Html => Ok(format!("<html><h1>{}</h1><p>{}</p></html>", memo.node.title, memo.content).into_bytes()),
             }
        } else {
            Err(RepositoryError::NotFound("Node not found".to_string()))
        }
    }
}
