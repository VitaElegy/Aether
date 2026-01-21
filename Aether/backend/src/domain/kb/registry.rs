use super::ast::Block;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Unknown block type: {0}")]
    UnknownType(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// A Trait that every Custom Block Type must implement.
/// This enforces the "Explicit Indexing" and "Strict Validation" protocols.
pub trait BlockSchema: Send + Sync {
    /// Validate the JSON payload against the schema rules.
    /// Returns Ok(()) if valid, or a SchemaError if invalid.
    fn validate(&self, payload: &Value) -> Result<(), SchemaError>;

    /// Extract strictly searchable text from the payload.
    /// This avoids indexing system metadata.
    fn to_searchable_text(&self, payload: &Value) -> String;
}

/// The Global Registry for all Block Schemas.
/// Thread-safe wrapper around the schema map.
#[derive(Clone)]
pub struct SchemaRegistry {
    schemas: Arc<RwLock<HashMap<String, Box<dyn BlockSchema>>>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new Block Schema.
    /// Typically called at application startup.
    pub fn register<S: BlockSchema + 'static>(&self, block_type: &str, schema: S) {
        let mut map = self.schemas.write().expect("Registry lock poisoned");
        map.insert(block_type.to_string(), Box::new(schema));
    }

    /// Validate a single Block against its registered schema.
    pub fn validate_block(&self, block: &Block) -> Result<(), SchemaError> {
        let map = self.schemas.read().expect("Registry lock poisoned");
        
        let schema = map.get(&block.block_type)
            .ok_or_else(|| SchemaError::UnknownType(block.block_type.clone()))?;
            
        schema.validate(&block.payload)
    }

    /// Extract searchable text from a Block.
    /// If the block type is unknown, it returns an empty string (graceful degradation),
    /// or we could decide to error out. For now, strictness suggests error or warning.
    pub fn extract_text(&self, block: &Block) -> Result<String, SchemaError> {
        let map = self.schemas.read().expect("Registry lock poisoned");
        
        // If type is missing, we might skip indexing or error.
        // Let's error to be consistent with Strict mode.
        let schema = map.get(&block.block_type)
            .ok_or_else(|| SchemaError::UnknownType(block.block_type.clone()))?;
            
        Ok(schema.to_searchable_text(&block.payload))
    }
}

// Default implementation (singleton-like pattern usage would be handled by Dependency Injection/State)
// For now, we provide a basic logical container.
