-- PRKB V2: Advanced Paper Research Schema

-- 1. Authors (Entity)
CREATE TABLE IF NOT EXISTS prkb_authors (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    canonical_name TEXT, -- For disambiguation
    profile_url TEXT,
    aliases JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Venues (Entity)
CREATE TABLE IF NOT EXISTS prkb_venues (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    tier TEXT, -- Top, A, B
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. Signals (Volatile Data)
CREATE TABLE IF NOT EXISTS prkb_signals (
    paper_id UUID PRIMARY KEY,
    citation_count INTEGER NOT NULL DEFAULT 0,
    github_stars INTEGER NOT NULL DEFAULT 0,
    sota_rank TEXT,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (paper_id) REFERENCES prkb_papers(id) ON DELETE CASCADE
);

-- 4. Relations: Papers <-> Authors
CREATE TABLE IF NOT EXISTS prkb_papers_authors (
    paper_id UUID NOT NULL,
    author_id UUID NOT NULL,
    PRIMARY KEY (paper_id, author_id),
    FOREIGN KEY (paper_id) REFERENCES prkb_papers(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES prkb_authors(id) ON DELETE CASCADE
);

-- 5. Relations: Papers -> Venues
-- Adding column to existing papers table
-- SQLite does not support IF NOT EXISTS in ADD COLUMN. We use a separate block or ignore error in app logic, 
-- but for migration file simplicity in this dev environment, we'll try to add and if it fails (duplicate), the migration runner logs it but continues (as seen in logs).
-- However, the syntax error "near EXISTS" stops execution. We must remove "IF NOT EXISTS".
-- ALTER TABLE prkb_papers ADD COLUMN venue_id UUID REFERENCES prkb_venues(id) ON DELETE SET NULL;

-- 6. Add State & Local Path
-- ALTER TABLE prkb_papers ADD COLUMN state TEXT NOT NULL DEFAULT 'Inbox'; -- Inbox, Screening, Reading, Archived
-- ALTER TABLE prkb_papers ADD COLUMN pdf_local_path TEXT;

-- 7. Inbox State
-- ALTER TABLE prkb_inbox ADD COLUMN state TEXT NOT NULL DEFAULT 'Inbox';

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_prkb_authors_name ON prkb_authors(name);
CREATE INDEX IF NOT EXISTS idx_prkb_venues_name ON prkb_venues(name);
CREATE INDEX IF NOT EXISTS idx_prkb_papers_state ON prkb_papers(state);
