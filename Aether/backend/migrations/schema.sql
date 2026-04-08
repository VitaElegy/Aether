-- ============================================================
-- Aether Consolidated Schema
-- Single source of truth for all tables.
-- ============================================================

PRAGMA foreign_keys = ON;

-- ============================================================
-- 1. Core: Users
-- ============================================================
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

-- ============================================================
-- 2. Core: Knowledge Bases
-- ============================================================
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

-- ============================================================
-- 3. Core: Nodes (universal content node)
-- ============================================================
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

-- ============================================================
-- 4. Node Detail: Articles
-- ============================================================
CREATE TABLE IF NOT EXISTS article_details (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    category TEXT,
    body JSONB NOT NULL,
    tags TEXT NOT NULL,
    derived_data JSONB,
    public_version_id UUID,
    FOREIGN KEY (id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- ============================================================
-- 5. Node Detail: Memos
-- ============================================================
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
    channel TEXT,
    linked_entities JSONB,
    scheduled_at TIMESTAMPTZ,
    snoozed_until TIMESTAMPTZ,
    reviewed_at TIMESTAMPTZ,
    FOREIGN KEY (id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- ============================================================
-- 6. Node Detail: Vocabulary
-- ============================================================
CREATE TABLE IF NOT EXISTS vocab_roots (
    id UUID PRIMARY KEY,
    root TEXT UNIQUE NOT NULL,
    meaning TEXT
);

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
    FOREIGN KEY (root_id) REFERENCES vocab_roots(id) ON DELETE SET NULL
);

-- ============================================================
-- 7. Vocabulary: Examples & Shared Sentences
-- ============================================================
CREATE TABLE IF NOT EXISTS global_sentences (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    translation TEXT,
    origin_article_id UUID,
    origin_sentence_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_global_sentences_text ON global_sentences(text);

CREATE TABLE IF NOT EXISTS vocab_examples (
    id UUID PRIMARY KEY,
    vocab_id UUID NOT NULL,
    sentence TEXT,
    translation TEXT,
    note TEXT,
    image_url TEXT,
    article_id UUID,
    sentence_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    global_sentence_id UUID,
    FOREIGN KEY (vocab_id) REFERENCES vocab_details(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_vocab_examples_vocab_id ON vocab_examples(vocab_id);

-- ============================================================
-- 8. Content Versions
-- ============================================================
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

-- ============================================================
-- 9. Blocks (document content blocks)
-- ============================================================
CREATE TABLE IF NOT EXISTS blocks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    type TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    revision INTEGER DEFAULT 1,
    payload TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(document_id) REFERENCES nodes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_blocks_doc ON blocks(document_id, ordinal);

-- ============================================================
-- 10. Drafts
-- ============================================================
CREATE TABLE IF NOT EXISTS drafts (
    article_id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    body JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (article_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- ============================================================
-- 11. Comments
-- ============================================================
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY,
    target_id TEXT NOT NULL,
    user_id UUID NOT NULL,
    parent_id UUID,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- ============================================================
-- 12. Legacy Articles (backward compatibility)
-- ============================================================
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

-- ============================================================
-- 13. PRKB: Paper Research Knowledge Base
-- ============================================================
CREATE TABLE IF NOT EXISTS prkb_feeds (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    feed_type TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    last_fetched_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    health_status TEXT NOT NULL DEFAULT 'unknown',
    total_fetched BIGINT NOT NULL DEFAULT 0,
    parse_errors BIGINT NOT NULL DEFAULT 0,
    last_error TEXT
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
    publication TEXT,
    publish_date TIMESTAMPTZ NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    is_saved BOOLEAN NOT NULL DEFAULT FALSE,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    state TEXT NOT NULL DEFAULT 'Inbox',
    priority INTEGER,
    note TEXT,
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
    publication TEXT,
    publish_date TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    saved_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    tags JSONB NOT NULL DEFAULT '[]',
    arxiv_id TEXT,
    venue_id UUID,
    state TEXT NOT NULL DEFAULT 'Inbox',
    pdf_local_path TEXT,
    metadata JSONB,
    pdf_status TEXT NOT NULL DEFAULT 'none',
    notes TEXT
);

CREATE TABLE IF NOT EXISTS prkb_authors (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    canonical_name TEXT,
    profile_url TEXT,
    aliases JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_prkb_authors_name ON prkb_authors(name);

CREATE TABLE IF NOT EXISTS prkb_venues (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    tier TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_prkb_venues_name ON prkb_venues(name);

CREATE TABLE IF NOT EXISTS prkb_signals (
    paper_id UUID PRIMARY KEY,
    citation_count INTEGER NOT NULL DEFAULT 0,
    github_stars INTEGER NOT NULL DEFAULT 0,
    sota_rank TEXT,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (paper_id) REFERENCES prkb_papers(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prkb_papers_authors (
    paper_id UUID NOT NULL,
    author_id UUID NOT NULL,
    PRIMARY KEY (paper_id, author_id),
    FOREIGN KEY (paper_id) REFERENCES prkb_papers(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES prkb_authors(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prkb_collections (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    collection_type TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS prkb_collection_items (
    collection_id UUID NOT NULL,
    paper_id UUID NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (collection_id, paper_id),
    FOREIGN KEY (collection_id) REFERENCES prkb_collections(id) ON DELETE CASCADE,
    FOREIGN KEY (paper_id) REFERENCES prkb_papers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_prkb_papers_state ON prkb_papers(state);

-- ============================================================
-- 14. VRKB: Vulnerability Research Knowledge Base
-- ============================================================
CREATE TABLE IF NOT EXISTS vrkb_projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    repository_url TEXT,
    quota_bytes BIGINT NOT NULL DEFAULT 0,
    settings JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS vrkb_sections (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    title TEXT NOT NULL,
    checklist JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vrkb_findings (
    id UUID PRIMARY KEY,
    section_id UUID NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    severity TEXT NOT NULL,
    content JSONB,
    is_triage BOOLEAN NOT NULL DEFAULT FALSE,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confidence TEXT,
    owner_id UUID,
    due_date TIMESTAMPTZ,
    affected_assets JSONB,
    repro_steps TEXT,
    remediation TEXT,
    verification_note TEXT,
    FOREIGN KEY (section_id) REFERENCES vrkb_sections(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_vrkb_findings_status ON vrkb_findings(status);
CREATE INDEX IF NOT EXISTS idx_vrkb_findings_due_date ON vrkb_findings(due_date);
CREATE INDEX IF NOT EXISTS idx_vrkb_findings_owner ON vrkb_findings(owner_id);

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

CREATE TABLE IF NOT EXISTS vrkb_evidence (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    evidence_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content JSONB,
    asset_id UUID,
    url TEXT,
    linked_entity_type TEXT,
    linked_entity_id UUID,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vrkb_checklist_items (
    id UUID PRIMARY KEY,
    section_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (section_id) REFERENCES vrkb_sections(id) ON DELETE CASCADE
);

-- ============================================================
-- 15. Dynamic Layouts
-- ============================================================
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

-- ============================================================
-- 16. Audit Logs
-- ============================================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    action TEXT NOT NULL,
    actor_id UUID NOT NULL,
    resource_id UUID,
    details JSONB,
    ip_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- 17. User Groups
-- ============================================================
CREATE TABLE IF NOT EXISTS user_groups (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    owner_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- 18. User Module Settings
-- ============================================================
CREATE TABLE IF NOT EXISTS user_module_settings (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    module_key TEXT NOT NULL,
    settings JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, module_key)
);
CREATE INDEX IF NOT EXISTS idx_user_module_settings_user ON user_module_settings(user_id);

-- ============================================================
-- 19. System Settings
-- ============================================================
CREATE TABLE IF NOT EXISTS system_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO system_settings (key, value, description)
VALUES ('max_upload_size_mb', '5', 'Maximum allowed file upload size in Megabytes')
ON CONFLICT(key) DO NOTHING;

-- ============================================================
-- 20. Memo Indexes
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_memo_details_due_at ON memo_details(due_at) WHERE due_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memo_details_snoozed_until ON memo_details(snoozed_until) WHERE snoozed_until IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memo_details_channel ON memo_details(channel) WHERE channel IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memo_details_status ON memo_details(status);
