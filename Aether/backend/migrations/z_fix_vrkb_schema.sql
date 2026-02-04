-- Fix VRKB Schema to match Entities (SQLite Compatible)

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS vrkb_project_assets;
DROP TABLE IF EXISTS vrkb_members;
DROP TABLE IF EXISTS vrkb_findings;
DROP TABLE IF EXISTS vrkb_sections;
DROP TABLE IF EXISTS vrkb_specs;
DROP TABLE IF EXISTS vrkb_docs;
DROP TABLE IF EXISTS vrkb_assets;
DROP TABLE IF EXISTS vrkb_projects;

-- 1. Projects
CREATE TABLE vrkb_projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    repository_url TEXT,
    quota_bytes BIGINT NOT NULL DEFAULT 0,
    settings JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Sections
CREATE TABLE vrkb_sections (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    title TEXT NOT NULL,
    checklist JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
);

-- 3. Findings
CREATE TABLE vrkb_findings (
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
    FOREIGN KEY (section_id) REFERENCES vrkb_sections(id) ON DELETE CASCADE
);

-- 4. Assets
CREATE TABLE vrkb_assets (
    id UUID PRIMARY KEY,
    hash TEXT UNIQUE NOT NULL,
    storage_path TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Project Assets
CREATE TABLE vrkb_project_assets (
    project_id UUID NOT NULL,
    asset_id UUID NOT NULL,
    virtual_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, asset_id),
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES vrkb_assets(id) ON DELETE CASCADE
);

-- 6. Members
CREATE TABLE vrkb_members (
    project_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role TEXT NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, user_id),
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 7. Specs
CREATE TABLE vrkb_specs (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    version INT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES vrkb_projects(id) ON DELETE CASCADE
);

-- 8. Docs
CREATE TABLE vrkb_docs (
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
