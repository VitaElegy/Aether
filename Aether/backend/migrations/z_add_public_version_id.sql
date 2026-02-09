-- ALTER TABLE article_details
-- ADD COLUMN IF NOT EXISTS public_version_id UUID NULL;

-- ALTER TABLE article_details
-- ADD CONSTRAINT fk_article_details_public_version
-- FOREIGN KEY (public_version_id) REFERENCES content_versions(id)
-- ON DELETE SET NULL;

-- Note: Column added manually via python script. SQLite does not support ADD CONSTRAINT in ALTER TABLE.
SELECT 1;
