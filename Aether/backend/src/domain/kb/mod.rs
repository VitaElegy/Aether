pub mod ast;
pub mod registry;

// Re-export common types
pub use ast::{Block, Document, BlockId};
pub use registry::{SchemaRegistry, BlockSchema};

mod tests;
pub mod schemas;
