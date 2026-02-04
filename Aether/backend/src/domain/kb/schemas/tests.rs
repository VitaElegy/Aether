#[cfg(test)]
mod tests {
    use crate::domain::kb::registry::{BlockSchema, SchemaError};
    use crate::domain::kb::schemas::markdown::MarkdownSchema;
    use crate::domain::kb::schemas::math::MathSchema;
    use serde_json::json;

    #[test]
    fn test_markdown_validation() {
        let schema = MarkdownSchema;
        
        // Valid
        assert!(schema.validate(&json!({ "content": "Hello" })).is_ok());
        
        // Invalid
        assert!(schema.validate(&json!({ "text": "Wrong Key" })).is_err());
        assert!(schema.validate(&json!({})).is_err());
    }

    #[test]
    fn test_math_validation() {
        let schema = MathSchema;
        
        // Valid Theorem
        assert!(schema.validate(&json!({ 
            "math_type": "theorem",
            "latex": "E=mc^2"
        })).is_ok());

        // Valid Axiom with Label
        assert!(schema.validate(&json!({ 
            "math_type": "axiom",
            "latex": "0=0",
            "label": "Trivial"
        })).is_ok());
        
        // Invalid Type
        assert!(schema.validate(&json!({ 
            "math_type": "magic_spell",
            "latex": "foo"
        })).is_err());

        // Missing Latex
        assert!(schema.validate(&json!({ 
            "math_type": "theorem"
        })).is_err());
    }

    #[test]
    fn test_paper_validation() {
        use crate::domain::kb::schemas::paper_v1::PaperSchema;
        let schema = PaperSchema;

        // Valid
        assert!(schema.validate(&json!({
            "title": "Attention Is All You Need",
            "ingest_status": "inbox",
            "year": 2017
        })).is_ok());

        // Valid (Minimal)
        assert!(schema.validate(&json!({
            "title": "Foo",
            "ingest_status": "trash"
        })).is_ok());

        // Invalid (Missing Status)
        assert!(schema.validate(&json!({
            "title": "Bar"
        })).is_err());

        // Invalid (Bad Status)
        assert!(schema.validate(&json!({
            "title": "Baz",
            "ingest_status": "magic"
        })).is_err());

        // Invalid (Bad Year Type)
        assert!(schema.validate(&json!({
            "title": "Qux",
            "ingest_status": "inbox",
            "year": "2024" // String instead of Int
        })).is_err());
    }

    #[test]
    fn test_paper_search_text() {
        use crate::domain::kb::schemas::paper_v1::PaperSchema;
        let schema = PaperSchema;

        let payload = json!({
            "title": "Deep Learning",
            "authors": ["LeCun", "Bengio", "Hinton"],
            "venue": "Nature",
            "abstract": "Deep learning allows computational models..."
        });

        let text = schema.to_searchable_text(&payload);
        assert!(text.contains("Deep Learning"));
        assert!(text.contains("LeCun Bengio Hinton"));
        assert!(text.contains("Nature"));
    }
}
