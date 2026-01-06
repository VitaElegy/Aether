
#[cfg(test)]
mod tests {
    use sea_orm::{Database, DatabaseConnection, ConnectionTrait, ActiveModelTrait, Set, EntityTrait};
    use crate::infrastructure::persistence::postgres::PostgresRepository;
    use crate::domain::models::{Comment, CommentId, CommentableId, CommentableType, UserId};
    use crate::domain::ports::CommentRepository;
    use chrono::Utc;
    use uuid::Uuid;

    async fn setup_schema(db: &DatabaseConnection) {
        // EXACT Schema from main.rs
        db.execute_unprepared("
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                permissions BIGINT NOT NULL,
                display_name TEXT DEFAULT NULL,
                bio TEXT DEFAULT NULL,
                avatar_url TEXT DEFAULT NULL
            );
            CREATE TABLE IF NOT EXISTS comments (
                id UUID PRIMARY KEY,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                user_id UUID NOT NULL,
                parent_id UUID,
                text TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL
            );
        ").await.expect("Failed to create tables");
    }

    #[tokio::test]
    async fn test_add_comment_schema_compatibility() {
        let db = Database::connect("sqlite::memory:?mode=memory&cache=shared").await.unwrap();
        setup_schema(&db).await;
        let repo = PostgresRepository::new(db.clone());

        // 1. Create User
        let user_id = Uuid::new_v4();
        // Manually insert user to satisfy imagined FK (though SQLite default is lazy)
        db.execute_unprepared(&format!("INSERT INTO users (id, username, email, password_hash, permissions) VALUES ('{}', 'test', 'test@test.com', 'hash', 0)", user_id)).await.unwrap();

        // 2. Add Comment
        let comment = Comment {
            id: CommentId(Uuid::new_v4()),
            target: CommentableId {
                target_type: CommentableType::Content,
                target_id: Uuid::new_v4(),
            },
            user_id: UserId(user_id),
            user_name: None,
            user_avatar: None,
            parent_id: None,
            text: "Test Comment".to_string(),
            created_at: Utc::now(),
            replies: vec![],
        };

        let result = repo.add_comment(comment).await;
        
        match result {
            Ok(_) => println!("Comment added successfully"),
            Err(e) => {
                println!("Failed to add comment: {:?}", e);
                panic!("Schema mismatch suspected: {:?}", e);
            }
        }
    }
}
