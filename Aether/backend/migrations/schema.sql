-- Consolidated Schema (Development Mode)
-- Replaces incremental migrations for cleaner dev workflow
-- Contains: Init, Blocks, System Settings

-- 1. Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    bio TEXT,
    avatar_url TEXT,
    permissions BIGINT NOT NULL,
    experience JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 1.5 Knowledge Bases
CREATE TABLE IF NOT EXISTS knowledge_bases (
    id UUID PRIMARY KEY,
    author_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    tags JSONB NOT NULL DEFAULT '[]',
    cover_image TEXT,
    cover_offset_y INTEGER NOT NULL DEFAULT 0,
    renderer_id TEXT,
    visibility TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 1.6 Nodes
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY,
    parent_id UUID,
    author_id UUID NOT NULL,
    knowledge_base_id UUID,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    permission_mode TEXT NOT NULL,
    permission_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id),
    FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE SET NULL
);

-- 1.7 Article Details (extends nodes for Article type)
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

-- 1.8 Memo Details (extends nodes for Memo type)
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

-- 1.9 Vocab Details (extends nodes for Vocabulary type)
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

-- 1.10 Content Versions (version history for articles)
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

-- 2. Articles (Legacy table, kept for backward compatibility)
CREATE TABLE IF NOT EXISTS articles (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    category TEXT NOT NULL,
    tags JSONB NOT NULL,
    summary TEXT,
    content TEXT NOT NULL,
    cover_image TEXT,
    status TEXT NOT NULL,
    author_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    published_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    FOREIGN KEY (author_id) REFERENCES users(id)
);

-- 3. Comments
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY,
    target_id TEXT NOT NULL,
    user_id UUID NOT NULL,
    parent_id UUID,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 5. VRKB Schema
CREATE TABLE IF NOT EXISTS vrkb_projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS vrkb_assets (
    id UUID PRIMARY KEY,
    hash TEXT UNIQUE NOT NULL,
    storage_path TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS vrkb_project_assets (
    project_id UUID NOT NULL,
    asset_id UUID NOT NULL,
    virtual_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, asset_id),
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES vrkb_assets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vrkb_members (
    project_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role TEXT NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, user_id),
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vrkb_specs (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    version INT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vrkb_docs (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    title TEXT NOT NULL,
    content JSONB,
    parent_id UUID,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES vrkb_docs(id) ON DELETE SET NULL,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE SET NULL
);

-- 6. Dynamic Layouts
CREATE TABLE IF NOT EXISTS layout_templates (
    id UUID PRIMARY KEY,
    renderer_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    thumbnail TEXT,
    tags JSONB,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 7. Audit Logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    action TEXT NOT NULL,
    actor_id UUID NOT NULL,
    resource_id UUID,
    details JSONB,
    ip_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 8. Groups / Teams
CREATE TABLE IF NOT EXISTS user_groups (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    owner_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 9. PRKB (Paper Research)
CREATE TABLE IF NOT EXISTS prkb_feeds (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    feed_type TEXT NOT NULL,
    last_fetched_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS prkb_inbox (
    id UUID PRIMARY KEY,
    feed_id UUID NOT NULL,
    external_id TEXT NOT NULL,
    title TEXT NOT NULL,
    authors JSONB NOT NULL DEFAULT '[]',
    abstract_text TEXT NOT NULL,
    url TEXT NOT NULL,
    pdf_url TEXT,
    publish_date TIMESTAMPTZ NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    is_saved BOOLEAN NOT NULL DEFAULT FALSE,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feed_id) REFERENCES prkb_feeds(id) ON DELETE CASCADE,
    UNIQUE (feed_id, external_id)
);

CREATE TABLE IF NOT EXISTS prkb_papers (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    authors JSONB NOT NULL DEFAULT '[]',
    abstract_text TEXT NOT NULL,
    url TEXT NOT NULL,
    pdf_url TEXT,
    publish_date TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    saved_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    tags JSONB NOT NULL DEFAULT '[]',
    arxiv_id TEXT
);

-- 10. Blocks
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

-- 11. System Settings
CREATE TABLE IF NOT EXISTS system_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO system_settings (key, value, description)
VALUES ('max_upload_size_mb', '5', 'Maximum allowed file upload size in Megabytes')
ON CONFLICT(key) DO NOTHING;
