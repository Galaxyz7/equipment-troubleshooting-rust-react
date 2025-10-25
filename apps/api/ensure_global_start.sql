-- Ensure global start node exists
-- Run this SQL once to set up the issue selection system

-- Insert or update the global start node
INSERT INTO nodes (id, category, node_type, text, semantic_id, is_active, created_at, updated_at)
VALUES (
  '00000000-0000-0000-0000-000000000001'::uuid,
  'root',
  'Question',
  'What issue are you troubleshooting?',
  'start',
  true,
  NOW(),
  NOW()
)
ON CONFLICT (id) DO UPDATE
SET
  text = 'What issue are you troubleshooting?',
  semantic_id = 'start',
  is_active = true,
  updated_at = NOW();
