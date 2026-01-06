
#[cfg(test)]
mod tests {
    use sea_orm::{Database, DatabaseConnection, ConnectionTrait, ActiveModelTrait, Set, EntityTrait};
    use crate::infrastructure::persistence::postgres::PostgresRepository;
    use crate::domain::models::{ContentAggregate, ContentId, ContentStatus, Visibility, ContentType, UserId};
    use crate::domain::ports::ContentRepository;
    use chrono::Utc;
    use uuid::Uuid;
    use crate::infrastructure::persistence::entities::content_version;
    use sea_orm::QueryOrder;

    async fn setup_schema(db: &DatabaseConnection) {
        db.execute_unprepared("
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                permissions BIGINT NOT NULL,
                display_name TEXT, bio TEXT, avatar_url TEXT
            );
            CREATE TABLE IF NOT EXISTS contents (
                id UUID PRIMARY KEY,
                author_id UUID NOT NULL,
                title TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL,
                visibility TEXT NOT NULL DEFAULT 'Public',
                category TEXT,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                body JSONB NOT NULL,
                tags TEXT NOT NULL,
                knowledge_base_id UUID,
                parent_id UUID,
                content_type TEXT DEFAULT 'Article'
            );
            CREATE TABLE IF NOT EXISTS content_versions (
                id UUID PRIMARY KEY,
                content_id UUID NOT NULL,
                version INT NOT NULL,
                title TEXT NOT NULL,
                body JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                change_reason TEXT,
                content_hash TEXT NOT NULL,
                editor_id UUID NOT NULL
            );
        ").await.expect("Failed to create tables");
    }

    #[tokio::test]
    async fn test_redundant_snapshot_prevention() {
        let db = Database::connect("sqlite::memory:?mode=memory&cache=shared").await.unwrap();
        setup_schema(&db).await;
        let repo = PostgresRepository::new(db.clone());

        let user_id = UserId(Uuid::new_v4());
        db.execute_unprepared(&format!("INSERT INTO users (id, username, email, password_hash, permissions) VALUES ('{}', 'test', 'test@test.com', 'hash', 0)", user_id.0)).await.unwrap();

        let content_id = ContentId(Uuid::new_v4());
        let content = ContentAggregate {
            id: content_id.clone(),
            author_id: user_id.0,
            author_name: "test".to_string(),
            title: "Original Title".to_string(),
            slug: "slug".to_string(),
            status: ContentStatus::Published,
            visibility: Visibility::Public,
            category: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            body: serde_json::from_str(r#"{"type":"Markdown","data":"Original Body"}"#).unwrap(),
            tags: vec![],
            version_message: None,
            knowledge_base_id: None,
            parent_id: None,
            content_type: ContentType::Article,
        };

        // 1. Initial Save -> v1
        repo.save(content.clone(), user_id, true).await.unwrap();
        
        let versions = content_version::Entity::find().all(&db).await.unwrap();
        assert_eq!(versions.len(), 1, "Should have 1 version initially");

        // 2. Redundant Save (Identical) -> Should NOT create v2
        let mut content_v2 = content.clone();
        content_v2.version_message = None; // Explicitly no message
        repo.save(content_v2, user_id, true).await.unwrap();

        let versions = content_version::Entity::find().all(&db).await.unwrap();
        assert_eq!(versions.len(), 1, "Should still have 1 version after redundant save");

        // 3. Changed Save -> Should create v2
        let mut content_v3 = content.clone();
        content_v3.title = "New Title".to_string();
        repo.save(content_v3, user_id, true).await.unwrap();

        let versions = content_version::Entity::find().all(&db).await.unwrap();
        assert_eq!(versions.len(), 2, "Should have 2 versions after actual change");

        // 4. Redundant Save BUT with specific message -> "Force Snapshot" -> Should create v3
        let mut content_v4 = content.clone(); // Original content (same hash as v1!) 
        // Wait, v2 has different hash. v1 has 'A'. v2 has 'B'. v3?
        // Note: content_v4 has 'A'. Last version is 'B'.
        // So this is NOT redundant. It's a revert to A. It SHOULD create v3.
        
        // Let's make one identical to *current stored state* (v2 'B')
        let mut content_v2_dup = content.clone();
        content_v2_dup.title = "New Title".to_string(); // 'B' state
        content_v2_dup.version_message = Some("Forced Snapshot".to_string());
        
        repo.save(content_v2_dup, user_id, true).await.unwrap();
        
        let versions = content_version::Entity::find().all(&db).await.unwrap();
        assert_eq!(versions.len(), 3, "Should have 3 versions because message was provided even if content same");
    }
}
