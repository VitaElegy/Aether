use super::models::BlockDefinition;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use jsonschema::JSONSchema;
use serde_json::Value;

#[derive(Clone)]
#[allow(dead_code)]
pub struct SchemaRegistry {
    definitions: Arc<RwLock<HashMap<String, BlockDefinition>>>,
}

impl SchemaRegistry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn register(&self, type_name: &str, schema_json: Value, complexity_score: u8) -> Result<(), String> {
        // Enforce Quotas (Simplistic implementation)
        if complexity_score > 100 {
            return Err("Schema complexity too high".to_string());
        }

        // Complie Schema
        let compiled = JSONSchema::compile(&schema_json)
            .map_err(|e| format!("Invalid JSON Schema: {:?}", e))?;

        let def = BlockDefinition {
            type_name: type_name.to_string(),
            validation_schema: Arc::new(compiled),
            complexity_score,
        };

        self.definitions.write().unwrap().insert(type_name.to_string(), def);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate(&self, type_name: &str, payload: &Value) -> Result<(), String> {
        let registry = self.definitions.read().unwrap();
        
        if let Some(def) = registry.get(type_name) {
            let result = def.validation_schema.validate(payload);
            if let Err(errors) = result {
                let error_msg = errors
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                return Err(format!("Validation failed: {}", error_msg));
            }
            Ok(())
        } else {
            // If unknown type, do we fail? Or allow "untyped"? 
            // Strict mode: Fail.
            Err(format!("Unknown block type: {}", type_name))
        }
    }
}
