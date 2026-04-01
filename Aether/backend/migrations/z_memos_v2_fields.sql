-- Migration: Add MEMO-01 through MEMO-07 columns to memo_details
-- MEMO-01: Stream Core - channel
ALTER TABLE memo_details ADD COLUMN IF NOT EXISTS channel TEXT;

-- MEMO-05: Backlinks and References - linked_entities
ALTER TABLE memo_details ADD COLUMN IF NOT EXISTS linked_entities JSONB;

-- MEMO-06: Rhythm and Review - scheduling fields
ALTER TABLE memo_details ADD COLUMN IF NOT EXISTS scheduled_at TIMESTAMPTZ;
ALTER TABLE memo_details ADD COLUMN IF NOT EXISTS snoozed_until TIMESTAMPTZ;
ALTER TABLE memo_details ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ;

-- Create indexes for review queue performance
CREATE INDEX IF NOT EXISTS idx_memo_details_due_at ON memo_details(due_at) WHERE due_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memo_details_snoozed_until ON memo_details(snoozed_until) WHERE snoozed_until IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memo_details_channel ON memo_details(channel) WHERE channel IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memo_details_status ON memo_details(status);
