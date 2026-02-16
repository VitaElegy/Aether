#[cfg(test)]
mod tests {
    use crate::infrastructure::persistence::postgres::PostgresRepository;
    use crate::domain::ports::VocabularyRepository;
    use crate::domain::models::{Vocabulary, Node, NodeType, PermissionMode, VocabularyExample};
    use sea_orm::{Database, ConnectionTrait};
    use uuid::Uuid;
    use chrono::Utc;

    async fn setup_schema(db: &sea_orm::DatabaseConnection) {
        // SQLite schema for testing
        db.execute_unprepared("
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                permissions INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE nodes (
                id TEXT PRIMARY KEY,
                parent_id TEXT,
                author_id TEXT NOT NULL,
                knowledge_base_id TEXT,
                type TEXT NOT NULL,
                title TEXT NOT NULL,
                permission_mode TEXT NOT NULL,
                permission_data TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE vocab_details (
                id TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                definition TEXT NOT NULL,
                translation TEXT,
                phonetic TEXT,
                language TEXT NOT NULL,
                status TEXT NOT NULL,
                root_id TEXT,
                query_count INTEGER DEFAULT 0,
                is_important BOOLEAN DEFAULT 0
            );
            CREATE TABLE global_sentences (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                translation TEXT,
                origin_article_id TEXT,
                origin_sentence_uuid TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE vocab_examples (
                id TEXT PRIMARY KEY,
                vocab_id TEXT NOT NULL,
                sentence TEXT,
                translation TEXT,
                note TEXT,
                image_url TEXT,
                article_id TEXT,
                sentence_uuid TEXT,
                created_at TEXT NOT NULL,
                global_sentence_id TEXT
            );
            CREATE TABLE vocab_roots (
                id TEXT PRIMARY KEY,
                root TEXT NOT NULL,
                meaning TEXT
            );
        ").await.expect("Failed to create tables");
    }

    #[tokio::test]
    async fn test_shared_sentences() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db).await;
        let repo = PostgresRepository::new(db.clone());

        let user_id = crate::domain::models::UserId(Uuid::new_v4());

        // 1. Create Vocab A
        let vocab_a_id = Uuid::new_v4();
        let example_text = "This is a shared sentence.";
        
        let vocab_a = Vocabulary {
            node: Node {
                id: vocab_a_id,
                parent_id: None,
                author_id: user_id.0,
                knowledge_base_id: None,
                r#type: NodeType::Vocabulary,
                title: "Word A".to_string(),
                permission_mode: PermissionMode::Private,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            word: "Word A".to_string(),
            definition: "Def A".to_string(),
            translation: None,
            phonetic: None,
            context_sentence: None,
            image_url: None,
            language: "en".to_string(),
            status: "New".to_string(),
            root: None,
            examples: vec![
                VocabularyExample {
                    id: Uuid::new_v4(),
                    sentence: example_text.to_string(),
                    translation: Some("Trans".to_string()),
                    note: None,
                    image_url: None,
                    article_id: None,
                    sentence_uuid: None,
                    created_at: Utc::now(),
                }
            ],
            query_count: 0,
            is_important: false,
        };

        repo.save(vocab_a).await.expect("Failed to save Vocab A");

        // 2. Verify Global Sentence Created
        let results = repo.search_global_sentences("shared").await.expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, example_text);
        let global_id = results[0].0;

        // 3. Create Vocab B with SAME sentence
        let vocab_b_id = Uuid::new_v4();
        let vocab_b = Vocabulary {
            node: Node {
                id: vocab_b_id,
                parent_id: None,
                author_id: user_id.0,
                knowledge_base_id: None,
                r#type: NodeType::Vocabulary,
                title: "Word B".to_string(),
                permission_mode: PermissionMode::Private,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            word: "Word B".to_string(),
            definition: "Def B".to_string(),
            translation: None,
            phonetic: None,
            context_sentence: None,
            image_url: None,
            language: "en".to_string(),
            status: "New".to_string(),
            root: None,
            examples: vec![
                VocabularyExample {
                    id: Uuid::new_v4(),
                    sentence: example_text.to_string(), // Same text
                    translation: Some("Trans".to_string()),
                    note: None,
                    image_url: None,
                    article_id: None,
                    sentence_uuid: None,
                    created_at: Utc::now(),
                }
            ],
            query_count: 0,
            is_important: false,
        };

        repo.save(vocab_b).await.expect("Failed to save Vocab B");

        // 4. Verify Global Sentence Count is still 1
        let results_2 = repo.search_global_sentences("shared").await.expect("Search failed");
        assert_eq!(results_2.len(), 1);
        assert_eq!(results_2[0].0, global_id); // Same ID

        // 5. Verify Retrieval
        let fetched_b = repo.find_by_id(&vocab_b_id).await.expect("Find failed").unwrap();
        assert_eq!(fetched_b.examples[0].sentence, example_text);
        
        // 6. Test Global Update
        // Update Vocab A with modified text for the same example
        let mut fetched_a = repo.find_by_id(&vocab_a_id).await.expect("Find failed").unwrap();
        fetched_a.examples[0].sentence = "Updated shared sentence.".to_string();
        repo.save(fetched_a).await.expect("Failed to update Vocab A");

        // Verify Global Sentence Updated
        let results_3 = repo.search_global_sentences("Updated").await.expect("Search updated failed");
        assert_eq!(results_3.len(), 1);
        assert_eq!(results_3[0].1, "Updated shared sentence.");

        // Verify Vocab B sees the update
        let fetched_b_updated = repo.find_by_id(&vocab_b_id).await.expect("Find failed").unwrap();
        assert_eq!(fetched_b_updated.examples[0].sentence, "Updated shared sentence.");
    }
}
