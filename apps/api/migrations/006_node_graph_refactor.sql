-- Migration: Refactor to pure node-graph architecture
-- This simplifies the data model from questions/answers to nodes/connections

-- Create new nodes table (replaces questions + conclusion answers)
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category VARCHAR(255) NOT NULL,
    node_type VARCHAR(50) NOT NULL CHECK (node_type IN ('question', 'conclusion')),
    text TEXT NOT NULL,
    semantic_id VARCHAR(255),
    position_x FLOAT,
    position_y FLOAT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create new connections table (replaces answers that point to questions)
CREATE TABLE IF NOT EXISTS connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    to_node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    label VARCHAR(255) NOT NULL,
    order_index INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_nodes_category ON nodes(category);
CREATE INDEX IF NOT EXISTS idx_nodes_semantic_id ON nodes(semantic_id);
CREATE INDEX IF NOT EXISTS idx_connections_from_node ON connections(from_node_id);
CREATE INDEX IF NOT EXISTS idx_connections_to_node ON connections(to_node_id);

-- Migrate existing data from questions/answers to nodes/connections

-- Step 1: Migrate all questions to nodes
INSERT INTO nodes (id, category, node_type, text, semantic_id, is_active, created_at, updated_at)
SELECT
    id,
    category,
    'question',
    text,
    semantic_id,
    is_active,
    created_at,
    updated_at
FROM questions;

-- Step 2: Create conclusion nodes for answers that have conclusion_text
INSERT INTO nodes (id, category, node_type, text, semantic_id, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(),
    q.category,
    'conclusion',
    a.conclusion_text,
    'conclusion_' || a.id::text,
    a.is_active,
    a.created_at,
    a.updated_at
FROM answers a
JOIN questions q ON a.question_id = q.id
WHERE a.conclusion_text IS NOT NULL AND a.conclusion_text != '';

-- Step 3: Create connections for answers that point to next questions
INSERT INTO connections (id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at)
SELECT
    a.id,
    a.question_id,
    a.next_question_id,
    a.label,
    a.order_index,
    a.is_active,
    a.created_at,
    a.updated_at
FROM answers a
WHERE a.next_question_id IS NOT NULL;

-- Step 4: Create connections for answers that point to conclusion nodes
INSERT INTO connections (from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at)
SELECT
    a.question_id,
    n.id,
    a.label,
    a.order_index,
    a.is_active,
    a.created_at,
    a.updated_at
FROM answers a
JOIN questions q ON a.question_id = q.id
JOIN nodes n ON n.semantic_id = 'conclusion_' || a.id::text
WHERE a.conclusion_text IS NOT NULL AND a.conclusion_text != '';

-- Note: We keep the old questions/answers tables for now for rollback safety
-- They can be dropped later after confirming everything works:
-- DROP TABLE IF EXISTS answers;
-- DROP TABLE IF EXISTS questions;
