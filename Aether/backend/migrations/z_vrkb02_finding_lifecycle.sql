-- VRKB-02: Finding Lifecycle Extended Fields
-- Adds 7 new columns to vrkb_findings for full lifecycle support

-- Confidence level: certain / firm / tentative
ALTER TABLE vrkb_findings ADD COLUMN confidence TEXT;

-- Assigned owner (user_id reference)
ALTER TABLE vrkb_findings ADD COLUMN owner_id UUID;

-- Remediation deadline
ALTER TABLE vrkb_findings ADD COLUMN due_date TIMESTAMPTZ;

-- JSON array of affected asset references
ALTER TABLE vrkb_findings ADD COLUMN affected_assets JSONB;

-- Reproduction steps (rich text / markdown)
ALTER TABLE vrkb_findings ADD COLUMN repro_steps TEXT;

-- Remediation guidance
ALTER TABLE vrkb_findings ADD COLUMN remediation TEXT;

-- Verification note (filled when verifying fix)
ALTER TABLE vrkb_findings ADD COLUMN verification_note TEXT;

-- Index on status for kanban queries
CREATE INDEX IF NOT EXISTS idx_vrkb_findings_status ON vrkb_findings(status);

-- Index on due_date for deadline tracking
CREATE INDEX IF NOT EXISTS idx_vrkb_findings_due_date ON vrkb_findings(due_date);

-- Index on owner_id for assignment queries
CREATE INDEX IF NOT EXISTS idx_vrkb_findings_owner ON vrkb_findings(owner_id);
