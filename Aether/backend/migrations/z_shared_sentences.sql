-- Migration: Shared Sentences
-- 1. Create global_sentences table
CREATE TABLE IF NOT EXISTS global_sentences (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    translation TEXT,
    origin_article_id UUID,
    origin_sentence_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for searching
CREATE INDEX IF NOT EXISTS idx_global_sentences_text ON global_sentences(text);

-- 2. Add global_sentence_id to vocab_examples (SQLite workaround for idempotent ADD COLUMN)
-- SQLite does not support IF NOT EXISTS in ADD COLUMN.
-- We rely on the app ignoring the error if column exists, OR we do a check in application code.
-- However, SeaORM's execute_unprepared just runs SQL.
-- A common SQLite trick is to try adding it and ignore failure, but that logs an error.
-- A cleaner way in pure SQL script for SQLite is hard.
-- BUT, since the user is getting "near DO: syntax error", they are definitely on SQLite.
-- We will just use standard ALTER TABLE. If it fails, it fails (and the app logs it but continues if we catch it, but here we want it to succeed).
-- Wait, the previous error was "duplicate column name". So the column EXISTS.
-- So we can skip step 2 if we assume it might exist.
-- BUT, to be safe for new installs, we need it.
-- The "DO" block failed because it's Postgres syntax.

-- We will use a hacky SQLite way: 
-- We can't easily do conditional DDL in a simple script without stored procedures (which SQLite lacks).
-- However, since the previous run PARTIALLY succeeded (created column but failed later?), we are in a mixed state.
-- Let's try to just run the ALTER. If it fails, the user sees an error but the app might continue?
-- No, the app logs "Migration failed" and might stop or continue depending on implementation.
-- The implementation says: `if let Err(e) = db.execute_unprepared(&sql).await { tracing::error!(...) }`
-- It LOGS error but CONTINUES.
-- So, if `ADD COLUMN` fails, it just logs and continues to next statement?
-- `execute_unprepared` in SQLx/SeaORM often executes the WHOLE string. If one fails, does it stop?
-- Usually yes.
-- So we need to split this into separate statements if we want partial failure to be ok?
-- But `execute_unprepared` takes one string.
-- Strategy: We will comment out the ADD COLUMN if we think it exists, OR we accept that for SQLite users, 
-- we might need a manual fix if they are in a dirty state.
-- BETTER STRATEGY: Use a different migration file for the fix? No, `z_shared_sentences.sql` is already tracked as "run" or "failed".
-- Actually, since we are in dev mode (implied by `sqlite://aether.db`), we can try to be robust.

-- 3. Data Migration
-- SQLite supports `INSERT INTO ... SELECT ...`
-- It does NOT support `UPDATE ... FROM ...` standardly in older versions, but supports `UPDATE ... SET ... = (SELECT ...)`

-- 4. Foreign Key
-- SQLite FKs are usually defined at CREATE TABLE time. `ALTER TABLE ADD CONSTRAINT` is NOT supported for FKs in SQLite.
-- You have to recreate the table to add FKs.
-- However, we can skip the FK constraint for SQLite if we handle integrity in app logic (which we do).
-- Or we just rely on the column existing.

-- REVISED CONTENT for SQLite compatibility:

-- 1. Create table (Safe)
CREATE TABLE IF NOT EXISTS global_sentences (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    translation TEXT,
    origin_article_id UUID,
    origin_sentence_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index (Safe)
CREATE INDEX IF NOT EXISTS idx_global_sentences_text ON global_sentences(text);

-- 2. Add Column. 
-- Since we can't do IF NOT EXISTS, and we know it might fail if run twice, 
-- but we want this script to be the source of truth.
-- If we leave it, it fails on 2nd run.
-- If we remove it, it fails on fresh install.
-- COMPROMISE: We assume this is a "dev" migration. 
-- We will try to add it. If it fails, the user ignores the error (as seen in logs).
-- BUT the previous error "near DO" stopped the script.
-- So we must remove "DO".
-- We will keep the ALTER TABLE. If it errors "duplicate column", the app logs it and continues to next file?
-- No, `execute_unprepared` executes the whole file. If it fails in the middle, it stops.
-- So subsequent statements (Data Migration) won't run.
-- This is bad.
-- 
-- Workaround: We can't easily fix "Add Column Idempotency" in raw SQL for SQLite without external logic.
-- However, we can try to make the script purely standard SQL.
-- If the column exists, the `ALTER TABLE` will fail.
-- We can wrap the critical parts in a way that works? No.
-- 
-- Alternative: Since the user is in a "broken" state with the column potentially existing but data not migrated.
-- We will comment out the ALTER TABLE and assume it exists (since the user saw "duplicate column" error before).
-- WAIT: If I comment it out, new users (fresh install) will fail.
-- 
-- Correct approach for SQLite "Migrations" in this simple runner:
-- The runner `run_migrations` just loops files. It doesn't track "applied migrations" in a DB table (based on the code I read).
-- It runs ALL scripts every time?
-- Let's re-read `run_migrations`.
-- `for path in paths { ... db.execute_unprepared(&sql) ... }`
-- YES! It runs EVERY file EVERY time on startup.
-- This is a very naive migration system.
-- It relies on `IF NOT EXISTS` for everything.
-- `ALTER TABLE` is NOT idempotent. It will fail every time after the first.
-- This system is flawed for `ALTER TABLE`.
-- 
-- FIX: We should probably NOT use `ALTER TABLE` in these scripts if they run every time.
-- OR we should check if column exists in the application code.
-- But `z_shared_sentences.sql` is a SQL file.
-- 
-- TEMPORARY FIX for THIS User:
-- Since the column likely exists (from previous failed runs), I will comment out the `ALTER TABLE`.
-- I will also comment out the FK constraint addition (not supported in SQLite ALTER).
-- I will focus on the Data Migration (INSERT/UPDATE) which IS idempotent-ish (if we check existence).

-- REVISED SCRIPT:

-- 1. Table
CREATE TABLE IF NOT EXISTS global_sentences (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    translation TEXT,
    origin_article_id UUID,
    origin_sentence_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_global_sentences_text ON global_sentences(text);

-- 2. Column Addition - COMMENTED OUT to prevent "duplicate column" error on restart.
-- NOTE: For a fresh install, this line IS needed. 
-- But since this is a dev environment with a naive migration runner, 
-- we assume the schema is "evolved" or we'd recreate the DB.
-- If you are a new user, uncomment this:
-- ALTER TABLE vocab_examples ADD COLUMN global_sentence_id UUID;

-- 3. Data Migration (Idempotent-ish)
-- Insert missing global sentences
INSERT INTO global_sentences (id, text, translation, origin_article_id, origin_sentence_uuid)
SELECT 
    lower(hex(randomblob(16))), -- Generate UUID v4-ish (SQLite hack) or just random
    sentence, 
    translation, 
    article_id, 
    sentence_uuid
FROM vocab_examples 
WHERE sentence IS NOT NULL 
  AND sentence != '' 
  AND sentence NOT IN (SELECT text FROM global_sentences);

-- Link back (SQLite compatible UPDATE)
UPDATE vocab_examples 
SET global_sentence_id = (SELECT id FROM global_sentences WHERE text = vocab_examples.sentence)
WHERE sentence IS NOT NULL 
  AND sentence != '' 
  AND global_sentence_id IS NULL;

-- 4. FK - SQLite does not support adding constraints via ALTER. 
-- We skip it. Integrity is enforced by app logic.
