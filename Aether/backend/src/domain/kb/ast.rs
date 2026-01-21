use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub type BlockId = Uuid;
// pub type AssetId = Uuid;

/// A Block is the fundamental unit of content in Aether.
/// It corresponds to the "Block-First" architecture decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    /// The type of the block (e.g., "english_sentence", "latex_theorem")
    /// This key is used to look up the Schema in the Registry.
    #[serde(rename = "type")]
    pub block_type: String,
    /// The actual content payload. Its structure is opaque to the Core,
    /// but must validate against the Schema registered for `block_type`.
    pub payload: Value,
    /// Child blocks (for nested structures like lists or trees).
    #[serde(default)]
    pub children: Vec<BlockId>,
}

/// A Document is a container for Blocks.
/// It represents the "Manifest" versioned by the system.
///
/// Note: While the AST is tree-like, we might store it as a flat list
/// in the database for query efficiency. But here in the Domain AST,
/// we represent the high-level structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub blocks: Vec<Block>,
    /// Metadata for the document (e.g. created_at, author_id)
    /// This is separate from the `nodes` table metadata, specific to the document body.
    #[serde(default)]
    pub meta: Value,
}

impl Block {
    pub fn new(block_type: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            block_type: block_type.into(),
            payload,
            children: vec![],
        }
    }
}

impl Document {
    #[allow(dead_code)]
    pub fn new(title: impl Into<String>, blocks: Vec<Block>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            blocks,
            meta: Value::Null,
        }
    }
}
