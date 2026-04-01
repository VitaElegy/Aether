-- PRKB Module Migration: PRKB-01 through PRKB-08
-- Run this migration to add all new PRKB columns and tables

-- PRKB-01: Feed Control Center - Add diagnostic columns
ALTER TABLE prkb_feeds ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE prkb_feeds ADD COLUMN IF NOT EXISTS health_status VARCHAR(50) NOT NULL DEFAULT 'unknown';
ALTER TABLE prkb_feeds ADD COLUMN IF NOT EXISTS total_fetched BIGINT NOT NULL DEFAULT 0;
ALTER TABLE prkb_feeds ADD COLUMN IF NOT EXISTS parse_errors BIGINT NOT NULL DEFAULT 0;
ALTER TABLE prkb_feeds ADD COLUMN IF NOT EXISTS last_error TEXT;

-- PRKB-02: Inbox Triage - Add priority and note
ALTER TABLE prkb_inbox ADD COLUMN IF NOT EXISTS priority INTEGER;
ALTER TABLE prkb_inbox ADD COLUMN IF NOT EXISTS note TEXT;
-- Update existing rows to have 'new' state if currently 'Inbox'
UPDATE prkb_inbox SET state = 'new' WHERE state = 'Inbox' OR state = '';

-- PRKB-03/06: Paper enhancements
ALTER TABLE prkb_papers ADD COLUMN IF NOT EXISTS pdf_status VARCHAR(50) NOT NULL DEFAULT 'not_attached';
ALTER TABLE prkb_papers ADD COLUMN IF NOT EXISTS notes TEXT;

-- PRKB-05: Collections
CREATE TABLE IF NOT EXISTS prkb_collections (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    collection_type VARCHAR(50) NOT NULL DEFAULT 'topic_collection',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS prkb_collection_items (
    collection_id UUID NOT NULL REFERENCES prkb_collections(id) ON DELETE CASCADE,
    paper_id UUID NOT NULL REFERENCES prkb_papers(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (collection_id, paper_id)
);

CREATE INDEX IF NOT EXISTS idx_prkb_collection_items_collection ON prkb_collection_items(collection_id);
CREATE INDEX IF NOT EXISTS idx_prkb_collection_items_paper ON prkb_collection_items(paper_id);

-- PRKB-07: Enhanced Signals
ALTER TABLE prkb_signals ADD COLUMN IF NOT EXISTS feed_freshness VARCHAR(50);
ALTER TABLE prkb_signals ADD COLUMN IF NOT EXISTS venue_tier VARCHAR(10);
ALTER TABLE prkb_signals ADD COLUMN IF NOT EXISTS author_recurrence INTEGER;
ALTER TABLE prkb_signals ADD COLUMN IF NOT EXISTS custom_importance INTEGER;

-- PRKB-04: Search indexes
CREATE INDEX IF NOT EXISTS idx_prkb_papers_title_trgm ON prkb_papers USING gin (title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_prkb_papers_state ON prkb_papers(state);
CREATE INDEX IF NOT EXISTS idx_prkb_papers_publish_date ON prkb_papers(publish_date);
CREATE INDEX IF NOT EXISTS idx_prkb_papers_pdf_status ON prkb_papers(pdf_status);
CREATE INDEX IF NOT EXISTS idx_prkb_inbox_state ON prkb_inbox(state);
CREATE INDEX IF NOT EXISTS idx_prkb_inbox_priority ON prkb_inbox(priority);
