use async_trait::async_trait;
use sea_orm::*;
use crate::domain::models::{ContentAggregate, ContentId, ContentStatus, Visibility, User, UserId, Comment, CommentId, ContentVersionSnapshot, Memo, MemoId, CommentableId, CommentableType, KnowledgeBase, KnowledgeBaseId, ContentType};
use crate::domain::ports::{ContentRepository, UserRepository, CommentRepository, MemoRepository, KnowledgeBaseRepository, RepositoryError};
use super::entities::{content, user, content_version, comment, memo, knowledge_base};
use chrono::Utc;
use sea_orm::sea_query::{Expr, Func};

pub struct PostgresRepository {
    db: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 构建内容可见性查询条件
    ///
    /// 规则：
    /// - Public Published: 所有用户（包括 Guest）可见
    /// - Internal Published: 仅登录用户可见
    /// - 自己的内容: 登录用户可以看到自己的所有内容（包括 Draft/Private）
    fn build_visibility_condition(
        &self,
        viewer_id: Option<&UserId>,
        is_own_content: bool,
    ) -> Condition {
        // 基础条件：Public 文章对所有用户无条件可见
        // 使用不区分大小写的匹配以兼容不同的数据源
        // 使用 Func::lower(Expr::col(...)) 以确保 SeaORM 正确处理表别名 (在 join 查询中至关重要)
        let public_published = Condition::all()
            .add(Expr::expr(Func::lower(Expr::col(content::Column::Visibility))).eq("public"))
            .add(Expr::expr(Func::lower(Expr::col(content::Column::Status))).eq("published"));

        // Guest 用户：只能看到 Public Published
        if viewer_id.is_none() {
            return public_published;
        }

        // 登录用户可以额外看到：
        let uid = viewer_id.unwrap();
        let mut condition = Condition::any()
            .add(public_published);

        // 1. Internal Published 文章
        let internal_published = Condition::all()
            .add(Expr::expr(Func::lower(Expr::col(content::Column::Visibility))).eq("internal"))
            .add(Expr::expr(Func::lower(Expr::col(content::Column::Status))).eq("published"));
        condition = condition.add(internal_published);

        // 2. 如果是自己的内容，可以看到所有状态（包括 Draft/Private/Archived）
        if is_own_content {
            condition = condition.add(content::Column::AuthorId.eq(uid.0.to_string()));
        }

        condition
    }
}

#[async_trait]
impl UserRepository for PostgresRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let model = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = model {
            Ok(Some(User {
                id: UserId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                username: m.username,
                email: m.email,
                display_name: m.display_name,
                bio: m.bio,
                avatar_url: m.avatar_url,
                password_hash: m.password_hash,
                permissions: m.permissions as u64,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError> {
        let model = user::Entity::find_by_id(id.0.to_string())
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = model {
            Ok(Some(User {
                id: UserId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                username: m.username,
                email: m.email,
                display_name: m.display_name,
                bio: m.bio,
                avatar_url: m.avatar_url,
                password_hash: m.password_hash,
                permissions: m.permissions as u64,
            }))
        } else {
            Ok(None)
        }
    }

    async fn save(&self, u: User) -> Result<UserId, RepositoryError> {
        let model = user::ActiveModel {
            id: Set(u.id.0.to_string()),
            username: Set(u.username),
            email: Set(u.email),
            display_name: Set(u.display_name),
            bio: Set(u.bio),
            avatar_url: Set(u.avatar_url),
            password_hash: Set(u.password_hash),
            permissions: Set(u.permissions as i64),
        };

        user::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(user::Column::Id)
                    .update_columns([
                        user::Column::Username,
                        user::Column::Email,
                        user::Column::DisplayName,
                        user::Column::Bio,
                        user::Column::AvatarUrl,
                        user::Column::PasswordHash,
                        user::Column::Permissions,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(u.id)
    }
}

#[async_trait]
impl ContentRepository for PostgresRepository {
    async fn save(&self, content: ContentAggregate, editor_id: UserId, should_create_snapshot: bool) -> Result<ContentId, RepositoryError> {
        let serialized_body = serde_json::to_string(&content.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?;
        let serialized_tags = serde_json::to_string(&content.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        // Calculate Hash (SHA256)
        // Combine only versioned fields (Title + Body) to detect content changes.
        // Status/Visibility/Tags are currently not versioned in 'content_versions', so changing them shouldn't trigger a snapshot.
        let hash_input = format!("{}{}",
            content.title,
            serialized_body
        );
        let hash_digest = ring::digest::digest(&ring::digest::SHA256, hash_input.as_bytes());
        let current_hash = hash_digest.as_ref().iter().map(|b| format!("{:02x}", b)).collect::<String>();

        // Start Transaction
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // CHECK: Does content exist? (To decide Insert vs Update manually if needed, or trust on_conflict)
        // SeaORM insert with on_conflict is usually an UPSERT.
        // However, if we insert, we must ensure we don't duplicate via logic error elsewhere.
        // The ID is Primary Key, so duplicate ID insert will trigger OnConflict Update.

        // Convert enum to string using serde serialization for consistency
        let status_str = match content.status {
            ContentStatus::Draft => "Draft",
            ContentStatus::Published => "Published",
            ContentStatus::Archived => "Archived",
        };
        let visibility_str = match content.visibility {
            Visibility::Public => "Public",
            Visibility::Private => "Private",
            Visibility::Internal => "Internal",
        };

        let model = content::ActiveModel {
            id: Set(content.id.0.to_string()),
            author_id: Set(content.author_id.to_string()),
            title: Set(content.title.clone()),
            slug: Set(content.slug),
            status: Set(status_str.to_string()),
            visibility: Set(visibility_str.to_string()),
            category: Set(content.category),
            created_at: Set(content.created_at.to_rfc3339()),
            updated_at: Set(Utc::now().to_rfc3339()),
            body: Set(serialized_body.clone()),
            tags: Set(serialized_tags), // Use Vec<String> directly
            knowledge_base_id: Set(content.knowledge_base_id.map(|id| id.to_string())),
            parent_id: Set(content.parent_id.map(|id| id.to_string())),
            content_type: Set(match content.content_type {
                ContentType::Article => "Article".to_string(),
                ContentType::Directory => "Directory".to_string(),
            }),
        };

        content::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(content::Column::Id)
                    .update_columns([
                        content::Column::Title,
                        content::Column::Status,
                        content::Column::Visibility,
                        content::Column::Category,
                        content::Column::Body,
                        content::Column::UpdatedAt,
                        content::Column::Tags,
                        // Duplicate Tags removed
                        content::Column::KnowledgeBaseId,
                        content::Column::ParentId,
                        content::Column::ContentType,
                    ])
                    .to_owned()
            )
            .exec(&txn) // Use transaction
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // --- Versioning Logic ---

        if should_create_snapshot {
             let last_version = content_version::Entity::find()
                .filter(content_version::Column::ContentId.eq(content.id.0.to_string()))
                .order_by_desc(content_version::Column::Version)
                .one(&txn) // Use transaction
                .await
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

             // Double check if content actually changed to avoid spamming equivalent snapshots if forced?
             // Requirement says: "Backend currently creates snapshots on every save, change to only create when snapshot: true"
             // It implies we should TRUST the snapshot flag primarily.
             // But usually systems also check if hash changed so we don't have duplicates.
             // However, "snapshot: true" acts as a force commit in many systems (like "Publish new version").
             // Yet, let's keep the content hash check as a "safety" or just rely on the flag?
             // "Default behavior: if snapshot not provided, create for Published".
             // If I edit a Published post but don't change anything and hit save... do I want a new version?
             // Let's assume if `should_create_snapshot` is TRUE, we create one, UNLESS it's identical-identical?
             // The prompt says: "change to ONLY create when snapshot: true". This implies the flag is the primary gate.
             // Let's perform a lightweight check: if last version hash == current hash AND reasons are empty, maybe skip?
             // But if user provided a "reason", we surely want a snapshot.

             let perform_insert = if let Some(_lv) = &last_version {
                    // If content unchanged AND no specific version message, effectively skip?
                    // Or should we obey the flag strictly?
                    // Strict obedience is safer for "Force Snapshot" feature.
                    // But prevent exact duplicates if unintended?
                    // Let's implement strict obedience to `should_create_snapshot` BUT
                    // maybe we want to avoid 100% duplicates.
                    // For now, I will trust the flag as the Gatekeeper.
                    // If the user sends snapshot=true, they get a snapshot.
                    true
             } else {
                 true // First version always
             };

             if perform_insert {
                 let next_version = last_version.as_ref().map(|v| v.version + 1).unwrap_or(1);

                 let version_model = content_version::ActiveModel {
                    id: Set(uuid::Uuid::new_v4().to_string()),
                    content_id: Set(content.id.0.to_string()),
                    version: Set(next_version),
                    title: Set(content.title.clone()),
                    body: Set(serialized_body), // Back to String
                    change_reason: Set(content.version_message.clone()),
                    content_hash: Set(current_hash),
                    editor_id: Set(editor_id.0.to_string()), // Back to String
                    created_at: Set(Utc::now().to_rfc3339()),
                };
                content_version::Entity::insert(version_model)
                    .exec(&txn) // Use transaction
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to insert content_version: {:?}", e);
                        RepositoryError::ConnectionError(e.to_string())
                    })?;
             }
        }

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(content.id)
    }

    async fn find_by_id(&self, id: &ContentId) -> Result<Option<ContentAggregate>, RepositoryError> {
        let result = content::Entity::find_by_id(id.0.to_string())
            .find_also_related(user::Entity)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some((m, author)) = result {
            Ok(Some(ContentAggregate {
                id: ContentId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                author_name: author.map(|u| u.display_name.or(Some(u.username)).unwrap_or_default()),
                title: m.title,
                slug: m.slug,
                status: match m.status.to_lowercase().as_str() {
                    "published" => ContentStatus::Published,
                    "archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                visibility: match m.visibility.to_lowercase().as_str() {
                    "private" => Visibility::Private,
                    "internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
                category: m.category,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                body: serde_json::from_str(&m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                version_message: None,
                knowledge_base_id: m.knowledge_base_id.map(|id| uuid::Uuid::parse_str(&id).unwrap_or_default()),
                parent_id: m.parent_id.map(|id| uuid::Uuid::parse_str(&id).unwrap_or_default()),
                content_type: match m.content_type.as_str() {
                    "Directory" => ContentType::Directory,
                    _ => ContentType::Article,
                },
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, _slug: &str) -> Result<Option<ContentAggregate>, RepositoryError> {
        todo!("Implement slug lookup")
    }

    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>, limit: u64, offset: u64) -> Result<Vec<ContentAggregate>, RepositoryError> {
        tracing::debug!(
            "Querying contents: viewer_id={:?}, author_id={:?}, limit={}, offset={}",
            viewer_id.as_ref().map(|id| id.0),
            author_id.as_ref().map(|id| id.0),
            limit,
            offset
        );

        let condition = if let Some(ref aid) = author_id {
            // 查询特定作者的内容
            let aid_str = aid.0.to_string();
            let is_own_content = viewer_id.as_ref().map(|vid| vid.0 == aid.0).unwrap_or(false);

            if is_own_content {
                // 查看自己的资料：看到所有内容（无状态/可见性限制）
                Condition::all().add(content::Column::AuthorId.eq(aid_str))
            } else {
                // 查看别人的资料：必须同时满足作者ID和可见性规则
                // 对于 Guest: author_id = X AND (visibility = 'Public' AND status = 'Published')
                // 对于登录用户: author_id = X AND ((visibility = 'Public' AND status = 'Published') OR (visibility = 'Internal' AND status = 'Published'))
                let visibility_cond = self.build_visibility_condition(viewer_id.as_ref(), false);
                Condition::all()
                    .add(content::Column::AuthorId.eq(aid_str))
                    .add(visibility_cond)
            }
        } else {
            // Feed 模式：使用统一的可见性规则
            // Guest: 只能看到 Public Published（所有作者的）
            // 登录用户: Public Published + Internal Published + 自己的所有内容
            // 这里的 is_own_content = true 允许 visibility logic 添加 "OR AuthorId = Me" 的条件
            self.build_visibility_condition(viewer_id.as_ref(), viewer_id.is_some())
        };

        let results = content::Entity::find()
            .find_also_related(user::Entity)
            .filter(condition)
            .order_by_desc(content::Column::CreatedAt) // Fix pagination order
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Database query error: {:?}", e);
                RepositoryError::ConnectionError(e.to_string())
            })?;

        tracing::debug!("Query returned {} results", results.len());

        let mut aggregates = Vec::new();
        for (m, author) in results {
             aggregates.push(ContentAggregate {
                id: ContentId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                author_name: author.map(|u| u.display_name.or(Some(u.username)).unwrap_or_default()),
                title: m.title,
                slug: m.slug,
                status: match m.status.to_lowercase().as_str() {
                    "published" => ContentStatus::Published,
                    "archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                visibility: match m.visibility.to_lowercase().as_str() {
                    "private" => Visibility::Private,
                    "internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
                category: m.category,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                body: serde_json::from_str(&m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                version_message: None,
                knowledge_base_id: m.knowledge_base_id.map(|id| uuid::Uuid::parse_str(&id).unwrap_or_default()),
                parent_id: m.parent_id.map(|id| uuid::Uuid::parse_str(&id).unwrap_or_default()),
                content_type: match m.content_type.as_str() {
                    "Directory" => ContentType::Directory,
                    _ => ContentType::Article,
                },
            });
        }
        Ok(aggregates)
    }

    async fn search(&self, query: &str) -> Result<Vec<ContentAggregate>, RepositoryError> {
        // 1. Find Content IDs referenced by matching comments
        let comment_matches: Vec<String> = comment::Entity::find()
            .filter(comment::Column::Text.contains(query))
            .filter(comment::Column::TargetType.eq("Content")) // Only match comments on Content
            .select_only()
            .column(comment::Column::TargetId)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Find Author IDs matching name/display name
        let author_matches: Vec<String> = user::Entity::find()
            .filter(
                Condition::any()
                    .add(user::Column::Username.contains(query))
                    .add(user::Column::DisplayName.contains(query))
            )
            .select_only()
            .column(user::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 3. Main Content Search
        // Matches if:
        // - Title or Category or Tags contains query
        // - Body (as text) contains query
        // - Author ID is in author_matches
        // - ID is in comment_matches

        let pattern = format!("%{}%", query);
        // Explicitly casting OrderExpr for SeaORM to inject raw SQL sorting
        // Score: Title(10) + Author(8) + Tags(5) + Body(1)
        // We use standard SQL CASE syntax which works in SQLite and Postgres
        let order_expr = Expr::cust_with_values(
            "CASE WHEN title LIKE $1 THEN 10 ELSE 0 END +
             CASE WHEN tags LIKE $1 THEN 5 ELSE 0 END +
             CASE WHEN body LIKE $1 THEN 1 ELSE 0 END",
            vec![pattern.clone(), pattern.clone(), pattern.clone()]
        );

        // Note: Author match needs a Join which is harder to sort by in this specific ORM query builder flow
        // without breaking the Entity structure.
        // For simplicity and robustness, we prioritize the content fields in the DB sort,
        // and relying on the basic filter to include Author matches.
        // Ideally we would join and add user.username score, but let's stick to the core content attributes first.

        let mut condition = Condition::any()
            .add(content::Column::Title.contains(query))
            .add(content::Column::Category.contains(query))
            .add(content::Column::Tags.contains(query))
            .add(Expr::cust_with_values("body LIKE $1", vec![pattern.clone()]));

        if !author_matches.is_empty() {
            condition = condition.add(content::Column::AuthorId.is_in(author_matches));
        }
        if !comment_matches.is_empty() {
            condition = condition.add(content::Column::Id.is_in(comment_matches));
        }

        let results = content::Entity::find()
            .find_also_related(user::Entity)
            .filter(condition)
            .order_by_desc(order_expr)
            .order_by_desc(content::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // ... (The rest of mapping logic remains the same)
        let mut aggregates = Vec::new();
        for (m, author) in results {
             // ... existing mapping ...
             aggregates.push(ContentAggregate {
                id: ContentId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                author_name: author.as_ref().map(|u| u.display_name.clone().or(Some(u.username.clone())).unwrap_or_default()),
                title: m.title,
                slug: m.slug,
                status: match m.status.to_lowercase().as_str() {
                    "published" => ContentStatus::Published,
                    "archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                visibility: match m.visibility.to_lowercase().as_str() {
                    "private" => Visibility::Private,
                    "internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
                category: m.category,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                body: serde_json::from_str(&m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                version_message: None,
                knowledge_base_id: m.knowledge_base_id.map(|id| uuid::Uuid::parse_str(&id).unwrap_or_default()),
                parent_id: m.parent_id.map(|id| uuid::Uuid::parse_str(&id).unwrap_or_default()),
                content_type: match m.content_type.as_str() {
                    "Directory" => ContentType::Directory,
                    _ => ContentType::Article,
                },
            });
        }
        Ok(aggregates)
    }

    async fn delete(&self, id: &ContentId) -> Result<(), RepositoryError> {
        // ... existing delete implementation ...
        let txn = self.db.begin().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 1. Delete versions
        content_version::Entity::delete_many()
            .filter(content_version::Column::ContentId.eq(id.0.to_string()))
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // 2. Delete content
        content::Entity::delete_by_id(id.0.to_string())
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        txn.commit().await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(())
    }

    async fn get_version(&self, id: &ContentId, version: i32) -> Result<Option<(String, String)>, RepositoryError> {
        let model = content_version::Entity::find()
            .filter(content_version::Column::ContentId.eq(id.0.to_string()))
            .filter(content_version::Column::Version.eq(version))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(model.map(|m| (m.title, m.body)))
    }

    async fn get_history(&self, id: &ContentId) -> Result<Vec<ContentVersionSnapshot>, RepositoryError> {
        let versions = content_version::Entity::find()
            .filter(content_version::Column::ContentId.eq(id.0.to_string()))
            .order_by_desc(content_version::Column::Version)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let snapshots = versions.into_iter().map(|v| ContentVersionSnapshot {
            id: v.id,
            version: format!("0.0.{}", v.version),
            title: v.title,
            created_at: chrono::DateTime::parse_from_rfc3339(&v.created_at)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            reason: v.change_reason,
            editor_id: uuid::Uuid::parse_str(&v.editor_id).unwrap_or_default(),
        }).collect();

        Ok(snapshots)
    }
}

#[async_trait]
impl CommentRepository for PostgresRepository {
    async fn add_comment(&self, c: Comment) -> Result<CommentId, RepositoryError> {
        let (target_type, target_id) = match c.target.target_type {
            CommentableType::Content => ("Content", c.target.target_id.to_string()),
            CommentableType::Memo => ("Memo", c.target.target_id.to_string()),
        };

        let model = comment::ActiveModel {
            id: Set(c.id.0.to_string()),
            target_type: Set(target_type.to_string()),
            target_id: Set(target_id.clone()),
            // Legacy Backfill: content_id is NOT NULL in old schema.
            // We set it to target_id (if valid UUID) or nil UUID if not, to satisfy constraint.
            content_id: Set(Some(target_id)), 
            user_id: Set(c.user_id.0.to_string()),
            parent_id: Set(c.parent_id.map(|id| id.0.to_string())),
            text: Set(c.text),
            created_at: Set(c.created_at.to_rfc3339()),
        };

        comment::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(c.id)
    }

    async fn get_comments(&self, target: &CommentableId) -> Result<Vec<Comment>, RepositoryError> {
        let (t_type, t_id) = match target.target_type {
            CommentableType::Content => ("Content", target.target_id.to_string()),
            CommentableType::Memo => ("Memo", target.target_id.to_string()),
        };

        let results = comment::Entity::find()
            .filter(comment::Column::TargetType.eq(t_type))
            .filter(comment::Column::TargetId.eq(t_id))
            .find_also_related(user::Entity)
            .order_by_asc(comment::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut flat_list = Vec::new();

        for (m, author) in results {
             let c = Comment {
                id: CommentId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                target: CommentableId {
                    target_type: match m.target_type.as_str() {
                        "Content" => CommentableType::Content,
                        "Memo" => CommentableType::Memo,
                        _ => CommentableType::Content, // Default fallback
                    },
                    target_id: uuid::Uuid::parse_str(&m.target_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                },
                user_id: UserId(uuid::Uuid::parse_str(&m.user_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                user_name: author.as_ref().map(|u| u.display_name.clone().or(Some(u.username.clone())).unwrap_or_default()),
                user_avatar: author.as_ref().and_then(|u| u.avatar_url.clone()),
                parent_id: m.parent_id.map(|pid| CommentId(uuid::Uuid::parse_str(&pid).unwrap_or_default())),
                text: m.text,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                replies: Vec::new(),
            };
            flat_list.push(c);
        }

        Ok(flat_list)
    }

    async fn get_comments_batch(&self, targets: &[CommentableId]) -> Result<Vec<Comment>, RepositoryError> {
        // Naive implementation Loop for now, optimizing later if needed
        let mut all_comments = Vec::new();
        for target in targets {
            let mut comments = self.get_comments(target).await?;
            all_comments.append(&mut comments);
        }
        Ok(all_comments)
    }
}

#[async_trait]
impl MemoRepository for PostgresRepository {
    async fn save(&self, memo: Memo) -> Result<MemoId, RepositoryError> {
        let serialized_tags = serde_json::to_string(&memo.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        let model = memo::ActiveModel {
            id: Set(memo.id.0.to_string()),
            author_id: Set(memo.author_id.to_string()),
            title: Set(memo.title),
            content: Set(memo.content),
            tags: Set(serialized_tags),
            created_at: Set(memo.created_at.to_rfc3339()),
            updated_at: Set(memo.updated_at.to_rfc3339()),
            visibility: Set(match memo.visibility {
                Visibility::Public => "Public".to_string(),
                Visibility::Private => "Private".to_string(),
                Visibility::Internal => "Internal".to_string(),
            }),
        };

        memo::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(memo::Column::Id)
                    .update_columns([
                        memo::Column::Title,
                        memo::Column::Content,
                        memo::Column::Tags,
                        memo::Column::UpdatedAt,
                        memo::Column::Visibility,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(memo.id)
    }

    async fn find_by_id(&self, id: &MemoId) -> Result<Option<Memo>, RepositoryError> {
        let result = memo::Entity::find_by_id(id.0.to_string())
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = result {
             Ok(Some(Memo {
                id: MemoId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                title: m.title,
                content: m.content,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                visibility: match m.visibility.to_lowercase().as_str() {
                    "private" => Visibility::Private,
                    "internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
             }))
        } else {
            Ok(None)
        }
    }

    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<Memo>, RepositoryError> {
         let mut condition = Condition::all();

         if let Some(aid) = author_id {
             condition = condition.add(memo::Column::AuthorId.eq(aid.0.to_string()));
         }

         // Visibility Logic (Simplified for Memo compared to Content)
         // If viewer is author, show everything.
         // Else show Public.
         // Internal not fully spec'd for Memo yet, assuming same as Content?
         let vid_str = viewer_id.as_ref().map(|v| v.0.to_string());

         let visibility_cond = if let Some(v_id) = vid_str {
             Condition::any()
                .add(memo::Column::Visibility.eq("Public"))
                .add(memo::Column::AuthorId.eq(v_id)) // Own memos
         } else {
             Condition::all().add(memo::Column::Visibility.eq("Public"))
         };

         condition = condition.add(visibility_cond);

         let results = memo::Entity::find()
            .filter(condition)
            .order_by_desc(memo::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

         let mut memos = Vec::new();
         for m in results {
             memos.push(Memo {
                id: MemoId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                title: m.title,
                content: m.content,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                visibility: match m.visibility.to_lowercase().as_str() {
                    "private" => Visibility::Private,
                    "internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
             });
         }
         Ok(memos)
    }

    async fn delete(&self, id: &MemoId) -> Result<(), RepositoryError> {
        memo::Entity::delete_by_id(id.0.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl KnowledgeBaseRepository for PostgresRepository {
    async fn save(&self, kb: KnowledgeBase) -> Result<KnowledgeBaseId, RepositoryError> {
        let model = knowledge_base::ActiveModel {
            id: Set(kb.id.0.to_string()),
            author_id: Set(kb.author_id.to_string()),
            title: Set(kb.title),
            description: Set(kb.description),
            tags: Set(serde_json::to_string(&kb.tags).unwrap_or_else(|_| "[]".to_string())),
            cover_image: Set(kb.cover_image),
            visibility: Set(match kb.visibility {
                Visibility::Public => "Public".to_string(),
                Visibility::Internal => "Internal".to_string(),
                Visibility::Private => "Private".to_string(),
            }),
            created_at: Set(kb.created_at.to_rfc3339()),
            updated_at: Set(kb.updated_at.to_rfc3339()),
        };

        knowledge_base::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(knowledge_base::Column::Id)
                    .update_columns([
                        knowledge_base::Column::Title,
                        knowledge_base::Column::Description,
                        knowledge_base::Column::Tags,
                        knowledge_base::Column::CoverImage,
                        knowledge_base::Column::Visibility,
                        knowledge_base::Column::UpdatedAt,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(kb.id)
    }

    async fn find_by_id(&self, id: &KnowledgeBaseId) -> Result<Option<KnowledgeBase>, RepositoryError> {
        let model = knowledge_base::Entity::find_by_id(id.0.to_string())
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = model {
            Ok(Some(KnowledgeBase {
                id: KnowledgeBaseId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                title: m.title,
                description: m.description,
                tags: serde_json::from_str(&m.tags).unwrap_or_default(),
                cover_image: m.cover_image,
                visibility: match m.visibility.as_str() {
                    "Public" => Visibility::Public,
                    "Internal" => Visibility::Internal,
                    _ => Visibility::Private,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_title(&self, author_id: &UserId, title: &str) -> Result<Option<KnowledgeBase>, RepositoryError> {
        let model = knowledge_base::Entity::find()
             .filter(
                Condition::all()
                    .add(knowledge_base::Column::AuthorId.eq(author_id.0.to_string()))
                    .add(Expr::expr(Func::lower(Expr::col(knowledge_base::Column::Title))).eq(title.to_lowercase()))
            )
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        if let Some(m) = model {
            Ok(Some(KnowledgeBase {
                id: KnowledgeBaseId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                title: m.title,
                description: m.description,
                tags: serde_json::from_str(&m.tags).unwrap_or_default(),
                cover_image: m.cover_image,
                visibility: match m.visibility.as_str() {
                    "Public" => Visibility::Public,
                    "Internal" => Visibility::Internal,
                    _ => Visibility::Private,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list(&self, viewer_id: Option<UserId>, author_id: Option<UserId>) -> Result<Vec<KnowledgeBase>, RepositoryError> {
        let mut condition = Condition::all();

        if let Some(aid) = author_id {
            condition = condition.add(knowledge_base::Column::AuthorId.eq(aid.0.to_string()));
        }

        // Visibility Logic
        let vid_str = viewer_id.as_ref().map(|v| v.0.to_string());

        let mut vis_cond = Condition::any()
            .add(knowledge_base::Column::Visibility.eq("Public"));

        if let Some(vid) = vid_str {
            vis_cond = vis_cond.add(knowledge_base::Column::Visibility.eq("Internal"));
            vis_cond = vis_cond.add(knowledge_base::Column::AuthorId.eq(vid)); // Always see own
        }

        condition = condition.add(vis_cond);

        let results = knowledge_base::Entity::find()
            .filter(condition)
            .order_by_desc(knowledge_base::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let kbs = results.into_iter().map(|m| {
            Ok(KnowledgeBase {
                id: KnowledgeBaseId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                title: m.title,
                description: m.description,
                tags: serde_json::from_str(&m.tags).unwrap_or_default(),
                cover_image: m.cover_image,
                visibility: match m.visibility.as_str() {
                    "Public" => Visibility::Public,
                    "Internal" => Visibility::Internal,
                    _ => Visibility::Private,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
            })
        }).collect::<Result<Vec<_>, RepositoryError>>()?;

        Ok(kbs)
    }

    async fn delete(&self, id: &KnowledgeBaseId) -> Result<(), RepositoryError> {
        knowledge_base::Entity::delete_by_id(id.0.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl crate::domain::ports::TagRepository for PostgresRepository {
    async fn get_all_tags(&self) -> Result<Vec<String>, RepositoryError> {
        use std::collections::HashSet;
        let mut all_tags = HashSet::new();

        // 1. Fetch tags from Knowledge Bases (JSON String)
        let kb_models = knowledge_base::Entity::find()
            .select_only()
            .column(knowledge_base::Column::Tags)
            .into_model::<knowledge_base::Model>()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        for kb in kb_models {
            if let Ok(tags) = serde_json::from_str::<Vec<String>>(&kb.tags) {
                for tag in tags {
                    all_tags.insert(tag);
                }
            }
        }

        // 2. Fetch tags from Memos (JSON String)
        let memo_models = memo::Entity::find()
            .select_only()
            .column(memo::Column::Tags)
            .into_model::<memo::Model>()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

         for m in memo_models {
            if let Ok(tags) = serde_json::from_str::<Vec<String>>(&m.tags) {
                 for tag in tags {
                    all_tags.insert(tag);
                }
            }
        }

        // 3. Fetch tags from Contents (Postgres Array)
        let content_models = content::Entity::find()
            .select_only()
            .column(content::Column::Tags)
             .into_model::<content::Model>()
             .all(&self.db)
             .await
             .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

         for c in content_models {
             // Parse JSON string similar to KnowledgeBase and Memo
             if let Ok(tags) = serde_json::from_str::<Vec<String>>(&c.tags) {
                 for tag in tags {
                     all_tags.insert(tag);
                 }
             }
         }

        // Convert to sorted vector
        let mut sorted_tags: Vec<String> = all_tags.into_iter().collect();
        sorted_tags.sort();

        Ok(sorted_tags)
    }
}



