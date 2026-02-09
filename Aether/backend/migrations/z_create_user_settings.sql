-- Migration: Create User Module Settings
-- Stores user-specific preferences for modules (e.g., Memos pinned tags)

CREATE TABLE IF NOT EXISTS user_module_settings (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    module_key TEXT NOT NULL, -- e.g., 'memos', 'kanban'
    settings JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, module_key)
);

CREATE INDEX idx_user_module_settings_user ON user_module_settings(user_id);
