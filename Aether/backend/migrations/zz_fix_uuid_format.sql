-- Migration: Fix UUID Format in global_sentences
-- Problem: Previous migration used hex(randomblob(16)) which creates 32-char hex strings
-- SeaORM expects UUIDs to be stored as 16-byte binary blobs on SQLite
--
-- Simple Fix Strategy:
-- 1. Clear the corrupted global_sentences data
-- 2. Clear the references in vocab_examples 
-- 3. Let the application re-populate via the existing "create_or_find_global_sentence" logic
--    when vocabulary is next saved

-- Step 1: Clear the broken foreign key references
UPDATE vocab_examples SET global_sentence_id = NULL WHERE global_sentence_id IS NOT NULL;

-- Step 2: Delete all global_sentences (they will be recreated properly by app code)
DELETE FROM global_sentences;

-- The application code in vocab.rs correctly uses Uuid::new_v4() which SeaORM 
-- serializes as proper binary blobs. When the user next saves any vocabulary,
-- the examples will create new properly-formatted global_sentences.
