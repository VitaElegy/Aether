-- Migration: Create Blocks Table
-- Compatible with SQLite (Text storage for UUID and JSON)

CREATE TABLE IF NOT EXISTS blocks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    type TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    revision INTEGER DEFAULT 1,
    payload TEXT NOT NULL, -- JSON content stored as TEXT
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(document_id) REFERENCES nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_blocks_doc ON blocks(document_id, ordinal);
