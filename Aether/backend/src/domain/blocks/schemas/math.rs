#![allow(dead_code)]
use serde_json::json;

pub fn get_axiom_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "label": { "type": "string", "description": "e.g., 'Axiom of Choice'" },
            "content": { "type": "string", "description": "LaTeX content" },
            "text_mirror": { "type": "string" }
        },
        "required": ["content"]
    })
}

pub fn get_theorem_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "label": { "type": "string", "description": "e.g., 'Theorem 1.4'" },
            "content": { "type": "string" },
            "proof_id": { "type": "string", "format": "uuid" },
            "text_mirror": { "type": "string" }
        },
        "required": ["content"]
    })
}

pub fn get_definition_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "term": { "type": "string", "description": "The term being defined" },
            "content": { "type": "string" },
            "text_mirror": { "type": "string" }
        },
        "required": ["term", "content"]
    })
}

pub fn get_proof_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "theorem_id": { "type": "string", "format": "uuid" },
            "steps": { "type": "string", "description": "Markdown/LaTeX content of the proof" },
            "qcd_symbol": { "type": "string", "default": "■" },
            "text_mirror": { "type": "string" }
        },
        "required": ["steps"]
    })
}
