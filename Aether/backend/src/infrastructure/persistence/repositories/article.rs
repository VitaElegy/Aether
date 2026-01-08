use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Article, ContentBody, ContentVersionSnapshot, Node, NodeType, PermissionMode, ContentItem};
use crate::domain::models::UserId;
use crate::domain::ports::{ArticleRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{node, article_detail, content_version, user};

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
            knowledge_base_id: Set(article.node.knowledge_base_id),
            r#type: Set("Article".to_owned()),
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
                    .update_columns([
                        node::Column::Title, 
                        node::Column::UpdatedAt, 
                        node::Column::PermissionMode,
                        node::Column::ParentId,
                        node::Column::KnowledgeBaseId
                    ])
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

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ContentItem>, RepositoryError> {
        // Try to fetch as generic Node first
        let node_model = node::Entity::find_by_id(*id)
            .find_also_related(article_detail::Entity)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        match node_model {
             Some((n, Some(d))) => {
                let user = user::Entity::find_by_id(n.author_id)
                    .one(&self.db)
                    .await
                    .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
                Ok(Some(ContentItem::Article(map_article(n, d, user))))
            },
            Some((n, None)) => {
                 // It's a node without details (Folder, etc.)
                 Ok(Some(ContentItem::Node(Node {
                    id: n.id,
                    parent_id: n.parent_id,
                    author_id: n.author_id,
                    knowledge_base_id: n.knowledge_base_id,
                    r#type: match n.r#type.as_str() {
                        "Article" => NodeType::Article,
                        "Folder" => NodeType::Folder,
                        "Vocabulary" => NodeType::Vocabulary,
                        "Memo" => NodeType::Memo,
                        _ => NodeType::Article, 
                    },
                    title: n.title,
                    permission_mode: match n.permission_mode.as_str() {
                        "Private" => PermissionMode::Private,
                        "Internal" => PermissionMode::Internal,
                        _ => PermissionMode::Public,
                    },
                    created_at: n.created_at.into(),
                    updated_at: n.updated_at.into(),
                })))
            },
            None => Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Article>, RepositoryError> {
        let detail = article_detail::Entity::find()
            .filter(article_detail::Column::Slug.eq(slug))
            .find_also_related(node::Entity)
            .one(&self.db)
            .await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        match detail {
            Some((d, Some(n))) => {
                let user = user::Entity::find_by_id(n.author_id)
                    .one(&self.db)
                    .await
                    .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
                Ok(Some(map_article(n, d, user)))
            },
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
             Some((n, Some(d))) => {
                let user = user::Entity::find_by_id(n.author_id)
                    .one(&self.db)
                    .await
                    .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
                Ok(Some(map_article(n, d, user)))
            },
             _ => Ok(None)
        }
    }

    async fn list(&self, _viewer_id: Option<UserId>, _author_id: Option<UserId>, knowledge_base_id: Option<Uuid>, limit: u64, offset: u64) -> Result<Vec<ContentItem>, RepositoryError> {
        let mut query = node::Entity::find();

        if let Some(kb_id) = knowledge_base_id {
            // In KB view, show everything (Folders, Articles, etc.)
            query = query.filter(node::Column::KnowledgeBaseId.eq(kb_id));
        } else {
            // Global Feed: Show only Articles
            query = query.filter(node::Column::Type.eq("Article"));
        }

        let results = query
            .find_also_related(article_detail::Entity)
            .limit(limit)
            .offset(offset)
            .order_by_desc(node::Column::CreatedAt)
            .all(&self.db)
            .await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let author_ids: Vec<Uuid> = results.iter().map(|(n, _)| n.author_id).collect();
        // Batch fetch users. Note: author_ids might have duplicates, sea-orm filter is fine.
        let users = user::Entity::find()
            .filter(user::Column::Id.is_in(author_ids))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        let user_map: std::collections::HashMap<Uuid, user::Model> = users.into_iter().map(|u| (u.id, u)).collect();

        let mut content_items = Vec::new();
        for (n, d) in results {
            let user = user_map.get(&n.author_id).cloned();
            if let Some(detail) = d {
                content_items.push(ContentItem::Article(map_article(n, detail, user)));
            } else {
                // Generic Node (Folder or other)
                content_items.push(ContentItem::Node(Node {
                    id: n.id,
                    parent_id: n.parent_id,
                    author_id: n.author_id,
                    knowledge_base_id: n.knowledge_base_id,
                    r#type: match n.r#type.as_str() {
                        "Article" => NodeType::Article,
                        "Folder" => NodeType::Folder,
                        "Vocabulary" => NodeType::Vocabulary,
                        "Memo" => NodeType::Memo,
                        _ => NodeType::Article, 
                    },
                    title: n.title,
                    permission_mode: match n.permission_mode.as_str() {
                        "Private" => PermissionMode::Private,
                        "Internal" => PermissionMode::Internal,
                        _ => PermissionMode::Public,
                    },
                    created_at: n.created_at.into(),
                    updated_at: n.updated_at.into(),
                }));
            }
        }
        Ok(content_items)
    }

    async fn search(&self, _query: &str) -> Result<Vec<Article>, RepositoryError> {
        Ok(vec![]) // Todo
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id).exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn get_version(&self, id: &Uuid, version: &str) -> Result<Option<ContentVersionSnapshot>, RepositoryError> {
        let ver_int = version.parse::<i32>().map_err(|_| RepositoryError::ValidationError("Invalid version number".to_string()))?;
        
        let result = content_version::Entity::find()
            .filter(content_version::Column::NodeId.eq(*id))
            .filter(content_version::Column::Version.eq(ver_int))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(result.map(|v| ContentVersionSnapshot {
            id: v.id.to_string(),
            version: v.version.to_string(),
            title: v.title,
            created_at: v.created_at.into(),
            reason: v.change_reason,
            editor_id: v.editor_id,
        }))
    }
    async fn get_history(&self, id: &Uuid) -> Result<Vec<ContentVersionSnapshot>, RepositoryError> {
        let versions = content_version::Entity::find()
             .filter(content_version::Column::NodeId.eq(*id))
             .order_by_desc(content_version::Column::Version)
             .all(&self.db)
             .await
             .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(versions.into_iter().map(|v| ContentVersionSnapshot {
            id: v.id.to_string(),
            version: v.version.to_string(),
            title: v.title,
            created_at: v.created_at.with_timezone(&Utc),
            reason: v.change_reason,
            editor_id: v.editor_id,
        }).collect())
    }
}

fn map_article(n: node::Model, d: article_detail::Model, match_user: Option<crate::infrastructure::persistence::entities::user::Model>) -> Article {
    Article {
        node: Node {
            id: n.id,
            parent_id: n.parent_id,
            author_id: n.author_id,
            knowledge_base_id: n.knowledge_base_id,
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
        author_name: match_user.as_ref().map(|u| u.display_name.clone().unwrap_or(u.username.clone())),
        author_avatar: match_user.as_ref().and_then(|u| u.avatar_url.clone()),
    }
}
