-- Equipment Troubleshooting Database Schema
-- Improved schema with UUIDs, soft deletes, audit logging, and proper constraints

-- ============================================
-- 1. USER MANAGEMENT
-- ============================================

CREATE TYPE user_role AS ENUM ('ADMIN', 'VIEWER', 'TECH');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'VIEWER',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active);

-- ============================================
-- 2. DECISION TREE
-- ============================================

CREATE TABLE questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    semantic_id VARCHAR(100) UNIQUE NOT NULL,
    text TEXT NOT NULL,
    category VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_questions_semantic_id ON questions(semantic_id);
CREATE INDEX idx_questions_category ON questions(category);
CREATE INDEX idx_questions_active ON questions(is_active);

CREATE TABLE answers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    next_question_id UUID REFERENCES questions(id) ON DELETE SET NULL,
    conclusion_text TEXT,
    order_index INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure an answer either points to next question OR has a conclusion
    CONSTRAINT answer_routing CHECK (
        (next_question_id IS NOT NULL AND conclusion_text IS NULL) OR
        (next_question_id IS NULL AND conclusion_text IS NOT NULL)
    )
);

CREATE INDEX idx_answers_question_id ON answers(question_id);
CREATE INDEX idx_answers_next_question ON answers(next_question_id);
CREATE INDEX idx_answers_order ON answers(question_id, order_index);
CREATE INDEX idx_answers_active ON answers(is_active);

-- ============================================
-- 3. SESSION TRACKING
-- ============================================

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(100) UNIQUE NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    final_conclusion TEXT,
    tech_identifier VARCHAR(100),
    client_site VARCHAR(100),
    user_agent VARCHAR(500),
    ip_hash VARCHAR(64),
    abandoned BOOLEAN NOT NULL DEFAULT false,

    -- Ensure completed sessions have a timestamp
    CONSTRAINT session_completed CHECK (
        (completed_at IS NULL AND final_conclusion IS NULL) OR
        (completed_at IS NOT NULL)
    )
);

CREATE INDEX idx_sessions_started_at ON sessions(started_at);
CREATE INDEX idx_sessions_completed ON sessions(completed_at);
CREATE INDEX idx_sessions_site ON sessions(client_site);
CREATE INDEX idx_sessions_abandoned ON sessions(abandoned);
CREATE INDEX idx_sessions_tech ON sessions(tech_identifier);

-- ============================================
-- 4. AUDIT LOGGING
-- ============================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    target_type VARCHAR(50) NOT NULL,
    target_id UUID NOT NULL,
    before_state JSONB,
    after_state JSONB,
    ip_address VARCHAR(45),
    user_agent VARCHAR(500),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_logs_target ON audit_logs(target_type, target_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- ============================================
-- 5. UPDATED_AT TRIGGER
-- ============================================

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to all tables with updated_at column
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_questions_updated_at BEFORE UPDATE ON questions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_answers_updated_at BEFORE UPDATE ON answers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
