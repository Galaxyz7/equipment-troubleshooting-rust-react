-- Apply migration 007 manually
ALTER TABLE nodes ADD COLUMN IF NOT EXISTS display_category VARCHAR(255);
CREATE INDEX IF NOT EXISTS idx_nodes_display_category ON nodes(display_category);
UPDATE nodes SET display_category = 'General' WHERE display_category IS NULL;

-- Mark migrations as applied
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
VALUES
  (6, 'node graph refactor', true, decode('', 'hex'), 0),
  (7, 'add issue category', true, decode('', 'hex'), 0)
ON CONFLICT (version) DO NOTHING;
