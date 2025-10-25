-- Add display_category field to nodes table for grouping issues
-- This allows users to categorize issues (e.g., "Electrical", "Mechanical", "General")

ALTER TABLE nodes ADD COLUMN IF NOT EXISTS display_category VARCHAR(255);

-- Add index for filtering
CREATE INDEX IF NOT EXISTS idx_nodes_display_category ON nodes(display_category);

-- Update existing nodes to have a default category (can be changed later)
UPDATE nodes SET display_category = 'General' WHERE display_category IS NULL;
