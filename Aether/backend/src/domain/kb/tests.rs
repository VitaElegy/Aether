#[cfg(test)]
mod tests {
    use crate::domain::kb::ast::{Block, Document};
    use crate::domain::kb::registry::{SchemaRegistry, BlockSchema};
    use serde_json::json;
    use serde_json::Value; // Added this import
    use anyhow::Result;

    // --- Mocks ---
    struct MockEnglishBlock;
    impl BlockSchema for MockEnglishBlock {
        fn validate(&self, payload: &Value) -> Result<(), crate::domain::kb::registry::SchemaError> {
            if payload.get("text").is_none() {
                return Err(crate::domain::kb::registry::SchemaError::ValidationFailed("Missing 'text' field".into()));
            }
            Ok(())
        }
        fn to_searchable_text(&self, payload: &Value) -> String {
            payload["text"].as_str().unwrap_or("").to_string()
        }
    }

    // --- Tests ---

    #[test]
    fn test_ast_serialization() {
        let block1 = Block::new("english_sentence", json!({
            "text": "Hello World",
            "is_verified": true
        }));
        
        let doc = Document::new("My Test Doc", vec![block1]);
        
        // Serialize
        let json_str = serde_json::to_string(&doc).unwrap();
        println!("Serialized Doc: {}", json_str);
        
        // Deserialize
        let loaded_doc: Document = serde_json::from_str(&json_str).unwrap();
        assert_eq!(loaded_doc.title, "My Test Doc");
        assert_eq!(loaded_doc.blocks.len(), 1);
        assert_eq!(loaded_doc.blocks[0].block_type, "english_sentence");
    }

    #[test]
    fn test_registry_validation() {
        let registry = SchemaRegistry::new();
        registry.register("english_sentence", MockEnglishBlock);

        // Valid Block
        let valid_block = Block::new("english_sentence", json!({ "text": "Valid" }));
        assert!(registry.validate_block(&valid_block).is_ok());

        // Invalid Block
        let invalid_block = Block::new("english_sentence", json!({ "wrong_field": 123 }));
        assert!(registry.validate_block(&invalid_block).is_err());
        
        // Unknown Type
        let unknown_block = Block::new("alien_tech", json!({}));
        assert!(registry.validate_block(&unknown_block).is_err());
    }

    #[test]
    fn test_search_extraction() {
        let registry = SchemaRegistry::new();
        registry.register("english_sentence", MockEnglishBlock); // Fixed type mismatch: instance instead of type
        
        let block = Block::new("english_sentence", json!({ 
            "text": "Search Me", 
            "hidden": "Ignore Me" 
        }));

        let text = registry.extract_text(&block).unwrap();
        assert_eq!(text, "Search Me");
    }
}
