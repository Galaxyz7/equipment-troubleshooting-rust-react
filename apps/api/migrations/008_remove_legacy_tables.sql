-- Migration: Remove Legacy Questions/Answers Tables
-- =====================================================
-- This migration removes the old questions/answers data model
-- after successful migration to the new nodes/connections architecture.
--
-- IMPORTANT: Ensure all data has been migrated to nodes/connections
-- before running this migration in production!
--
-- The application now exclusively uses the nodes/connections model
-- introduced in migration 006_node_graph_refactor.sql

-- Drop foreign key constraints first (if they exist)
ALTER TABLE IF EXISTS answers DROP CONSTRAINT IF EXISTS answers_question_id_fkey;
ALTER TABLE IF EXISTS answers DROP CONSTRAINT IF EXISTS answers_next_question_id_fkey;

-- Drop indexes
DROP INDEX IF EXISTS idx_questions_category;
DROP INDEX IF EXISTS idx_questions_semantic_id;
DROP INDEX IF EXISTS idx_questions_is_active;
DROP INDEX IF EXISTS idx_answers_question_id;
DROP INDEX IF EXISTS idx_answers_next_question_id;
DROP INDEX IF EXISTS idx_answers_is_active;

-- Drop the legacy tables
DROP TABLE IF EXISTS answers;
DROP TABLE IF EXISTS questions;

-- Log successful removal
-- The tables have been safely removed after migration to nodes/connections model
