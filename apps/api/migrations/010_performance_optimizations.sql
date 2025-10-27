-- Performance optimization indexes for troubleshoot endpoints
-- These composite indexes significantly speed up the most common query patterns

-- 1. Optimize start_session queries: nodes lookup by semantic_id + is_active
-- This index covers: WHERE semantic_id = X AND is_active = true
CREATE INDEX IF NOT EXISTS idx_nodes_semantic_active
ON nodes(semantic_id, is_active)
WHERE is_active = true;

-- 2. Optimize connection queries: from_node_id + is_active + order_index
-- This index covers: WHERE from_node_id = X AND is_active = true ORDER BY order_index
CREATE INDEX IF NOT EXISTS idx_connections_from_active_order
ON connections(from_node_id, is_active, order_index)
WHERE is_active = true;

-- 3. Optimize connection JOIN queries with target nodes
-- This covering index includes the target node id to reduce lookups
CREATE INDEX IF NOT EXISTS idx_connections_from_with_target
ON connections(from_node_id, is_active, to_node_id, order_index)
WHERE is_active = true;

-- 4. Optimize session lookups by session_id (already has unique constraint, but explicit for clarity)
-- UNIQUE constraint already creates an index, this comment documents it
-- Existing: UNIQUE(session_id) creates index automatically

-- 5. Optimize nodes table for active node lookups with included columns
-- This helps avoid table lookups for common fields
CREATE INDEX IF NOT EXISTS idx_nodes_active_complete
ON nodes(is_active, id)
INCLUDE (category, node_type, text, semantic_id, display_category)
WHERE is_active = true;

-- 6. Add index on nodes for category-based filtering (active nodes only)
CREATE INDEX IF NOT EXISTS idx_nodes_category_active
ON nodes(category, is_active)
WHERE is_active = true;

-- Performance analysis queries (for monitoring)
-- Uncomment to check index usage:
-- SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read, idx_tup_fetch
-- FROM pg_stat_user_indexes
-- WHERE schemaname = 'public'
-- ORDER BY idx_scan DESC;
