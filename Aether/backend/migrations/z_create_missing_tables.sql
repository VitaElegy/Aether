-- Migration: Create missing tables (article_details, memo_details, vocab_details, content_versions, comments)
-- This migration ensures all required tables exist for the application to function

-- Article Details (extends nodes for Article type)
CREATE TABLE IF NOT EXISTS article_details (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    category TEXT,
    body JSONB NOT NULL,
    tags TEXT NOT NULL,
    derived_data JSONB,
    FOREIGN KEY (id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Memo Details (extends nodes for Memo type)
CREATE TABLE IF NOT EXISTS memo_details (
    id UUID PRIMARY KEY,
    project_id UUID,
    color TEXT NOT NULL DEFAULT 'Yellow',
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    content JSONB NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'P2',
    due_at TIMESTAMPTZ,
    reminder_at TIMESTAMPTZ,
    tags JSONB NOT NULL DEFAULT '[]',
    FOREIGN KEY (id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Vocab Details (extends nodes for Vocabulary type)
CREATE TABLE IF NOT EXISTS vocab_details (
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
    FOREIGN KEY (root_id) REFERENCES vocab_details(id) ON DELETE SET NULL
);

-- Content Versions (version history for articles)
CREATE TABLE IF NOT EXISTS content_versions (
    id UUID PRIMARY KEY,
    node_id UUID NOT NULL,
    version INTEGER NOT NULL,
    title TEXT NOT NULL,
    body JSONB NOT NULL,
    content_hash TEXT NOT NULL,
    editor_id UUID NOT NULL,
    change_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (editor_id) REFERENCES users(id),
    UNIQUE (node_id, version)
);

-- Comments (if not exists)
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY,
    target_id TEXT NOT NULL,
    user_id UUID NOT NULL,
    parent_id UUID,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
