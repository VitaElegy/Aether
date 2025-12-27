use async_trait::async_trait;
use sea_orm::*;
use crate::domain::models::{ContentAggregate, ContentId, ContentStatus, Visibility, User, UserId, Comment, CommentId, ContentVersionSnapshot};
use crate::domain::ports::{ContentRepository, UserRepository, CommentRepository, RepositoryError};
use super::entities::{content, user, content_version, comment};
use chrono::Utc;
use sea_orm::sea_query::Expr;

pub struct PostgresRepository {
    db: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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
    async fn save(&self, content: ContentAggregate, editor_id: UserId) -> Result<ContentId, RepositoryError> {
        let serialized_body = serde_json::to_string(&content.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?;
        let serialized_tags = serde_json::to_string(&content.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        // Calculate Hash (SHA256)
        // Combine all semantic fields to detect any change
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

        let model = content::ActiveModel {
            id: Set(content.id.0.to_string()),
            author_id: Set(content.author_id.to_string()),
            title: Set(content.title.clone()),
            slug: Set(content.slug),
            status: Set(format!("{:?}", content.status)),
            visibility: Set(format!("{:?}", content.visibility)),
            category: Set(content.category),
            created_at: Set(content.created_at.to_rfc3339()),
            updated_at: Set(Utc::now().to_rfc3339()),
            body: Set(serialized_body.clone()),
            tags: Set(serialized_tags),
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
                    ])
                    .to_owned()
            )
            .exec(&txn) // Use transaction
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // --- Versioning Logic ---

        let last_version = content_version::Entity::find()
            .filter(content_version::Column::ContentId.eq(content.id.0.to_string()))
            .order_by_desc(content_version::Column::Version)
            .one(&txn) // Use transaction
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let should_snapshot = if let Some(lv) = &last_version {
            // 1. Explicit reason -> Force snapshot
            if content.version_message.is_some() {
                true
            } else {
                // 2. Hash Changed -> Snapshot
                // We handle legacy records (empty hash) by assuming change if hash is empty
                if lv.content_hash.is_empty() {
                    true
                } else {
                    lv.content_hash != current_hash
                }
            }
        } else {
            true // First version
        };

        if should_snapshot {
            // FIX: If last_version is None, we start at 1. If it exists, we increment.
            // CAUTION: 'last_version' is Option<Model>.
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
                status: match m.status.as_str() {
                    "Published" => ContentStatus::Published,
                    "Archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                visibility: match m.visibility.as_str() {
                    "Private" => Visibility::Private,
                    "Internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
                category: m.category,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                body: serde_json::from_str(&m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                version_message: None, // Not persisted in main table
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, _slug: &str) -> Result<Option<ContentAggregate>, RepositoryError> {
        todo!("Implement slug lookup")
    }

    async fn list(&self, viewer_id: Option<UserId>, limit: u64, offset: u64) -> Result<Vec<ContentAggregate>, RepositoryError> {
         let mut condition = Condition::any()
            .add(
                Condition::all()
                    .add(content::Column::Visibility.eq("Public"))
                    .add(content::Column::Status.eq("Published"))
            );

         if let Some(uid) = viewer_id {
             // Logged in: Can see Internal Published
             condition = condition.add(
                Condition::all()
                    .add(content::Column::Visibility.eq("Internal"))
                    .add(content::Column::Status.eq("Published"))
             );
             // Logged in: Can see ALL my own content (Drafts, Private, etc.)
             condition = condition.add(content::Column::AuthorId.eq(uid.0.to_string()));
         }

         let results = content::Entity::find()
            .find_also_related(user::Entity)
            .filter(condition)
            .order_by_desc(content::Column::CreatedAt) // Fix pagination order
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut aggregates = Vec::new();
        for (m, author) in results {
             aggregates.push(ContentAggregate {
                id: ContentId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                author_id: uuid::Uuid::parse_str(&m.author_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                author_name: author.map(|u| u.display_name.or(Some(u.username)).unwrap_or_default()),
                title: m.title,
                slug: m.slug,
                status: match m.status.as_str() {
                    "Published" => ContentStatus::Published,
                    "Archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                visibility: match m.visibility.as_str() {
                    "Private" => Visibility::Private,
                    "Internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
                category: m.category,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                body: serde_json::from_str(&m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                version_message: None,
            });
        }
        Ok(aggregates)
    }

    async fn search(&self, query: &str) -> Result<Vec<ContentAggregate>, RepositoryError> {
        // 1. Find Content IDs referenced by matching comments
        let comment_matches: Vec<String> = comment::Entity::find()
            .filter(comment::Column::Text.contains(query))
            .select_only()
            .column(comment::Column::ContentId)
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
                status: match m.status.as_str() {
                    "Published" => ContentStatus::Published,
                    "Archived" => ContentStatus::Archived,
                    _ => ContentStatus::Draft,
                },
                visibility: match m.visibility.as_str() {
                    "Private" => Visibility::Private,
                    "Internal" => Visibility::Internal,
                    _ => Visibility::Public,
                },
                category: m.category,
                created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at).map_err(|e| RepositoryError::Unknown(e.to_string()))?.with_timezone(&Utc),
                body: serde_json::from_str(&m.body).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                tags: serde_json::from_str(&m.tags).map_err(|e| RepositoryError::Unknown(e.to_string()))?,
                version_message: None,
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
            version: v.version,
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
        let model = comment::ActiveModel {
            id: Set(c.id.0.to_string()),
            content_id: Set(c.content_id.0.to_string()),
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

    async fn get_comments(&self, content_id: &ContentId) -> Result<Vec<Comment>, RepositoryError> {
        let results = comment::Entity::find()
            .filter(comment::Column::ContentId.eq(content_id.0.to_string()))
            .find_also_related(user::Entity)
            .order_by_asc(comment::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut flat_list = Vec::new();

        for (m, author) in results {
             let c = Comment {
                id: CommentId(uuid::Uuid::parse_str(&m.id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
                content_id: ContentId(uuid::Uuid::parse_str(&m.content_id).map_err(|e| RepositoryError::Unknown(e.to_string()))?),
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
}
