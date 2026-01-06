
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, ConnectionTrait, ActiveModelTrait, Set};
    use crate::domain::models::{UserId, ContentStatus, Visibility};

    async fn setup_schema(db: &DatabaseConnection) {
        db.execute_unprepared("
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                permissions INTEGER NOT NULL,
                display_name TEXT,
                bio TEXT,
                avatar_url TEXT
            );
            CREATE TABLE contents (
                id TEXT PRIMARY KEY,
                author_id TEXT NOT NULL,
                title TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL,
                visibility TEXT NOT NULL,
                category TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                body TEXT NOT NULL,
                tags TEXT NOT NULL,
                knowledge_base_id TEXT,
                parent_id TEXT,
                content_type TEXT DEFAULT 'Article'
            );
            CREATE TABLE content_versions (
                 id TEXT PRIMARY KEY,
                 content_id TEXT NOT NULL,
                 version INTEGER NOT NULL,
                 title TEXT NOT NULL,
                 body TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 change_reason TEXT,
                 content_hash TEXT DEFAULT '',
                 editor_id TEXT
            );
        ").await.expect("Failed to create tables");
    }

    #[tokio::test]
    async fn test_guest_list_public_articles() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db).await;
        let repo = PostgresRepository::new(db.clone());

        let admin_id = UserId(uuid::Uuid::new_v4());
        let user_id = UserId(uuid::Uuid::new_v4());

        // 1. Create Users
        let admin = user::ActiveModel {
            id: Set(admin_id.0.to_string()),
            username: Set("admin".to_string()),
            email: Set("admin@test.com".to_string()),
            password_hash: Set("hash".to_string()),
            permissions: Set(u64::MAX as i64),
            ..Default::default()
        };
        admin.insert(&db).await.unwrap();

        let normal_user = user::ActiveModel {
            id: Set(user_id.0.to_string()),
            username: Set("user".to_string()),
            email: Set("user@test.com".to_string()),
            password_hash: Set("hash".to_string()),
            permissions: Set(0),
            ..Default::default()
        };
        normal_user.insert(&db).await.unwrap();

        // 2. Create Contents
        let create_content = |title: &str, author: UserId, vis: &str, status: &str| {
            content::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                author_id: Set(author.0.to_string()),
                title: Set(title.to_string()),
                slug: Set(title.to_lowercase().replace(" ", "-")),
                status: Set(status.to_string()),
                visibility: Set(vis.to_string()),
                body: Set("{}".to_string()),
                tags: Set("[]".to_string()),
                created_at: Set(Utc::now().to_rfc3339()),
                updated_at: Set(Utc::now().to_rfc3339()),
                content_type: Set("Article".to_string()),
                ..Default::default()
            }
        };

        // Article 1: Admin, Public, Published (Should be visible)
        create_content("Admin Public", admin_id, "Public", "Published").insert(&db).await.unwrap();
        
        // Article 2: Admin, Internal, Published (Should NOT be visible to Guest)
        create_content("Admin Internal", admin_id, "Internal", "Published").insert(&db).await.unwrap();

        // Article 3: User, Public, Published (Should be visible)
        create_content("User Public", user_id, "Public", "Published").insert(&db).await.unwrap();

        // Article 4: User, Private, Published (Should NOT be visible)
        create_content("User Private", user_id, "Private", "Published").insert(&db).await.unwrap();

        // Article 5: User, Public, Draft (Should NOT be visible)
        create_content("User Draft", user_id, "Public", "Draft").insert(&db).await.unwrap();

        // 3. Act: List as Guest (vied_id = None)
        let results = repo.list(None, None, 100, 0).await.expect("List failed");

        // 4. Assert
        println!("Found {} articles", results.len());
        for c in &results {
            println!(" - {} ({:?}) by {}", c.title, c.visibility, c.author_name.as_ref().unwrap_or(&"Unknown".to_string()));
        }

        assert_eq!(results.len(), 2, "Should find exactly 2 public published articles");
        let titles: Vec<String> = results.into_iter().map(|c| c.title).collect();
        assert!(titles.contains(&"Admin Public".to_string()));
        assert!(titles.contains(&"User Public".to_string()));
    }
}
