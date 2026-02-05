-- Migration: Create vocab_examples table
-- Fixes "no such table: vocab_examples" error

CREATE TABLE IF NOT EXISTS vocab_examples (
    id UUID PRIMARY KEY,
    vocab_id UUID NOT NULL,
    sentence TEXT NOT NULL,
    translation TEXT,
    note TEXT,
    image_url TEXT,
    article_id UUID,
    sentence_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (vocab_id) REFERENCES vocab_details(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_vocab_examples_vocab_id ON vocab_examples(vocab_id);
