use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::{Article, ContentBody, ContentVersionSnapshot, Node, NodeType, PermissionMode, ContentItem, ContentDiff};
use crate::domain::models::UserId;
use crate::domain::ports::{ArticleRepository, PermissionRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::{node, article_detail, content_version, user, blocks};
use crate::domain::blocks::parser::parse_markdown_to_blocks;

#[async_trait]
impl ArticleRepository for PostgresRepository {
    async fn save(&self, article: Article, editor_id: UserId, change_reason: Option<String>) -> Result<Uuid, RepositoryError> {
        // 0. Duplicate Title Check
        // Check if another article exists with the same title but different ID
        let duplicate = node::Entity::find()
             .filter(node::Column::Type.eq("Article"))
             .filter(node::Column::Title.eq(&article.node.title))
             .filter(node::Column::Id.ne(article.node.id))
             .one(&self.db)
             .await
             .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if duplicate.is_some() {
            return Err(RepositoryError::DuplicateTitle(format!("Article with title '{}' already exists", article.node.title)));
        }

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
            derived_data: Set(article.derived_data.clone()),
        };
        article_detail::Entity::insert(detail_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(article_detail::Column::Id)
                    .update_columns([
                        article_detail::Column::Body, 
                        article_detail::Column::Tags,
                        article_detail::Column::Status,
                        article_detail::Column::Slug,
                        article_detail::Column::DerivedData
                    ])
                    .to_owned()
            )
            .exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 3. Versioning Logic
        // Calculate Content Hash
        let current_hash = format!("{:x}", md5::compute(body_json.to_string()));

        // Fetch max version
        let max_ver_query = content_version::Entity::find()
            .filter(content_version::Column::NodeId.eq(article.node.id))
            .order_by_desc(content_version::Column::Version)
            .one(&txn)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let (new_version, should_save_version) = match max_ver_query {
            Some(latest) => {
                 if latest.content_hash == current_hash && change_reason.is_none() {
                     // No content change and no forced reason -> Skip versioning
                     (latest.version, false)
                 } else {
                     (latest.version + 1, true)
                 }
            },
            None => (1, true), // First version
        };

        if should_save_version {
            let version_model = content_version::ActiveModel {
                 id: Set(Uuid::new_v4()),
                 node_id: Set(article.node.id),
                 version: Set(new_version),
                 title: Set(article.node.title),
                 body: Set(body_json.clone()),
                 change_reason: Set(change_reason),
                 content_hash: Set(current_hash),
                 editor_id: Set(editor_id.0),
                 created_at: Set(Utc::now().into()),
            };
            content_version::Entity::insert(version_model).exec(&txn).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }

        // [NEW] Dual Write Block Architecture
        // ---------------------------------------------------------
        // Parse Body and Write to Blocks Table
        if let crate::domain::models::ContentBody::Markdown(ref md_text) = article.body {
             let blocks_vec = parse_markdown_to_blocks(article.node.id, md_text);
             
             // 1. Delete existing blocks for this document
             blocks::Entity::delete_many()
                .filter(blocks::Column::DocumentId.eq(article.node.id))
                .exec(&txn)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

             // 2. Insert Parsed Blocks
             if !blocks_vec.is_empty() {
                 let active_blocks: Vec<blocks::ActiveModel> = blocks_vec.into_iter().map(|b| {
                    blocks::ActiveModel {
                        id: Set(b.id),
                        document_id: Set(b.document_id),
                        r#type: Set(b.type_name),
                        ordinal: Set(b.ordinal),
                        revision: Set(b.revision),
                        payload: Set(b.payload),
                        created_at: Set(b.created_at.into()),
                        updated_at: Set(b.updated_at.into()),
                    }
                }).collect();

                 blocks::Entity::insert_many(active_blocks)
                     .exec(&txn)
                     .await
                     .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
             }
        }
        // ---------------------------------------------------------

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 4. ReBAC Permissions (Executed after commit to ensure Node visibility)
        // Ignoring errors here to prevent failing the request if permission update lags
        let _ = self.add_relation(article.node.id, "node", "owner", article.node.author_id, "user").await;
        
        if let PermissionMode::Public = article.node.permission_mode {
             let public_group_id = Uuid::nil();
             let _ = self.add_relation(article.node.id, "node", "viewer", public_group_id, "group").await;
        }

        if let Some(kb_id) = article.node.knowledge_base_id {
            let _ = self.add_relation(article.node.id, "node", "parent", kb_id, "node").await;
        }
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
                 // Sync with list: Return NotFound for incomplete Articles
                 if n.r#type == "Article" {
                     return Ok(None);
                 }

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

    async fn list(&self, _viewer_id: Option<UserId>, author_id: Option<UserId>, knowledge_base_id: Option<Uuid>, tag: Option<String>, category: Option<String>, limit: u64, offset: u64) -> Result<Vec<ContentItem>, RepositoryError> {
        let mut query = node::Entity::find()
            .find_also_related(article_detail::Entity);

        if let Some(ref uid) = author_id {
            query = query.filter(node::Column::AuthorId.eq(uid.0));
        }
        
        // Filter by Category if provided (applied to related article_detail)
        if let Some(cat) = category {
            query = query.filter(article_detail::Column::Category.eq(cat));
        }

        if let Some(kb_id) = knowledge_base_id {
            // In KB view, show everything (Folders, Articles, etc.)
            query = query.filter(node::Column::KnowledgeBaseId.eq(kb_id));
        } else {
            // Global Feed:
            // If viewing a specific author's profile (Self Space), show everything (Drafts included).
            // If viewing global feed (no author), show only "Published".
            query = query.filter(node::Column::Type.eq("Article"));
            
            if author_id.is_none() {
                 query = query.filter(article_detail::Column::Status.eq("Published"));
            }
            
            if let Some(t) = tag {
                // Approximate JSON array search using string matching
                // Note: Only safe for simple alphanumeric tags.
                // Ideally, switch to JSONB containment operator (@>) if SeaORM/Postgres support enables it.
                // Format: ["foo","bar"] -> LIKE '%"foo"%'
                let like_expr = format!("%\"{}\"%", t);
                 query = query.filter(
                    sea_orm::sea_query::Expr::col((article_detail::Entity, article_detail::Column::Tags))
                        .cast_as(sea_orm::sea_query::Alias::new("TEXT"))
                        .like(like_expr)
                );
            }
            
            // SECURITY FIX: Filter by Visibility using the cached 'permission_mode' column
            // 1. Base: Public is always visible
            let mut viz_cond = sea_orm::Condition::any().add(node::Column::PermissionMode.eq("Public"));
            
            // 2. Authenticated: See Internal + Own Private
            if let Some(uid) = _viewer_id {
                viz_cond = viz_cond.add(node::Column::PermissionMode.eq("Internal"));
                viz_cond = viz_cond.add(
                    sea_orm::Condition::all()
                        .add(node::Column::PermissionMode.eq("Private"))
                        .add(node::Column::AuthorId.eq(uid.0))
                );
            }
            query = query.filter(viz_cond);
        }

        let results = query
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
                // Skip "Ghost" Articles (Nodes with type='Article' but no details)
                if n.r#type == "Article" {
                    continue;
                }

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

    async fn search(&self, query: &str) -> Result<Vec<Article>, RepositoryError> {
        let term = format!("%{}%", query);


        // 1. Find matching IDs first (using explicit join for filtering)
        let matching_nodes = node::Entity::find()
            .filter(node::Column::Type.eq("Article"))
            // SECURITY HOTFIX: Search only returns Public articles for now to preventing leaking Private titles
            .filter(node::Column::PermissionMode.eq("Public")) 
            .join(sea_orm::JoinType::LeftJoin, node::Relation::ArticleDetail.def())
            .filter(
                sea_orm::Condition::any()
                    .add(node::Column::Title.like(&term))
                    .add(
                        sea_orm::sea_query::Expr::col((article_detail::Entity, article_detail::Column::Body))
                            .cast_as(sea_orm::sea_query::Alias::new("TEXT"))
                            .like(&term)
                    )
            )
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let ids: Vec<Uuid> = matching_nodes.into_iter().map(|n| n.id).collect();

        if ids.is_empty() {
             return Ok(vec![]);
        }

        // 2. Fetch full entities
        let results = node::Entity::find()
            .filter(node::Column::Id.is_in(ids))
            .find_also_related(article_detail::Entity)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut articles = Vec::new();
        for (n, d) in results {
            if let Some(detail) = d {
                 let user = user::Entity::find_by_id(n.author_id)
                    .one(&self.db)
                    .await
                    .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
                articles.push(map_article(n, detail, user));
            }
        }
        Ok(articles)
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        node::Entity::delete_by_id(*id).exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn delete_recursive(&self, id: &Uuid) -> Result<(), RepositoryError> {
        // Fallback Robust Implementation: BFS Traversal in Application Layer
        // This ensures deletion works even if DB Cascade is not configured or CTEs fail (SQLite specific issues)
        
        let mut to_delete = vec![*id];
        let mut idx = 0;

        // 1. BFS to find all descendants
        while idx < to_delete.len() {
            let current = to_delete[idx];
            let children = node::Entity::find()
                .filter(node::Column::ParentId.eq(current))
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
            for child in children {
                to_delete.push(child.id);
            }
            idx += 1;
        }

        // 2. Delete in Reverse Order (Children first) to satisfy FK constraints (NO ACTION)
        to_delete.reverse();

        for target_id in to_delete {
             node::Entity::delete_by_id(target_id)
                .exec(&self.db)
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }

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
            body: Some(serde_json::from_value(v.body).unwrap_or(ContentBody::Markdown("Error parsing body".to_string()))),
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
            body: None,
        }).collect())
    }

    async fn get_diff(&self, id: &Uuid, v1: &str, v2: &str) -> Result<ContentDiff, RepositoryError> {
        // Fetch both versions using existing method
        let ver1 = self.get_version(id, v1).await?.ok_or(RepositoryError::NotFound(format!("Version {}", v1)))?;
        let ver2 = self.get_version(id, v2).await?.ok_or(RepositoryError::NotFound(format!("Version {}", v2)))?;

        // Extract body text
        let extract_text = |b: Option<ContentBody>| -> String {
             match b {
                 Some(ContentBody::Markdown(t)) => t,
                 Some(ContentBody::CodeSnippet { code, .. }) => code,
                 Some(ContentBody::Custom(v)) => v.to_string(),
                 Some(ContentBody::Video { url, .. }) => url,
                 None => "".to_string(),
             }
        };

        let t1 = extract_text(ver1.body);
        let t2 = extract_text(ver2.body);

        use similar::{ChangeTag, TextDiff};
        let diff = TextDiff::from_lines(&t1, &t2);
        let mut changes = Vec::new();

        for change in diff.iter_all_changes() {
            let tag = match change.tag() {
                ChangeTag::Delete => "Delete",
                ChangeTag::Insert => "Insert",
                ChangeTag::Equal => "Equal",
            };
            changes.push(crate::domain::models::DiffChange {
                tag: tag.to_string(),
                value: change.to_string(),
            });
        }

        Ok(ContentDiff {
            old_version: v1.to_string(),
            new_version: v2.to_string(),
            changes,
        })
    }

    async fn find_drafts_by_article_ids(&self, article_ids: Vec<Uuid>) -> Result<Vec<(Uuid, String, serde_json::Value, chrono::DateTime<chrono::Utc>)>, RepositoryError> {
        use crate::infrastructure::persistence::entities::draft;
        
        let drafts = draft::Entity::find()
            .filter(draft::Column::ArticleId.is_in(article_ids))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(drafts.into_iter().map(|d| (
            d.article_id,
            d.title,
            d.body,
            d.updated_at.with_timezone(&Utc)
        )).collect())
    }

    async fn find_draft_by_id(&self, article_id: &Uuid) -> Result<Option<(String, serde_json::Value)>, RepositoryError> {
        use crate::infrastructure::persistence::entities::draft;
        let draft = draft::Entity::find_by_id(*article_id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        Ok(draft.map(|d| (d.title, d.body)))
    }

    async fn save_draft(&self, article_id: Uuid, title: String, body: serde_json::Value) -> Result<(), RepositoryError> {
        use crate::infrastructure::persistence::entities::draft;
        
        // Upsert Logic manually or simple find+update
        let existing = draft::Entity::find_by_id(article_id).one(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        if let Some(d) = existing {
            let mut active: draft::ActiveModel = d.into();
            active.title = Set(title);
            active.body = Set(body);
            active.updated_at = Set(chrono::Utc::now().into());
            active.update(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        } else {
            let active = draft::ActiveModel {
                article_id: Set(article_id),
                title: Set(title),
                body: Set(body),
                updated_at: Set(chrono::Utc::now().into()),
            };
            active.insert(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn count(&self, author_id: Option<UserId>, knowledge_base_id: Option<Uuid>) -> Result<u64, RepositoryError> {
        let mut query = node::Entity::find()
            .filter(node::Column::Type.eq("Article"));

        if let Some(uid) = author_id {
            query = query.filter(node::Column::AuthorId.eq(uid.0));
        }

        if let Some(kb_id) = knowledge_base_id {
            query = query.filter(node::Column::KnowledgeBaseId.eq(kb_id));
        }

        let count = query.count(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(count)
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
        derived_data: d.derived_data,
    }
}
