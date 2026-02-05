-- Migration: Create vocab_roots and fix vocab_details FK
-- Created to fix "no such table: vocab_roots" error

-- 1. Create vocab_roots table
CREATE TABLE IF NOT EXISTS vocab_roots (
    id UUID PRIMARY KEY,
    root TEXT UNIQUE NOT NULL,
    meaning TEXT
);

-- 2. Fix vocab_details FK (SQLite requires table recreation)
-- First, check if we need to migrate data. 
-- Since we are in a dev/fix loop, we will attempt to preserve data if the table exists.

PRAGMA foreign_keys=OFF;

CREATE TABLE IF NOT EXISTS vocab_details_new (
    id UUID PRIMARY KEY,
    word TEXT NOT NULL,
    definition TEXT NOT NULL,
    translation TEXT,
    phonetic TEXT,
    language TEXT NOT NULL,
    status TEXT NOT NULL,
    root_id UUID,
    query_count INTEGER NOT NULL DEFAULT 0,
    is_important BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (root_id) REFERENCES vocab_roots(id) ON DELETE SET NULL
);

-- Copy data if the old table exists
INSERT OR IGNORE INTO vocab_details_new SELECT * FROM vocab_details;

-- Drop old table
DROP TABLE IF EXISTS vocab_details;

-- Rename new table
ALTER TABLE vocab_details_new RENAME TO vocab_details;

PRAGMA foreign_keys=ON;
