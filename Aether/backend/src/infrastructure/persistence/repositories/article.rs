use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Article, ContentBody, ContentVersionSnapshot, Node, NodeType, PermissionMode};
use crate::domain::models::UserId;
use crate::domain::ports::{ArticleRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{node, article_detail, content_version};

#[async_trait]
impl ArticleRepository for PostgresRepository {
    async fn save(&self, article: Article, editor_id: UserId) -> Result<Uuid, RepositoryError> {
         // Transactional Save: Node + ArticleDetail
         let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 1. Save Node
        let node_model = node::ActiveModel {
            id: Set(article.node.id),
            parent_id: Set(article.node.parent_id),
            author_id: Set(article.node.author_id),
            r#type: Set("Article".to_string()),
            title: Set(article.node.title.clone()),
            permission_mode: Set(match article.node.permission_mode {
                PermissionMode::Public => "Public".to_string(),
                PermissionMode::Private => "Private".to_string(),
                PermissionMode::Internal => "Internal".to_string(),
            }),
            permission_data: Set(None),
            created_at: Set(article.node.created_at.into()),
            updated_at: Set(article.node.updated_at.into()),
        };
        node::Entity::insert(node_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(node::Column::Id)
                    .update_columns([node::Column::Title, node::Column::UpdatedAt, node::Column::PermissionMode])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Save Detail
        let body_json = serde_json::to_value(&article.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?;
        let status_str = match article.status {
            crate::domain::models::ContentStatus::Draft => "Draft",
            crate::domain::models::ContentStatus::Archived => "Archived",
            crate::domain::models::ContentStatus::Published => "Published",
        }.to_string();

        let detail_model = article_detail::ActiveModel {
            id: Set(article.node.id),
            slug: Set(article.slug),
            status: Set(status_str),
            category: Set(article.category),
            body: Set(body_json.clone()),
            tags: Set(serde_json::to_string(&article.tags).unwrap_or_default()),
        };
        article_detail::Entity::insert(detail_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(article_detail::Column::Id)
                    .update_columns([
                        article_detail::Column::Body, 
                        article_detail::Column::Tags,
                        article_detail::Column::Status,
                        article_detail::Column::Slug
                    ])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 3. Versioning (Naive: Always create for now, optimization in future)
        // TODO: Port hash check logic if needed
        let current_hash = format!("{:x}", Uuid::new_v4().simple());
        
        let version_model = content_version::ActiveModel {
             id: Set(Uuid::new_v4()),
             node_id: Set(article.node.id),
             version: Set(1), // TODO: Fetch max version + 1
             title: Set(article.node.title),
             body: Set(body_json),
             change_reason: Set(None),
             content_hash: Set(current_hash),
             editor_id: Set(editor_id.0),
             created_at: Set(Utc::now().into()),
        };
        content_version::Entity::insert(version_model).exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(article.node.id)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Article>, RepositoryError> {
        let result = node::Entity::find_by_id(*id)
            .find_also_related(article_detail::Entity)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        match result {
            Some((n, Some(d))) => Ok(Some(map_article(n, d))),
            _ => Ok(None) // Node exists but detail missing (integrity error or wrong type)? or Node missing.
        }
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Article>, RepositoryError> {
        // Reverse lookup: Find Detail -> Find Node
        let detail = article_detail::Entity::find()
            .filter(article_detail::Column::Slug.eq(slug))
            .find_also_related(node::Entity)
            .one(&self.db)
            .await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        match detail {
            Some((d, Some(n))) => Ok(Some(map_article(n, d))),
            _ => Ok(None)
        }
    }

    async fn find_by_title(&self, title: &str) -> Result<Option<Article>, RepositoryError> {
         let result = node::Entity::find()
            .filter(node::Column::Type.eq("Article"))
            .filter(node::Column::Title.eq(title))
            .find_also_related(article_detail::Entity)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        match result {
            Some((n, Some(d))) => Ok(Some(map_article(n, d))),
            _ => Ok(None)
        }
    }

    async fn list(&self, _viewer_id: Option<UserId>, _author_id: Option<UserId>, limit: u64, offset: u64) -> Result<Vec<Article>, RepositoryError> {
        // Simple list for now
        let results = node::Entity::find()
            .filter(node::Column::Type.eq("Article"))
            .find_also_related(article_detail::Entity)
            .limit(limit)
            .offset(offset)
            .order_by_desc(node::Column::CreatedAt)
            .all(&self.db)
            .await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut articles = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                articles.push(map_article(n, detail));
            }
        }
        Ok(articles)
    }

    async fn search(&self, _query: &str) -> Result<Vec<Article>, RepositoryError> {
        Ok(vec![]) // Todo
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id).exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn get_version(&self, _id: &Uuid, _version: &str) -> Result<Option<ContentVersionSnapshot>, RepositoryError> {
 Ok(None) }
    async fn get_history(&self, _id: &Uuid) -> Result<Vec<ContentVersionSnapshot>, RepositoryError> { Ok(vec![]) }
}

fn map_article(n: node::Model, d: article_detail::Model) -> Article {
    Article {
        node: Node {
            id: n.id,
            parent_id: n.parent_id,
            author_id: n.author_id,
            r#type: NodeType::Article,
            title: n.title,
            permission_mode: match n.permission_mode.as_str() {
                "Private" => PermissionMode::Private,
                "Internal" => PermissionMode::Internal,
                _ => PermissionMode::Public,
            },
            created_at: n.created_at.with_timezone(&Utc),
            updated_at: n.updated_at.with_timezone(&Utc),
        },
        slug: d.slug,
        status: match d.status.as_str() {
            "Draft" => crate::domain::models::ContentStatus::Draft,
            "Archived" => crate::domain::models::ContentStatus::Archived,
            _ => crate::domain::models::ContentStatus::Published,
        },
        category: d.category,
        body: serde_json::from_value(d.body).unwrap_or(ContentBody::Markdown("".to_string())),
        tags: serde_json::from_str(&d.tags).unwrap_or_default(),
    }
}
