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
}
