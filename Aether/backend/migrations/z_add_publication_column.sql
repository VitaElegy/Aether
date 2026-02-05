-- Add publication column to supporting tables

ALTER TABLE prkb_inbox ADD COLUMN publication TEXT;
ALTER TABLE prkb_papers ADD COLUMN publication TEXT;
